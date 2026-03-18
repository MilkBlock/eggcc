#[cfg(feature = "eggplant")]
use eggplant::egglog::{
    ast::{Expr as EgglogExpr, Fact, Literal},
    prelude::span,
};
#[cfg(feature = "eggplant")]
use eggplant::prelude::{RustSpan, Span};
#[cfg(feature = "eggplant")]
use eggplant::wrap::{
    constraint::{HandleToConstrain, HandleTy, IntoConstraintFact},
    EgglogNode, EgglogTy, Insertable, RuleCtx, Value,
};

#[cfg(feature = "eggplant")]
pub(crate) fn node_expr(node: &(impl EgglogNode + 'static)) -> EgglogExpr {
    EgglogExpr::Var(span!(), format!("{}", node.cur_sym()))
}

#[cfg(feature = "eggplant")]
pub(crate) fn var_expr(name: impl Into<String>) -> EgglogExpr {
    EgglogExpr::Var(span!(), name.into())
}

#[cfg(feature = "eggplant")]
pub(crate) fn bool_expr(value: bool) -> EgglogExpr {
    EgglogExpr::Lit(span!(), Literal::Bool(value))
}

#[cfg(feature = "eggplant")]
pub(crate) fn i64_expr(value: i64) -> EgglogExpr {
    EgglogExpr::Lit(span!(), Literal::Int(value))
}

#[cfg(feature = "eggplant")]
pub(crate) fn handle_expr<T: EgglogTy>(handle: &HandleToConstrain<T>) -> EgglogExpr {
    match &handle.handle {
        HandleTy::Base { field_name, sym } => EgglogExpr::Var(span!(), format!("{sym}{field_name}")),
        HandleTy::Complex { sym } => EgglogExpr::Var(span!(), format!("{sym}")),
        HandleTy::Literal { lit } => EgglogExpr::Lit(span!(), lit.clone()),
    }
}

#[cfg(feature = "eggplant")]
pub(crate) fn call_expr(name: impl Into<String>, args: Vec<EgglogExpr>) -> EgglogExpr {
    EgglogExpr::Call(span!(), name.into(), args)
}

#[cfg(feature = "eggplant")]
#[derive(Debug, Clone)]
pub(crate) struct FactConstraint {
    fact: Fact,
}

#[cfg(feature = "eggplant")]
impl FactConstraint {
    pub(crate) fn new(name: impl Into<String>, args: Vec<EgglogExpr>) -> Self {
        Self {
            fact: Fact::Fact(call_expr(name, args)),
        }
    }
}

#[cfg(feature = "eggplant")]
impl IntoConstraintFact for FactConstraint {
    fn into_constraint_fact(&self, _egraph: &eggplant::egglog::EGraph) -> Vec<Fact> {
        vec![self.fact.clone()]
    }
}

#[cfg(feature = "eggplant")]
#[derive(Debug, Clone)]
pub(crate) struct EqCallConstraint {
    lhs: EgglogExpr,
    rhs_name: String,
    rhs_args: Vec<EgglogExpr>,
}

#[cfg(feature = "eggplant")]
impl EqCallConstraint {
    pub(crate) fn new(lhs: EgglogExpr, rhs_name: impl Into<String>, rhs_args: Vec<EgglogExpr>) -> Self {
        Self {
            lhs,
            rhs_name: rhs_name.into(),
            rhs_args,
        }
    }
}

#[cfg(feature = "eggplant")]
impl IntoConstraintFact for EqCallConstraint {
    fn into_constraint_fact(&self, _egraph: &eggplant::egglog::EGraph) -> Vec<Fact> {
        vec![Fact::Eq(
            span!(),
            self.lhs.clone(),
            call_expr(self.rhs_name.clone(), self.rhs_args.clone()),
        )]
    }
}

#[cfg(feature = "eggplant")]
#[derive(Clone, Copy, Debug)]
pub(crate) struct Inserted<T: EgglogTy>(pub(crate) Value<T>);

#[cfg(feature = "eggplant")]
impl<T: EgglogTy> Insertable<T> for Inserted<T> {
    fn to_value(&self, _ctx: &RuleCtx) -> Value<T> {
        self.0
    }
}

#[cfg(feature = "eggplant")]
pub(crate) fn insert_call<T: EgglogTy>(
    ctx: &RuleCtx,
    name: &str,
    args: &[eggplant::egglog::Value],
) -> Inserted<T> {
    Inserted(Value::new(ctx.insert(name, args)))
}
