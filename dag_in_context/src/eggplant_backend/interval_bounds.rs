#![allow(non_camel_case_types)]

use super::schema_dsl::Expr;
use eggplant::dashmap;
use eggplant::egglog;
use eggplant::prelude::*;

#[allow(dead_code)]
#[eggplant::dsl]
pub(crate) enum Bound {
    #[eggplant::typst("{value}")]
    #[eggplant::precedence(100)]
    IntB { value: i64 },
    #[eggplant::typst("{value}")]
    #[eggplant::precedence(100)]
    BoolB { value: bool },
    #[eggplant::typst("text(\"dead\")")]
    #[eggplant::precedence(100)]
    Dead {},
    #[eggplant::typst("text(\"bound_max\")({lhs}, {rhs})")]
    #[eggplant::precedence(95)]
    bound_max { lhs: Bound, rhs: Bound },
    #[eggplant::typst("text(\"bound_min\")({lhs}, {rhs})")]
    #[eggplant::precedence(95)]
    bound_min { lhs: Bound, rhs: Bound },
}

#[eggplant::func(output = Bound, merge = "(bound_max old new)")]
pub(crate) struct lo_bound {
    expr: Expr,
}

#[eggplant::func(output = Bound, merge = "(bound_min old new)")]
pub(crate) struct hi_bound {
    expr: Expr,
}
