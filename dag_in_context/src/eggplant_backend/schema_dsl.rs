use eggplant::dashmap;
use eggplant::egglog;
use eggplant::prelude::{RustSpan, Span};

#[eggplant::dsl]
enum BaseType {
    IntT {},
    BoolT {},
    FloatT {},
    PointerT { ty: BaseType },
    StateT {},
}

#[eggplant::dsl]
enum TypeList {
    TNil {},
    TCons { head: BaseType, tail: TypeList },
}

#[eggplant::dsl]
enum Type {
    Base { ty: BaseType },
    TupleT { tys: TypeList },
    TmpType {},
}

#[eggplant::dsl]
enum Expr {
    Opaque {},
    Arg {
        ty: Type,
        assumption: Assumption,
    },
    Const {
        constant: Constant,
        ty: Type,
        assumption: Assumption,
    },
    Empty {
        ty: Type,
        assumption: Assumption,
    },
    Top {
        op: TernaryOp,
        a: Expr,
        b: Expr,
        c: Expr,
    },
    Bop {
        op: BinaryOp,
        lhs: Expr,
        rhs: Expr,
    },
    Uop {
        op: UnaryOp,
        expr: Expr,
    },
}

#[eggplant::dsl]
enum ListExpr {
    Cons { head: Expr, tail: ListExpr },
    Nil {},
}

#[eggplant::dsl]
enum Term {
    OpaqueTerm {},
}

#[eggplant::dsl]
enum ListTerm {
    TermCons { head: Term, tail: ListTerm },
    TermNil {},
}

#[eggplant::dsl]
enum ProgramType {
    Program {
        entry: Expr,
        other_functions: ListExpr,
    },
}

#[eggplant::dsl(base = bool)]
enum Assumption {
    InFunc {
        name: String,
    },
    InLoop {
        input: Expr,
        pred_output: Expr,
    },
    InSwitch {
        branch: i64,
        pred: Expr,
        input: Expr,
    },
    InIf {
        pred_is_true: bool,
        pred: Expr,
        input: Expr,
    },
}

#[eggplant::dsl(base = bool)]
enum Constant {
    Int { value: i64 },
    Bool { value: bool },
    Float { value: f64 },
}

#[eggplant::dsl]
enum TernaryOp {
    Write {},
    Select {},
}

#[eggplant::dsl]
enum BinaryOp {
    Bitand {},
    Add {},
    Sub {},
    Div {},
    Mul {},
    LessThan {},
    GreaterThan {},
    LessEq {},
    GreaterEq {},
    Eq {},
    Smin {},
    Smax {},
    Shl {},
    Shr {},
    FAdd {},
    FSub {},
    FDiv {},
    FMul {},
    FLessThan {},
    FGreaterThan {},
    FLessEq {},
    FGreaterEq {},
    FEq {},
    Fmin {},
    Fmax {},
    And {},
    Or {},
    Load {},
    PtrAdd {},
    Print {},
    Free {},
}

#[eggplant::dsl]
enum UnaryOp {
    Neg {},
    Abs {},
    Not {},
}

pub(crate) fn types_section() -> String {
    datatypes_section(&["BaseType", "TypeList", "Type"])
}

pub(crate) fn operators_section() -> String {
    datatypes_section(&["TernaryOp", "BinaryOp", "UnaryOp"])
}

pub(crate) fn expr_section() -> String {
    // `Expr` references several other datatypes (Type/Assumption/Constant/Ops).
    // Emit them in the same `(datatypes ...)` block so egglog can resolve the
    // mutual/cyclic references without relying on section order in `schema.egg`.
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
    ])
}

pub(crate) fn list_expr_section() -> String {
    datatypes_section(&["ListExpr"])
}

pub(crate) fn program_type_section() -> String {
    datatypes_section(&["ProgramType"])
}

pub(crate) fn terms_section() -> String {
    datatypes_section(&["Term", "ListTerm"])
}

pub(crate) fn assumptions_section() -> String {
    datatypes_section(&["Assumption"])
}

pub(crate) fn constants_section() -> String {
    datatypes_section(&["Constant"])
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
