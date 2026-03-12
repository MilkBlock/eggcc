use eggplant::dashmap;
use eggplant::egglog as egglog;
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

pub(crate) fn types_section() -> String {
    use eggplant::egglog::ast::Command;

    let mut datatypes = eggplant::wrap::EgglogTypeRegistry::collect_type_defs()
        .into_iter()
        .find_map(|command| match command {
            Command::Datatypes { datatypes, .. } => Some(datatypes),
            _ => None,
        })
        .unwrap_or_default();

    datatypes.retain(|(_, name, _)| matches!(name.as_str(), "BaseType" | "TypeList" | "Type"));
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
