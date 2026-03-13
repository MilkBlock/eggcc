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

fn datatypes_section(datatype_names: &[&str]) -> String {
    use eggplant::egglog::ast::Command;

    let mut datatypes = eggplant::wrap::EgglogTypeRegistry::collect_type_defs()
        .into_iter()
        .find_map(|command| match command {
            Command::Datatypes { datatypes, .. } => Some(datatypes),
            _ => None,
        })
        .unwrap_or_default();

    datatypes
        .retain(|(_, name, _)| datatype_names.iter().any(|keep| name.as_str() == *keep));
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
