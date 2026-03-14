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
    Get {
        expr: Expr,
        index: i64,
    },
    Alloc {
        id: i64,
        amount: Expr,
        state_edge: Expr,
        pointer_ty: BaseType,
    },
    Call {
        name: String,
        arg: Expr,
    },
    Single {
        expr: Expr,
    },
    Concat {
        expr1: Expr,
        expr2: Expr,
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
