use eggplant::dashmap;
use eggplant::egglog;
use eggplant::prelude::*;

#[eggplant::dsl]
pub(crate) enum BaseType {
    #[eggplant::typst("text(\"int\")")]
    #[eggplant::precedence(100)]
    IntT {},
    #[eggplant::typst("text(\"bool\")")]
    #[eggplant::precedence(100)]
    BoolT {},
    #[eggplant::typst("text(\"float\")")]
    #[eggplant::precedence(100)]
    FloatT {},
    #[eggplant::typst("{ty}^*")]
    #[eggplant::precedence(95)]
    PointerT { ty: BaseType },
    #[eggplant::typst("text(\"state\")")]
    #[eggplant::precedence(100)]
    StateT {},
}

#[eggplant::dsl]
pub(crate) enum TypeList {
    #[eggplant::typst("()")]
    #[eggplant::precedence(100)]
    TNil {},
    #[eggplant::typst("{head}, {tail}")]
    #[eggplant::precedence(10)]
    TCons { head: BaseType, tail: TypeList },
}

#[eggplant::dsl]
pub(crate) enum Type {
    #[eggplant::typst("{ty}")]
    #[eggplant::precedence(100)]
    Base { ty: BaseType },
    #[eggplant::typst("({tys})")]
    #[eggplant::precedence(95)]
    TupleT { tys: TypeList },
    #[eggplant::typst("text(\"tmp_type\")")]
    #[eggplant::precedence(100)]
    TmpType {},
}

#[eggplant::dsl]
pub(crate) enum Expr {
    #[eggplant::typst("text(\"opaque\")")]
    #[eggplant::precedence(100)]
    Opaque {},
    #[eggplant::typst("text(\"arg\")({ty}, {assumption})")]
    #[eggplant::precedence(95)]
    Arg { ty: Type, assumption: Assumption },
    #[eggplant::typst("text(\"const\")({constant}, {ty}, {assumption})")]
    #[eggplant::precedence(95)]
    Const {
        constant: Constant,
        ty: Type,
        assumption: Assumption,
    },
    #[eggplant::typst("text(\"empty\")({ty}, {assumption})")]
    #[eggplant::precedence(95)]
    Empty { ty: Type, assumption: Assumption },
    #[eggplant::typst("text(\"top\")({op}, {a}, {b}, {c})")]
    #[eggplant::precedence(30)]
    Top {
        op: TernaryOp,
        a: Expr,
        b: Expr,
        c: Expr,
    },
    #[eggplant::typst("{lhs} {op} {rhs}")]
    #[eggplant::precedence(60)]
    Bop { op: BinaryOp, lhs: Expr, rhs: Expr },
    #[eggplant::typst("{op}({expr})")]
    #[eggplant::precedence(80)]
    Uop { op: UnaryOp, expr: Expr },
    #[eggplant::typst("{expr}_{index}")]
    #[eggplant::precedence(90)]
    Get { expr: Expr, index: i64 },
    #[eggplant::typst("text(\"alloc\")({id}, {amount}, {state_edge}, {pointer_ty})")]
    #[eggplant::precedence(30)]
    Alloc {
        id: i64,
        amount: Expr,
        state_edge: Expr,
        pointer_ty: BaseType,
    },
    #[eggplant::typst("text(\"call\")({name}, {arg})")]
    #[eggplant::precedence(30)]
    Call { name: String, arg: Expr },
    #[eggplant::typst("{expr}")]
    #[eggplant::precedence(100)]
    Single { expr: Expr },
    #[eggplant::typst("{expr1}, {expr2}")]
    #[eggplant::precedence(20)]
    Concat { expr1: Expr, expr2: Expr },
    #[eggplant::typst("text(\"switch\")({pred}, {inputs}, {branches})")]
    #[eggplant::precedence(20)]
    Switch {
        pred: Expr,
        inputs: Expr,
        branches: ListExpr,
    },
    #[eggplant::typst("text(\"if\")({pred}, {inputs}, {then_branch}, {else_branch})")]
    #[eggplant::precedence(20)]
    If {
        pred: Expr,
        inputs: Expr,
        then_branch: Expr,
        else_branch: Expr,
    },
    #[eggplant::typst("text(\"do_while\")({input}, {pred_and_body})")]
    #[eggplant::precedence(20)]
    DoWhile { input: Expr, pred_and_body: Expr },
    #[eggplant::typst("text(\"function\")({name}, {input_ty}, {output_ty}, {output})")]
    #[eggplant::precedence(20)]
    Function {
        name: String,
        input_ty: Type,
        output_ty: Type,
        output: Expr,
    },
}

#[eggplant::dsl]
pub(crate) enum ListExpr {
    #[eggplant::typst("{head}, {tail}")]
    #[eggplant::precedence(10)]
    Cons { head: Expr, tail: ListExpr },
    #[eggplant::typst("()")]
    #[eggplant::precedence(100)]
    Nil {},
}

#[eggplant::dsl]
pub(crate) enum Term {
    #[eggplant::typst("text(\"opaque\")")]
    #[eggplant::precedence(100)]
    OpaqueTerm {},
    #[eggplant::typst("text(\"arg\")")]
    #[eggplant::precedence(100)]
    TermArg {},
    #[eggplant::typst("{constant}")]
    #[eggplant::precedence(95)]
    TermConst { constant: Constant },
    #[eggplant::typst("text(\"empty\")")]
    #[eggplant::precedence(100)]
    TermEmpty {},
    #[eggplant::typst("text(\"top\")({op}, {a}, {b}, {c})")]
    #[eggplant::precedence(30)]
    TermTop {
        op: TernaryOp,
        a: Term,
        b: Term,
        c: Term,
    },
    #[eggplant::typst("{lhs} {op} {rhs}")]
    #[eggplant::precedence(60)]
    TermBop { op: BinaryOp, lhs: Term, rhs: Term },
    #[eggplant::typst("{op}({expr})")]
    #[eggplant::precedence(80)]
    TermUop { op: UnaryOp, expr: Term },
    #[eggplant::typst("{term}_{index}")]
    #[eggplant::precedence(90)]
    TermGet { term: Term, index: i64 },
    #[eggplant::typst("text(\"alloc\")({id}, {amount}, {state_edge}, {pointer_ty})")]
    #[eggplant::precedence(30)]
    TermAlloc {
        id: i64,
        amount: Term,
        state_edge: Term,
        pointer_ty: BaseType,
    },
    #[eggplant::typst("text(\"call\")({name}, {arg})")]
    #[eggplant::precedence(30)]
    TermCall { name: String, arg: Term },
    #[eggplant::typst("{term}")]
    #[eggplant::precedence(100)]
    TermSingle { term: Term },
    #[eggplant::typst("{term1}, {term2}")]
    #[eggplant::precedence(20)]
    TermConcat { term1: Term, term2: Term },
}

#[eggplant::dsl]
pub(crate) enum ListTerm {
    #[eggplant::typst("{head}, {tail}")]
    #[eggplant::precedence(10)]
    TermCons { head: Term, tail: ListTerm },
    #[eggplant::typst("()")]
    #[eggplant::precedence(100)]
    TermNil {},
}

#[eggplant::dsl]
pub(crate) enum ProgramType {
    #[eggplant::typst("text(\"program\")({entry}, {other_functions})")]
    #[eggplant::precedence(20)]
    Program {
        entry: Expr,
        other_functions: ListExpr,
    },
}

#[eggplant::dsl]
pub(crate) enum Assumption {
    #[eggplant::typst("text(\"in_func\")({name})")]
    #[eggplant::precedence(100)]
    InFunc { name: String },
    #[eggplant::typst("text(\"in_loop\")({input}, {pred_output})")]
    #[eggplant::precedence(100)]
    InLoop { input: Expr, pred_output: Expr },
    #[eggplant::typst("text(\"in_switch\")({branch}, {pred}, {input})")]
    #[eggplant::precedence(100)]
    InSwitch {
        branch: i64,
        pred: Expr,
        input: Expr,
    },
    #[eggplant::typst("text(\"in_if\")({pred_is_true}, {pred}, {input})")]
    #[eggplant::precedence(100)]
    InIf {
        pred_is_true: bool,
        pred: Expr,
        input: Expr,
    },
}

impl<T: eggplant::wrap::NodeDropperSgl> std::fmt::Debug for Type<T>
where
    Type<T>: EgglogNode,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.cur_sym())
    }
}

impl<T: eggplant::wrap::NodeDropperSgl> std::fmt::Debug for BaseType<T>
where
    BaseType<T>: EgglogNode,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.cur_sym())
    }
}

impl<T: eggplant::wrap::NodeDropperSgl> std::fmt::Debug for Expr<T>
where
    Expr<T>: EgglogNode,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.cur_sym())
    }
}

impl<T: eggplant::wrap::NodeDropperSgl> std::fmt::Debug for ListExpr<T>
where
    ListExpr<T>: EgglogNode,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.cur_sym())
    }
}

impl<T: eggplant::wrap::NodeDropperSgl> std::fmt::Debug for TypeList<T>
where
    TypeList<T>: EgglogNode,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.cur_sym())
    }
}

impl<T: eggplant::wrap::NodeDropperSgl> std::fmt::Debug for Term<T>
where
    Term<T>: EgglogNode,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.cur_sym())
    }
}

impl<T: eggplant::wrap::NodeDropperSgl> std::fmt::Debug for Assumption<T>
where
    Assumption<T>: EgglogNode,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.cur_sym())
    }
}

impl<T: eggplant::wrap::NodeDropperSgl> std::fmt::Debug for TernaryOp<T>
where
    TernaryOp<T>: EgglogNode,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.cur_sym())
    }
}

impl<T: eggplant::wrap::NodeDropperSgl> std::fmt::Debug for BinaryOp<T>
where
    BinaryOp<T>: EgglogNode,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.cur_sym())
    }
}

impl<T: eggplant::wrap::NodeDropperSgl> std::fmt::Debug for UnaryOp<T>
where
    UnaryOp<T>: EgglogNode,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.cur_sym())
    }
}

#[eggplant::relation]
pub(crate) struct ContextOf {
    expr: Expr,
    ctx: Assumption,
}

#[eggplant::relation]
pub(crate) struct HasArgType {
    expr: Expr,
    ty: Type,
}

#[eggplant::relation]
pub(crate) struct HasType {
    expr: Expr,
    ty: Type,
}

#[eggplant::relation]
pub(crate) struct ExprIsPure {
    expr: Expr,
}

#[eggplant::relation]
pub(crate) struct ExprIsResolved {
    expr: Expr,
}

#[eggplant::relation]
pub(crate) struct PureBaseType {
    ty: BaseType,
}

#[eggplant::relation]
pub(crate) struct PureType {
    ty: Type,
}

#[eggplant::relation]
pub(crate) struct PureTypeList {
    tylist: TypeList,
}

#[eggplant::relation]
pub(crate) struct InvCodeMotionCandidate {
    e1: Expr,
    e2: Expr,
}

#[eggplant::relation]
pub(crate) struct ExtractedExprCache {
    term: Term,
    expr: Expr,
    ctx: Assumption,
}

#[eggplant::relation]
pub(crate) struct IVTNewInputsAnalysisDemand {
    expr: Expr,
}

#[eggplant::relation]
pub(crate) struct TernaryOpIsPure {
    op: TernaryOp,
}

#[eggplant::relation]
pub(crate) struct BinaryOpIsPure {
    op: BinaryOp,
}

#[eggplant::relation]
pub(crate) struct UnaryOpIsPure {
    op: UnaryOp,
}

#[eggplant::relation]
pub(crate) struct NoAlias {
    lhs: Expr,
    rhs: Expr,
}

#[eggplant::relation]
pub(crate) struct IsIsEven {
    expr: Expr,
    input: Expr,
}

#[allow(non_camel_case_types)]
#[eggplant::relation]
pub(crate) struct NTZIterations {
    loop_expr: Expr,
    input: Expr,
    index: i64,
}

#[eggplant::relation]
pub(crate) struct BodyContainsExpr {
    body: Expr,
    expr: Expr,
}

#[eggplant::relation]
pub(crate) struct BodyContainsListExpr {
    body: Expr,
    list: ListExpr,
}

#[eggplant::relation]
pub(crate) struct IsInvExpr {
    body: Expr,
    expr: Expr,
}

#[eggplant::relation]
pub(crate) struct IsInvListExpr {
    body: Expr,
    list: ListExpr,
}

#[eggplant::relation]
pub(crate) struct IsInvListExprHelper {
    body: Expr,
    list: ListExpr,
    index: i64,
}

#[eggplant::relation]
pub(crate) struct LsrInv {
    loop_expr: Expr,
    input: Expr,
    output: Expr,
}

#[eggplant::relation]
pub(crate) struct ToSubsumeIf {
    pred: Expr,
    inputs: Expr,
    then_branch: Expr,
    else_branch: Expr,
}

#[eggplant::dsl]
pub(crate) enum Constant {
    #[eggplant::typst("{value}")]
    #[eggplant::precedence(100)]
    Int { value: i64 },
    #[eggplant::typst("text(\"{value}\")")]
    #[eggplant::precedence(100)]
    Bool { value: bool },
    #[eggplant::typst("{value}")]
    #[eggplant::precedence(100)]
    Float { value: f64 },
}

#[eggplant::dsl]
pub(crate) enum TernaryOp {
    #[eggplant::typst("text(\"write\")")]
    #[eggplant::precedence(100)]
    Write {},
    #[eggplant::typst("text(\"select\")")]
    #[eggplant::precedence(100)]
    Select {},
}

#[eggplant::dsl]
pub(crate) enum BinaryOp {
    #[eggplant::typst("&")]
    #[eggplant::precedence(100)]
    Bitand {},
    #[eggplant::typst("+")]
    #[eggplant::precedence(100)]
    Add {},
    #[eggplant::typst("-")]
    #[eggplant::precedence(100)]
    Sub {},
    #[eggplant::typst("/")]
    #[eggplant::precedence(100)]
    Div {},
    #[eggplant::typst("*")]
    #[eggplant::precedence(100)]
    Mul {},
    #[eggplant::typst("<")]
    #[eggplant::precedence(100)]
    LessThan {},
    #[eggplant::typst(">")]
    #[eggplant::precedence(100)]
    GreaterThan {},
    #[eggplant::typst("<=")]
    #[eggplant::precedence(100)]
    LessEq {},
    #[eggplant::typst(">=")]
    #[eggplant::precedence(100)]
    GreaterEq {},
    #[eggplant::typst("=")]
    #[eggplant::precedence(100)]
    Eq {},
    #[eggplant::typst("text(\"smin\")")]
    #[eggplant::precedence(100)]
    Smin {},
    #[eggplant::typst("text(\"smax\")")]
    #[eggplant::precedence(100)]
    Smax {},
    #[eggplant::typst("<<")]
    #[eggplant::precedence(100)]
    Shl {},
    #[eggplant::typst(">>")]
    #[eggplant::precedence(100)]
    Shr {},
    #[eggplant::typst("+")]
    #[eggplant::precedence(100)]
    FAdd {},
    #[eggplant::typst("-")]
    #[eggplant::precedence(100)]
    FSub {},
    #[eggplant::typst("/")]
    #[eggplant::precedence(100)]
    FDiv {},
    #[eggplant::typst("*")]
    #[eggplant::precedence(100)]
    FMul {},
    #[eggplant::typst("<")]
    #[eggplant::precedence(100)]
    FLessThan {},
    #[eggplant::typst(">")]
    #[eggplant::precedence(100)]
    FGreaterThan {},
    #[eggplant::typst("<=")]
    #[eggplant::precedence(100)]
    FLessEq {},
    #[eggplant::typst(">=")]
    #[eggplant::precedence(100)]
    FGreaterEq {},
    #[eggplant::typst("=")]
    #[eggplant::precedence(100)]
    FEq {},
    #[eggplant::typst("text(\"fmin\")")]
    #[eggplant::precedence(100)]
    Fmin {},
    #[eggplant::typst("text(\"fmax\")")]
    #[eggplant::precedence(100)]
    Fmax {},
    #[eggplant::typst("text(\"and\")")]
    #[eggplant::precedence(100)]
    And {},
    #[eggplant::typst("text(\"or\")")]
    #[eggplant::precedence(100)]
    Or {},
    #[eggplant::typst("text(\"load\")")]
    #[eggplant::precedence(100)]
    Load {},
    #[eggplant::typst("+")]
    #[eggplant::precedence(100)]
    PtrAdd {},
    #[eggplant::typst("text(\"print\")")]
    #[eggplant::precedence(100)]
    Print {},
    #[eggplant::typst("text(\"free\")")]
    #[eggplant::precedence(100)]
    Free {},
}

#[eggplant::dsl]
pub(crate) enum UnaryOp {
    #[eggplant::typst("-")]
    #[eggplant::precedence(100)]
    Neg {},
    #[eggplant::typst("text(\"abs\")")]
    #[eggplant::precedence(100)]
    Abs {},
    #[eggplant::typst("!")]
    #[eggplant::precedence(100)]
    Not {},
}

pub(crate) fn expr_section() -> String {
    // `Expr` constructors refer to schema datatypes that used to live in later
    // raw sections, so emit the mutually-referential group together.
    datatypes_section(&[
        "BaseType",
        "TypeList",
        "Type",
        "Assumption",
        "Constant",
        "TernaryOp",
        "BinaryOp",
        "UnaryOp",
        "Expr",
        "ListExpr",
    ])
}

#[allow(dead_code)]
pub(crate) fn types_section() -> String {
    datatypes_section(&["BaseType", "TypeList", "Type"])
}

#[allow(dead_code)]
pub(crate) fn assumptions_section() -> String {
    datatypes_section(&["Assumption"])
}

#[allow(dead_code)]
pub(crate) fn constants_section() -> String {
    datatypes_section(&["Constant"])
}

#[allow(dead_code)]
pub(crate) fn operators_section() -> String {
    datatypes_section(&["TernaryOp", "BinaryOp", "UnaryOp"])
}

pub(crate) fn program_type_section() -> String {
    datatypes_section(&["ProgramType"])
}

pub(crate) fn terms_section() -> String {
    datatypes_section(&["Term", "ListTerm"])
}

fn datatypes_section(datatype_names: &[&str]) -> String {
    use eggplant::egglog::ast::Command;

    let mut datatypes = eggplant::wrap::EgglogTypeRegistry::collect_type_defs()
        .into_iter()
        .find_map(|command| match command {
            Command::Datatypes { datatypes, .. } => Some(datatypes),
            _ => None,
        })
        .unwrap_or_default();

    datatypes.retain(|(_, name, _)| datatype_names.iter().any(|keep| name.as_str() == *keep));
    datatypes.sort_by(|a, b| a.1.cmp(&b.1));

    if datatypes.is_empty() {
        return String::new();
    }

    Command::Datatypes {
        span: eggplant::egglog::span!(),
        datatypes,
    }
    .to_string()
}
