use crate::EggCCError;
use bril_rs::{
    Argument, Code, ConstOps, EffectOps, Function, Instruction, Literal, Program, Type, ValueOps,
};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::tempdir;

const CUDA_BUILTIN_ARGS: [(&str, &str); 13] = [
    ("__cuda_threadIdx_x", "threadIdx.x"),
    ("__cuda_threadIdx_y", "threadIdx.y"),
    ("__cuda_threadIdx_z", "threadIdx.z"),
    ("__cuda_blockIdx_x", "blockIdx.x"),
    ("__cuda_blockIdx_y", "blockIdx.y"),
    ("__cuda_blockIdx_z", "blockIdx.z"),
    ("__cuda_blockDim_x", "blockDim.x"),
    ("__cuda_blockDim_y", "blockDim.y"),
    ("__cuda_blockDim_z", "blockDim.z"),
    ("__cuda_gridDim_x", "gridDim.x"),
    ("__cuda_gridDim_y", "gridDim.y"),
    ("__cuda_gridDim_z", "gridDim.z"),
    ("__cuda_warpSize", "warpSize"),
];

const CUDA_PARSE_PRELUDE: &str = r#"
#define __global__
#define __device__
#define __host__
#define __shared__
#define __forceinline__
#define __noinline__
#define __managed__
#define __constant__
#define __restrict__
struct dim3 { unsigned int x; unsigned int y; unsigned int z; };
extern const dim3 threadIdx;
extern const dim3 blockIdx;
extern const dim3 blockDim;
extern const dim3 gridDim;
extern const int warpSize;
"#;

#[derive(Debug, Clone)]
pub struct CudaLoweringResult {
    pub program: Program,
    pub lowered_functions: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct AstType {
    #[serde(rename = "qualType", default)]
    qual_type: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct AstDeclRef {
    #[serde(default)]
    name: Option<String>,
    #[serde(rename = "type", default)]
    decl_type: AstType,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct AstLocation {
    #[serde(default)]
    line: Option<usize>,
    #[serde(default)]
    col: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct AstRange {
    #[serde(default)]
    begin: AstLocation,
    #[serde(default)]
    end: AstLocation,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct AstNode {
    #[serde(default)]
    kind: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    opcode: Option<String>,
    #[serde(default)]
    value: Option<String>,
    #[serde(rename = "type", default)]
    node_type: AstType,
    #[serde(rename = "referencedDecl", default)]
    referenced_decl: AstDeclRef,
    #[serde(default)]
    range: AstRange,
    #[serde(default)]
    inner: Vec<AstNode>,
}

#[derive(Debug, Clone)]
struct LoweredExpr {
    name: String,
    ty: Type,
}

#[derive(Debug, Clone)]
enum AssignTarget {
    Variable { name: String, ty: Type },
    Pointer { ptr: String, ty: Type },
}

struct LoweringContext<'a> {
    function_names: &'a HashSet<String>,
    code: Vec<Code>,
    var_types: HashMap<String, Type>,
    allocated_vars: Vec<String>,
    opaque_effect_stubs: HashSet<String>,
    temp_counter: usize,
    label_counter: usize,
}

pub fn lower_cuda_file_to_program(path: &Path) -> Result<Program, EggCCError> {
    Ok(lower_cuda_file(path)?.program)
}

pub fn lower_cuda_to_program(source: &str, unit_name: &str) -> Result<Program, EggCCError> {
    let temp_dir = tempdir()
        .map_err(|error| EggCCError::Parse(format!("failed to create temp dir: {error}")))?;
    let path = temp_dir.path().join(format!("{unit_name}.cu"));
    fs::write(&path, source).map_err(|error| {
        EggCCError::Parse(format!(
            "failed to write temporary CUDA source {}: {error}",
            path.display()
        ))
    })?;
    lower_cuda_file_to_program(&path)
}

pub fn lower_cuda_file(path: &Path) -> Result<CudaLoweringResult, EggCCError> {
    let source = fs::read_to_string(path).map_err(|error| {
        EggCCError::Parse(format!("failed to read {}: {error}", path.display()))
    })?;
    let ast = dump_cuda_ast(path, &source)?;
    let function_nodes = collect_function_nodes(&ast);
    if function_nodes.is_empty() {
        return Err(EggCCError::Parse(format!(
            "no function definitions found in {}",
            path.display()
        )));
    }

    let function_names = function_nodes
        .iter()
        .filter_map(|node| node.name.clone())
        .collect::<HashSet<_>>();

    let mut lowered_functions = Vec::new();
    let mut functions = Vec::new();
    let mut opaque_effect_stubs = HashSet::new();
    for function in function_nodes {
        let mut lowering = LoweringContext::new(&function_names);
        let lowered = lowering.lower_function(function, &source)?;
        opaque_effect_stubs.extend(lowering.opaque_effect_stubs.into_iter());
        lowered_functions.push(lowered.name.clone());
        functions.push(lowered);
    }

    for stub_name in opaque_effect_stubs {
        functions.push(make_opaque_effect_stub(&stub_name));
    }

    Ok(CudaLoweringResult {
        program: Program {
            functions,
            imports: vec![],
        },
        lowered_functions,
    })
}

fn dump_cuda_ast(path: &Path, source: &str) -> Result<AstNode, EggCCError> {
    let temp_dir = tempdir()
        .map_err(|error| EggCCError::Parse(format!("failed to create temp dir: {error}")))?;
    let shim_path = temp_dir.path().join("cuda_frontend_input.cpp");
    let shim_source = format!(
        "{CUDA_PARSE_PRELUDE}\n#line 1 \"{}\"\n{}",
        path.display(),
        source
    );
    fs::write(&shim_path, shim_source).map_err(|error| {
        EggCCError::Parse(format!(
            "failed to write temporary CUDA shim {}: {error}",
            shim_path.display()
        ))
    })?;

    let output = Command::new("clang++")
        .arg("-x")
        .arg("c++")
        .arg("-std=c++17")
        .arg("-I")
        .arg(
            path.parent()
                .unwrap_or_else(|| Path::new(".")),
        )
        .arg("-iquote")
        .arg(
            path.parent()
                .unwrap_or_else(|| Path::new(".")),
        )
        .arg("-Xclang")
        .arg("-ast-dump=json")
        .arg("-fsyntax-only")
        .arg(&shim_path)
        .output()
        .map_err(|error| EggCCError::Parse(format!("failed to run clang++: {error}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(EggCCError::Parse(format!(
            "clang++ failed while parsing CUDA subset {}: {}",
            path.display(),
            stderr
        )));
    }

    serde_json::from_slice(&output.stdout)
        .map_err(|error| EggCCError::Parse(format!("failed to parse clang AST JSON: {error}")))
}

fn collect_function_nodes(root: &AstNode) -> Vec<&AstNode> {
    let mut out = Vec::new();
    collect_function_nodes_recursive(root, &mut out);
    out
}

fn collect_function_nodes_recursive<'a>(node: &'a AstNode, out: &mut Vec<&'a AstNode>) {
    if node.kind() == "FunctionDecl" && node.body().is_some() && node.name.is_some() {
        out.push(node);
    }
    for child in &node.inner {
        collect_function_nodes_recursive(child, out);
    }
}

fn make_opaque_effect_stub(name: &str) -> Function {
    Function {
        name: name.to_string(),
        args: vec![],
        instrs: vec![Code::Instruction(Instruction::Effect {
            args: vec![],
            funcs: vec![],
            labels: vec![],
            op: EffectOps::Return,
            pos: None,
        })],
        pos: None,
        return_type: None,
    }
}

impl AstNode {
    fn kind(&self) -> &str {
        self.kind.as_deref().unwrap_or("")
    }

    fn qual_type(&self) -> &str {
        &self.node_type.qual_type
    }

    fn decl_name(&self) -> Option<&str> {
        self.referenced_decl
            .name
            .as_deref()
            .or(self.name.as_deref())
    }

    fn body(&self) -> Option<&AstNode> {
        self.inner
            .iter()
            .find(|child| child.kind() == "CompoundStmt")
    }
}

impl<'a> LoweringContext<'a> {
    fn new(function_names: &'a HashSet<String>) -> Self {
        Self {
            function_names,
            code: Vec::new(),
            var_types: HashMap::new(),
            allocated_vars: Vec::new(),
            opaque_effect_stubs: HashSet::new(),
            temp_counter: 0,
            label_counter: 0,
        }
    }

    fn lower_function(&mut self, node: &AstNode, source: &str) -> Result<Function, EggCCError> {
        let name = node
            .name
            .clone()
            .ok_or_else(|| EggCCError::Parse("function is missing a name".to_string()))?;
        let return_type = parse_function_return_type(node.qual_type())?;
        let mut args = builtin_arguments();
        for argument in &args {
            self.var_types
                .insert(argument.name.clone(), argument.arg_type.clone());
        }

        for child in &node.inner {
            if child.kind() != "ParmVarDecl" {
                continue;
            }
            let argument = lower_param(child)?;
            self.var_types
                .insert(argument.name.clone(), argument.arg_type.clone());
            args.push(argument);
        }

        let body = node.body().ok_or_else(|| {
            EggCCError::Parse(format!("function `{name}` is missing a compound body"))
        })?;
        self.lower_compound(body, source)?;
        if !self.current_block_terminated() {
            self.emit_cleanup_and_return(None, return_type.as_ref())?;
        }

        Ok(Function {
            name,
            args,
            instrs: std::mem::take(&mut self.code),
            pos: None,
            return_type,
        })
    }

    fn lower_compound(&mut self, node: &AstNode, source: &str) -> Result<(), EggCCError> {
        for child in &node.inner {
            self.lower_stmt(child, source)?;
        }
        Ok(())
    }

    fn lower_stmt(&mut self, node: &AstNode, source: &str) -> Result<(), EggCCError> {
        match node.kind() {
            "CompoundStmt" => self.lower_compound(node, source),
            "DeclStmt" => {
                for decl in &node.inner {
                    if decl.kind() == "VarDecl" {
                        self.lower_var_decl(decl, source)?;
                    }
                }
                Ok(())
            }
            "BinaryOperator" if node.opcode.as_deref() == Some("=") => {
                self.lower_assignment(node, source)
            }
            "IfStmt" => self.lower_if(node, source),
            "ForStmt" => self.lower_for(node, source),
            "WhileStmt" => self.lower_while(node, source),
            "ReturnStmt" => self.lower_return(node, source),
            "CallExpr" => {
                self.lower_call_stmt(node, source)?;
                Ok(())
            }
            "" => Ok(()),
            other => Err(EggCCError::ConversionError(format!(
                "unsupported CUDA statement kind `{other}` in phase 0"
            ))),
        }
    }

    fn lower_var_decl(&mut self, node: &AstNode, source: &str) -> Result<(), EggCCError> {
        let name = node
            .name
            .clone()
            .ok_or_else(|| EggCCError::Parse("variable declaration missing name".to_string()))?;
        if let Some((element_type, len)) = parse_array_type(node.qual_type())? {
            let len_var = self.const_int(len);
            self.emit_value(
                name.clone(),
                Type::Pointer(Box::new(element_type.clone())),
                ValueOps::Alloc,
                vec![len_var],
                vec![],
                vec![],
            );
            self.var_types
                .insert(name.clone(), Type::Pointer(Box::new(element_type)));
            self.allocated_vars.push(name);
            return Ok(());
        }

        let ty = parse_value_type(node.qual_type())?;
        self.var_types.insert(name.clone(), ty.clone());
        if let Some(initializer) = node.inner.last() {
            let rhs = self.lower_expr(initializer, source)?;
            self.emit_value(name, ty, ValueOps::Id, vec![rhs.name], vec![], vec![]);
        }
        Ok(())
    }

    fn lower_assignment(&mut self, node: &AstNode, source: &str) -> Result<(), EggCCError> {
        let [lhs, rhs] = expect_arity(node, 2, "assignment")?;
        let rhs = self.lower_expr(rhs, source)?;
        match self.lower_assign_target(lhs, source)? {
            AssignTarget::Variable { name, ty } => {
                self.emit_value(name, ty, ValueOps::Id, vec![rhs.name], vec![], vec![]);
            }
            AssignTarget::Pointer { ptr, .. } => {
                self.emit_effect(EffectOps::Store, vec![ptr, rhs.name], vec![], vec![]);
            }
        }
        Ok(())
    }

    fn lower_if(&mut self, node: &AstNode, source: &str) -> Result<(), EggCCError> {
        let condition_node = node
            .inner
            .first()
            .ok_or_else(|| EggCCError::ConversionError("if missing condition".to_string()))?;
        let then_node = node
            .inner
            .get(1)
            .ok_or_else(|| EggCCError::ConversionError("if missing then body".to_string()))?;
        let else_node = node.inner.get(2);
        let cond = self.lower_expr(condition_node, source)?;
        let then_label = self.new_label("if_then");
        let else_label = self.new_label("if_else");
        let end_label = self.new_label("if_end");

        self.emit_effect(
            EffectOps::Branch,
            vec![cond.name],
            vec![],
            vec![then_label.clone(), else_label.clone()],
        );

        self.emit_label(then_label);
        self.lower_stmt(then_node, source)?;
        if !self.current_block_terminated() {
            self.emit_effect(EffectOps::Jump, vec![], vec![], vec![end_label.clone()]);
        }

        self.emit_label(else_label);
        if let Some(else_node) = else_node {
            self.lower_stmt(else_node, source)?;
        }
        if !self.current_block_terminated() {
            self.emit_effect(EffectOps::Jump, vec![], vec![], vec![end_label.clone()]);
        }

        self.emit_label(end_label);
        Ok(())
    }

    fn lower_for(&mut self, node: &AstNode, source: &str) -> Result<(), EggCCError> {
        let init = node.inner.first();
        let cond = node.inner.get(2);
        let step = node.inner.get(3);
        let body = node
            .inner
            .get(4)
            .ok_or_else(|| EggCCError::ConversionError("for missing loop body".to_string()))?;

        if let Some(init) = init {
            self.lower_stmt(init, source)?;
        }

        let cond_label = self.new_label("for_cond");
        let body_label = self.new_label("for_body");
        let step_label = self.new_label("for_step");
        let end_label = self.new_label("for_end");

        self.emit_label(cond_label.clone());
        if let Some(cond) = cond.filter(|node| !node.kind().is_empty()) {
            let cond = self.lower_expr(cond, source)?;
            self.emit_effect(
                EffectOps::Branch,
                vec![cond.name],
                vec![],
                vec![body_label.clone(), end_label.clone()],
            );
        } else {
            self.emit_effect(EffectOps::Jump, vec![], vec![], vec![body_label.clone()]);
        }

        self.emit_label(body_label);
        self.lower_stmt(body, source)?;
        if !self.current_block_terminated() {
            self.emit_effect(EffectOps::Jump, vec![], vec![], vec![step_label.clone()]);
        }

        self.emit_label(step_label);
        if let Some(step) = step.filter(|node| !node.kind().is_empty()) {
            self.lower_stmt(step, source)?;
        }
        if !self.current_block_terminated() {
            self.emit_effect(EffectOps::Jump, vec![], vec![], vec![cond_label]);
        }

        self.emit_label(end_label);
        Ok(())
    }

    fn lower_while(&mut self, node: &AstNode, source: &str) -> Result<(), EggCCError> {
        let condition_node = node
            .inner
            .first()
            .ok_or_else(|| EggCCError::ConversionError("while missing condition".to_string()))?;
        let body_node = node
            .inner
            .get(1)
            .ok_or_else(|| EggCCError::ConversionError("while missing body".to_string()))?;

        let cond_label = self.new_label("while_cond");
        let body_label = self.new_label("while_body");
        let end_label = self.new_label("while_end");

        self.emit_label(cond_label.clone());
        let cond = self.lower_expr(condition_node, source)?;
        self.emit_effect(
            EffectOps::Branch,
            vec![cond.name],
            vec![],
            vec![body_label.clone(), end_label.clone()],
        );

        self.emit_label(body_label);
        self.lower_stmt(body_node, source)?;
        if !self.current_block_terminated() {
            self.emit_effect(EffectOps::Jump, vec![], vec![], vec![cond_label]);
        }

        self.emit_label(end_label);
        Ok(())
    }

    fn lower_return(&mut self, node: &AstNode, source: &str) -> Result<(), EggCCError> {
        let value = match node.inner.first() {
            Some(expr) => Some(self.lower_expr(expr, source)?.name),
            None => None,
        };
        self.emit_cleanup_and_return(value, None)
    }

    fn emit_cleanup_and_return(
        &mut self,
        value: Option<String>,
        function_return_type: Option<&Type>,
    ) -> Result<(), EggCCError> {
        for allocated in self.allocated_vars.clone() {
            self.emit_effect(EffectOps::Free, vec![allocated.clone()], vec![], vec![]);
        }
        let args = match (value, function_return_type) {
            (Some(value), _) => vec![value],
            (None, Some(_)) => {
                return Err(EggCCError::ConversionError(
                    "non-void function is missing a return value".to_string(),
                ))
            }
            (None, None) => vec![],
        };
        self.emit_effect(EffectOps::Return, args, vec![], vec![]);
        Ok(())
    }

    fn lower_call_stmt(&mut self, node: &AstNode, source: &str) -> Result<(), EggCCError> {
        let callee = call_target_name(node)?;
        if !is_opaque_cuda_builtin(&callee) {
            return Err(EggCCError::ConversionError(format!(
                "phase 0 only supports opaque CUDA effect calls or assignments from calls; got statement call to `{callee}`"
            )));
        }
        let args = node
            .inner
            .iter()
            .skip(1)
            .map(|arg| self.lower_expr(arg, source).map(|expr| expr.name))
            .collect::<Result<Vec<_>, _>>()?;
        self.opaque_effect_stubs.insert(callee.clone());
        self.emit_effect(EffectOps::Call, args, vec![callee], vec![]);
        Ok(())
    }

    fn lower_expr(&mut self, node: &AstNode, source: &str) -> Result<LoweredExpr, EggCCError> {
        let node = strip_expr(node);
        match node.kind() {
            "IntegerLiteral" => {
                let value = node
                    .value
                    .as_deref()
                    .ok_or_else(|| EggCCError::Parse("integer literal missing value".to_string()))?
                    .parse::<i64>()
                    .map_err(|error| {
                        EggCCError::Parse(format!("invalid integer literal: {error}"))
                    })?;
                Ok(LoweredExpr {
                    name: self.const_int(value),
                    ty: Type::Int,
                })
            }
            "CXXBoolLiteralExpr" | "BoolLiteralExpr" => {
                let value = node.value.as_deref().ok_or_else(|| {
                    EggCCError::Parse("boolean literal missing value".to_string())
                })? == "true";
                Ok(LoweredExpr {
                    name: self.const_bool(value),
                    ty: Type::Bool,
                })
            }
            "DeclRefExpr" => {
                let name = node.decl_name().ok_or_else(|| {
                    EggCCError::Parse("DeclRefExpr missing referenced declaration name".to_string())
                })?;
                let ty = self
                    .var_types
                    .get(name)
                    .cloned()
                    .or_else(|| parse_value_type(node.qual_type()).ok())
                    .ok_or_else(|| {
                        EggCCError::ConversionError(format!(
                            "unsupported declaration reference `{name}` with type `{}`",
                            node.qual_type()
                        ))
                    })?;
                Ok(LoweredExpr {
                    name: name.to_string(),
                    ty,
                })
            }
            "MemberExpr" => self.lower_member_expr(node),
            "ArraySubscriptExpr" => {
                let target = self.lower_assign_target(node, source)?;
                let AssignTarget::Pointer { ptr, ty } = target else {
                    unreachable!("array subscript should lower to pointer target");
                };
                let dest = self.new_temp("load");
                self.emit_value(
                    dest.clone(),
                    ty.clone(),
                    ValueOps::Load,
                    vec![ptr],
                    vec![],
                    vec![],
                );
                Ok(LoweredExpr { name: dest, ty })
            }
            "BinaryOperator" => self.lower_binary_expr(node, source),
            "UnaryOperator" => self.lower_unary_expr(node, source),
            "CallExpr" => self.lower_call_expr(node, source),
            other => Err(EggCCError::ConversionError(format!(
                "unsupported CUDA expression kind `{other}` in phase 0"
            ))),
        }
    }

    fn lower_member_expr(&mut self, node: &AstNode) -> Result<LoweredExpr, EggCCError> {
        let base = node.inner.first().ok_or_else(|| {
            EggCCError::ConversionError("member expression missing base".to_string())
        })?;
        let base = strip_expr(base);
        let base_name = base.decl_name().ok_or_else(|| {
            EggCCError::ConversionError("member expression base is not a declaration".to_string())
        })?;
        let member = node.name.as_deref().ok_or_else(|| {
            EggCCError::ConversionError("member expression missing member name".to_string())
        })?;
        let builtin = builtin_param_name(base_name, member).ok_or_else(|| {
            EggCCError::ConversionError(format!(
                "unsupported member expression `{base_name}.{member}` in phase 0"
            ))
        })?;
        Ok(LoweredExpr {
            name: builtin.to_string(),
            ty: Type::Int,
        })
    }

    fn lower_binary_expr(
        &mut self,
        node: &AstNode,
        source: &str,
    ) -> Result<LoweredExpr, EggCCError> {
        let [lhs_node, rhs_node] = expect_arity(node, 2, "binary operator")?;
        let lhs = self.lower_expr(lhs_node, source)?;
        let rhs = self.lower_expr(rhs_node, source)?;
        let opcode = node
            .opcode
            .as_deref()
            .ok_or_else(|| EggCCError::Parse("binary operator missing opcode".to_string()))?;
        let (op, ty) = lower_binary_opcode(opcode, &lhs.ty)?;
        let dest = self.new_temp("binop");
        self.emit_value(
            dest.clone(),
            ty.clone(),
            op,
            vec![lhs.name, rhs.name],
            vec![],
            vec![],
        );
        Ok(LoweredExpr { name: dest, ty })
    }

    fn lower_unary_expr(
        &mut self,
        node: &AstNode,
        source: &str,
    ) -> Result<LoweredExpr, EggCCError> {
        let child = node.inner.first().ok_or_else(|| {
            EggCCError::ConversionError("unary operator missing operand".to_string())
        })?;
        let expr = self.lower_expr(child, source)?;
        let opcode = node
            .opcode
            .as_deref()
            .ok_or_else(|| EggCCError::Parse("unary operator missing opcode".to_string()))?;
        match opcode {
            "+" => Ok(expr),
            "-" => {
                let dest = self.new_temp("neg");
                self.emit_value(
                    dest.clone(),
                    expr.ty.clone(),
                    ValueOps::Neg,
                    vec![expr.name],
                    vec![],
                    vec![],
                );
                Ok(LoweredExpr {
                    name: dest,
                    ty: expr.ty,
                })
            }
            "!" => {
                let dest = self.new_temp("not");
                self.emit_value(
                    dest.clone(),
                    Type::Bool,
                    ValueOps::Not,
                    vec![expr.name],
                    vec![],
                    vec![],
                );
                Ok(LoweredExpr {
                    name: dest,
                    ty: Type::Bool,
                })
            }
            other => Err(EggCCError::ConversionError(format!(
                "unsupported unary operator `{other}` in phase 0"
            ))),
        }
    }

    fn lower_call_expr(&mut self, node: &AstNode, source: &str) -> Result<LoweredExpr, EggCCError> {
        let callee = call_target_name(node)?;
        let mut args = node
            .inner
            .iter()
            .skip(1)
            .map(|arg| self.lower_expr(arg, source).map(|expr| expr.name))
            .collect::<Result<Vec<_>, _>>()?;

        let ty = parse_value_type(node.qual_type())?;
        if is_opaque_cuda_builtin(&callee) {
            return Err(EggCCError::ConversionError(format!(
                "phase 0 does not yet support value-returning opaque CUDA builtin `{callee}`"
            )));
        }

        if self.function_names.contains(&callee) {
            args.extend(
                CUDA_BUILTIN_ARGS
                    .iter()
                    .map(|(name, _)| (*name).to_string()),
            );
        }

        let dest = self.new_temp("call");
        self.emit_value(
            dest.clone(),
            ty.clone(),
            ValueOps::Call,
            args,
            vec![callee],
            vec![],
        );
        Ok(LoweredExpr { name: dest, ty })
    }

    fn lower_assign_target(
        &mut self,
        node: &AstNode,
        source: &str,
    ) -> Result<AssignTarget, EggCCError> {
        let node = strip_expr(node);
        match node.kind() {
            "DeclRefExpr" => {
                let name = node.decl_name().ok_or_else(|| {
                    EggCCError::Parse("assignment target missing declaration name".to_string())
                })?;
                let ty = self.var_types.get(name).cloned().ok_or_else(|| {
                    EggCCError::ConversionError(format!("unknown variable `{name}`"))
                })?;
                Ok(AssignTarget::Variable {
                    name: name.to_string(),
                    ty,
                })
            }
            "ArraySubscriptExpr" => {
                let [base_node, index_node] = expect_arity(node, 2, "array subscript")?;
                let base = self.lower_expr(base_node, source)?;
                let index = self.lower_expr(index_node, source)?;
                let element_type = parse_value_type(node.qual_type())?;
                let ptr = self.new_temp("ptr");
                self.emit_value(
                    ptr.clone(),
                    Type::Pointer(Box::new(element_type.clone())),
                    ValueOps::PtrAdd,
                    vec![base.name, index.name],
                    vec![],
                    vec![],
                );
                Ok(AssignTarget::Pointer {
                    ptr,
                    ty: element_type,
                })
            }
            other => Err(EggCCError::ConversionError(format!(
                "unsupported assignment target `{other}` in phase 0"
            ))),
        }
    }

    fn const_int(&mut self, value: i64) -> String {
        let name = self.new_temp("const_int");
        self.code.push(Code::Instruction(Instruction::Constant {
            dest: name.clone(),
            op: ConstOps::Const,
            pos: None,
            const_type: Type::Int,
            value: Literal::Int(value),
        }));
        name
    }

    fn const_bool(&mut self, value: bool) -> String {
        let name = self.new_temp("const_bool");
        self.code.push(Code::Instruction(Instruction::Constant {
            dest: name.clone(),
            op: ConstOps::Const,
            pos: None,
            const_type: Type::Bool,
            value: Literal::Bool(value),
        }));
        name
    }

    fn emit_label(&mut self, label: String) {
        self.code.push(Code::Label { label, pos: None });
    }

    fn emit_value(
        &mut self,
        dest: String,
        op_type: Type,
        op: ValueOps,
        args: Vec<String>,
        funcs: Vec<String>,
        labels: Vec<String>,
    ) {
        self.var_types.insert(dest.clone(), op_type.clone());
        self.code.push(Code::Instruction(Instruction::Value {
            args,
            dest,
            funcs,
            labels,
            op,
            pos: None,
            op_type,
        }));
    }

    fn emit_effect(
        &mut self,
        op: EffectOps,
        args: Vec<String>,
        funcs: Vec<String>,
        labels: Vec<String>,
    ) {
        self.code.push(Code::Instruction(Instruction::Effect {
            args,
            funcs,
            labels,
            op,
            pos: None,
        }));
    }

    fn new_temp(&mut self, prefix: &str) -> String {
        let name = format!("__cuda_{}_{}", prefix, self.temp_counter);
        self.temp_counter += 1;
        name
    }

    fn new_label(&mut self, prefix: &str) -> String {
        let name = format!("{}_{}", prefix, self.label_counter);
        self.label_counter += 1;
        name
    }

    fn current_block_terminated(&self) -> bool {
        self.code
            .iter()
            .rev()
            .find_map(|code| match code {
                Code::Instruction(Instruction::Effect { op, .. }) => Some(matches!(
                    op,
                    EffectOps::Jump | EffectOps::Branch | EffectOps::Return
                )),
                Code::Instruction(_) => Some(false),
                Code::Label { .. } => None,
            })
            .unwrap_or(false)
    }
}

fn builtin_arguments() -> Vec<Argument> {
    CUDA_BUILTIN_ARGS
        .into_iter()
        .map(|(name, _)| Argument {
            name: name.to_string(),
            arg_type: Type::Int,
        })
        .collect()
}

fn builtin_param_name(base: &str, member: &str) -> Option<&'static str> {
    CUDA_BUILTIN_ARGS
        .iter()
        .find_map(|(name, builtin)| (*builtin == format!("{base}.{member}")).then_some(*name))
}

fn is_opaque_cuda_builtin(name: &str) -> bool {
    name.starts_with("__")
}

fn lower_param(node: &AstNode) -> Result<Argument, EggCCError> {
    let name = node
        .name
        .clone()
        .ok_or_else(|| EggCCError::Parse("parameter missing name".to_string()))?;
    let arg_type = parse_value_type(node.qual_type())?;
    Ok(Argument { name, arg_type })
}

fn parse_function_return_type(qual_type: &str) -> Result<Option<Type>, EggCCError> {
    let return_prefix = qual_type
        .split(" (")
        .next()
        .map(str::trim)
        .ok_or_else(|| EggCCError::Parse(format!("invalid function type `{qual_type}`")))?;
    if return_prefix == "void" {
        Ok(None)
    } else {
        parse_value_type(return_prefix).map(Some)
    }
}

fn parse_array_type(qual_type: &str) -> Result<Option<(Type, i64)>, EggCCError> {
    let normalized = normalize_qual_type(qual_type);
    let Some((base, suffix)) = normalized.split_once('[') else {
        return Ok(None);
    };
    let len = suffix
        .strip_suffix(']')
        .ok_or_else(|| EggCCError::Parse(format!("invalid array type `{qual_type}`")))?
        .trim()
        .parse::<i64>()
        .map_err(|error| {
            EggCCError::Parse(format!("invalid array extent `{qual_type}`: {error}"))
        })?;
    Ok(Some((parse_value_type(base.trim())?, len)))
}

fn parse_value_type(qual_type: &str) -> Result<Type, EggCCError> {
    let normalized = normalize_qual_type(qual_type);
    if let Some(stripped) = normalized.strip_suffix('*') {
        return Ok(Type::Pointer(Box::new(parse_value_type(stripped.trim())?)));
    }

    match normalized.as_str() {
        "bool" => Ok(Type::Bool),
        "float" | "double" => Ok(Type::Float),
        "int" | "unsigned int" | "long" | "unsigned long" | "long long" | "unsigned long long"
        | "short" | "unsigned short" | "char" | "unsigned char" | "signed char" => Ok(Type::Int),
        other if other.starts_with("int") || other.starts_with("unsigned") => Ok(Type::Int),
        other => Err(EggCCError::ConversionError(format!(
            "unsupported CUDA type `{other}` in phase 0"
        ))),
    }
}

fn normalize_qual_type(qual_type: &str) -> String {
    qual_type
        .replace("const ", "")
        .replace("volatile ", "")
        .replace("struct ", "")
        .trim()
        .to_string()
}

fn lower_binary_opcode(opcode: &str, lhs_type: &Type) -> Result<(ValueOps, Type), EggCCError> {
    let float = matches!(lhs_type, Type::Float);
    let result = match opcode {
        "+" => (
            if float { ValueOps::Fadd } else { ValueOps::Add },
            lhs_type.clone(),
        ),
        "-" => (
            if float { ValueOps::Fsub } else { ValueOps::Sub },
            lhs_type.clone(),
        ),
        "*" => (
            if float { ValueOps::Fmul } else { ValueOps::Mul },
            lhs_type.clone(),
        ),
        "/" => (
            if float { ValueOps::Fdiv } else { ValueOps::Div },
            lhs_type.clone(),
        ),
        "==" => (if float { ValueOps::Feq } else { ValueOps::Eq }, Type::Bool),
        "<" => (if float { ValueOps::Flt } else { ValueOps::Lt }, Type::Bool),
        ">" => (if float { ValueOps::Fgt } else { ValueOps::Gt }, Type::Bool),
        "<=" => (if float { ValueOps::Fle } else { ValueOps::Le }, Type::Bool),
        ">=" => (if float { ValueOps::Fge } else { ValueOps::Ge }, Type::Bool),
        "&&" => (ValueOps::And, Type::Bool),
        "||" => (ValueOps::Or, Type::Bool),
        other => {
            return Err(EggCCError::ConversionError(format!(
                "unsupported binary operator `{other}` in phase 0"
            )))
        }
    };
    Ok(result)
}

fn strip_expr(node: &AstNode) -> &AstNode {
    match node.kind() {
        "ImplicitCastExpr"
        | "ParenExpr"
        | "ExprWithCleanups"
        | "CStyleCastExpr"
        | "MaterializeTemporaryExpr" => node.inner.last().map(strip_expr).unwrap_or(node),
        _ => node,
    }
}

fn call_target_name(node: &AstNode) -> Result<String, EggCCError> {
    let callee = node
        .inner
        .first()
        .ok_or_else(|| EggCCError::ConversionError("call expression missing callee".to_string()))?;
    let callee = strip_expr(callee);
    if callee.kind() == "DeclRefExpr" {
        return callee
            .decl_name()
            .map(ToString::to_string)
            .ok_or_else(|| EggCCError::Parse("callee declaration is unnamed".to_string()));
    }
    callee
        .inner
        .iter()
        .find_map(|child| {
            let child = strip_expr(child);
            (child.kind() == "DeclRefExpr")
                .then(|| child.decl_name().map(ToString::to_string))
                .flatten()
        })
        .ok_or_else(|| EggCCError::ConversionError("unsupported call target".to_string()))
}

fn expect_arity<'a, const N: usize>(
    node: &'a AstNode,
    expected: usize,
    what: &str,
) -> Result<[&'a AstNode; N], EggCCError> {
    node.inner
        .iter()
        .collect::<Vec<_>>()
        .try_into()
        .map_err(|_| {
            EggCCError::ConversionError(format!(
                "{what} expected {expected} children, found {}",
                node.inner.len()
            ))
        })
}

#[cfg(test)]
mod tests {
    use super::lower_cuda_file_to_program;
    use crate::Optimizer;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn lowers_shared_memory_kernel_to_rvsdg() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("phase0_shared.cu");
        fs::write(
            &file,
            r#"
extern "C" __global__ void saxpy(int *x, int *y, int n) {
  __shared__ int tile[32];
  int i = blockIdx.x * blockDim.x + threadIdx.x;
  if (i < n) {
    tile[threadIdx.x] = x[i];
    y[i] = tile[threadIdx.x] + y[i];
  }
}
"#,
        )
        .unwrap();

        let program = lower_cuda_file_to_program(&file).unwrap();
        let printed = program.to_string();
        assert!(printed.contains("@saxpy"));
        assert!(printed.contains("__cuda_threadIdx_x: int"));
        assert!(printed.contains("alloc"));
        assert!(printed.contains("ptradd"));
        assert!(printed.contains("store"));

        let _cfg = Optimizer::program_to_cfg(&program);
        let rvsdg = Optimizer::program_to_rvsdg(&program).unwrap();
        assert_eq!(rvsdg.functions.len(), 1);
    }

    #[test]
    fn lowers_for_loop_kernel_to_rvsdg() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("phase0_for.cu");
        fs::write(
            &file,
            r#"
extern "C" __global__ void fill(int *x, int n) {
  for (int i = 0; i < n; i = i + 1) {
    x[i] = i;
  }
}
"#,
        )
        .unwrap();

        let program = lower_cuda_file_to_program(&file).unwrap();
        let printed = program.to_string();
        assert!(printed.contains(".for_cond_"));
        assert!(printed.contains("br"));

        let _cfg = Optimizer::program_to_cfg(&program);
        let rvsdg = Optimizer::program_to_rvsdg(&program).unwrap();
        assert_eq!(rvsdg.functions.len(), 1);
    }

    #[test]
    fn lowers_barrier_call_to_opaque_effect_stub() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("phase0_barrier.cu");
        fs::write(
            &file,
            r#"
void __syncthreads();
extern "C" __global__ void barrier_kernel(int *x) {
  __syncthreads();
  x[threadIdx.x] = threadIdx.x;
}
"#,
        )
        .unwrap();

        let program = lower_cuda_file_to_program(&file).unwrap();
        let printed = program.to_string();
        assert!(printed.contains("call @__syncthreads;"));
        assert!(printed.contains("@__syncthreads"));

        let _cfg = Optimizer::program_to_cfg(&program);
        let rvsdg = Optimizer::program_to_rvsdg(&program).unwrap();
        assert_eq!(rvsdg.functions.len(), 2);
    }

    #[test]
    fn preserves_sibling_include_resolution() {
        let dir = tempdir().unwrap();
        let header = dir.path().join("helper.cuh");
        let file = dir.path().join("phase0_include.cu");
        fs::write(
            &header,
            r#"
inline int load_bias() { return 7; }
"#,
        )
        .unwrap();
        fs::write(
            &file,
            r#"
#include "helper.cuh"
extern "C" __global__ void include_kernel(int *x) {
  x[threadIdx.x] = load_bias();
}
"#,
        )
        .unwrap();

        let program = lower_cuda_file_to_program(&file).unwrap();
        let printed = program.to_string();
        assert!(printed.contains("@include_kernel"));
        assert!(printed.contains("call @load_bias"));

        let _cfg = Optimizer::program_to_cfg(&program);
        let rvsdg = Optimizer::program_to_rvsdg(&program).unwrap();
        assert_eq!(rvsdg.functions.len(), 1);
    }
}
