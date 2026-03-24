#[cfg(feature = "eggplant")]
use eggplant::wrap::{EgglogTy, Insertable, RuleCtx, Value};

#[cfg(feature = "eggplant")]
pub(crate) struct Inserted<T: EgglogTy>(pub(crate) Value<T>);

#[cfg(feature = "eggplant")]
impl<T: EgglogTy> Clone for Inserted<T> {
    fn clone(&self) -> Self {
        *self
    }
}

#[cfg(feature = "eggplant")]
impl<T: EgglogTy> Copy for Inserted<T> {}

#[cfg(feature = "eggplant")]
impl<T: EgglogTy> Insertable<T> for Inserted<T> {
    type MetaTy = ();

    fn to_value(&self, _ctx: &RuleCtx) -> Value<T> {
        self.0
    }

    fn meta(&self) -> Option<Self::MetaTy> {
        None
    }
}

#[cfg(feature = "eggplant")]
pub(crate) fn insert_call<T: EgglogTy>(
    ctx: &RuleCtx,
    name: &'static str,
    args: &[eggplant::egglog::Value],
) -> Inserted<T> {
    Inserted(Value::new(ctx.insert(name, args)))
}
