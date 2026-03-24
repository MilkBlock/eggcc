pub(crate) fn fragment() -> String {
    let mut out = String::new();
    out.push_str(GENERATED_MARKER);
    out.push_str(INTERVAL_ANALYSIS);
    out.push('\n');
    out
}

#[cfg(feature = "eggplant")]
pub(crate) fn native_fragment() -> String {
    const CONST_FOLD_HEADER: &str = r#"
; =================================
; Constant Folding
; =================================
"#;
    const ARITH_HEADER: &str = r#"
; =================================
; Arithmetic
; =================================
"#;

    let (prefix, const_fold_and_after) = INTERVAL_ANALYSIS
        .split_once(CONST_FOLD_HEADER)
        .expect("interval_analysis::native_fragment() expects the constant-fold header");
    let (_, arithmetic_and_after) = const_fold_and_after
        .split_once(ARITH_HEADER)
        .expect("interval_analysis::native_fragment() expects the arithmetic header");

    let mut out = String::new();
    out.push_str(GENERATED_MARKER);
    out.push_str(prefix);
    out.push_str(ARITH_HEADER);
    out.push_str(arithmetic_and_after);
    out.push('\n');
    out
}

const GENERATED_MARKER: &str =
    "; (Generated from eggplant Rust: src/eggplant_backend/interval_analysis.rs)\n";
const INTERVAL_ANALYSIS: &str = include_str!("../interval_analysis.egg");

#[cfg(feature = "eggplant")]
pub(crate) mod native {
    use super::super::native_rule_helpers::insert_call;
    use super::super::schema_dsl::{self, ExprRuleCtx};
    use crate::eggplant_backend::interval_bounds::{
        hi_bound, lo_bound, BoolB, BoolBTy, IntB, IntBTy,
    };
    use crate::eggplant_backend::peepholes::native::PeepholeTx;
    use eggplant::prelude::{BaseVar, IntoHandleTy, PEq, PatRecSgl, RuleRunnerSgl, RuleSetId};

    fn type_leaf<PR: PatRecSgl>() -> schema_dsl::Type<PR> {
        schema_dsl::Type::query_leaf()
    }

    fn assumption_leaf<PR: PatRecSgl>() -> schema_dsl::Assumption<PR> {
        schema_dsl::Assumption::query_leaf()
    }

    fn int_bound<PR: PatRecSgl>() -> crate::eggplant_backend::interval_bounds::Bound<PR, IntBTy> {
        IntB::query()
    }

    fn bool_bound<PR: PatRecSgl>() -> crate::eggplant_backend::interval_bounds::Bound<PR, BoolBTy> {
        BoolB::query()
    }

    #[eggplant::pat_vars]
    struct IntConstFoldPat<PR: PatRecSgl> {
        expr: schema_dsl::Expr,
        ty: schema_dsl::Type,
        ctx: schema_dsl::Assumption,
        value: i64,
    }

    #[eggplant::pat_vars]
    struct BoolConstFoldPat<PR: PatRecSgl> {
        expr: schema_dsl::Expr,
        ty: schema_dsl::Type,
        ctx: schema_dsl::Assumption,
        value: bool,
    }

    #[eggplant::pat_vars]
    struct BoolKnownPat<PR: PatRecSgl> {
        expr: schema_dsl::Expr,
        ty: schema_dsl::Type,
        ctx: schema_dsl::Assumption,
    }

    fn int_const_fold_pat<PR: PatRecSgl>() -> IntConstFoldPat<PR> {
        let expr = schema_dsl::Expr::query_leaf();
        let ty = schema_dsl::Type::query_leaf();
        let ctx = assumption_leaf::<PR>();
        let value = BaseVar::<i64, PR>::query_named("interval_const_fold_int");
        let lo = lo_bound::query(&expr);
        let hi = hi_bound::query(&expr);
        let lo_int = int_bound::<PR>();
        let hi_int = int_bound::<PR>();
        let lo_matches = lo.handle().eq(&lo_int.handle());
        let hi_matches = hi.handle().eq(&hi_int.handle());
        let lo_value_matches = lo_int.handle_value().eq(&value.handle());
        let hi_value_matches = hi_int.handle_value().eq(&value.handle());
        let has_arg_type = eggplant::wrap::FactCallConstraint {
            op: "HasArgType",
            operands: vec![expr.handle().into_handle_ty(), ty.handle().into_handle_ty()],
        };
        let context_of = eggplant::wrap::FactCallConstraint {
            op: "ContextOf",
            operands: vec![
                expr.handle().into_handle_ty(),
                ctx.handle().into_handle_ty(),
            ],
        };

        IntConstFoldPat::new(expr, ty, ctx, value)
            .assert(lo_matches)
            .assert(hi_matches)
            .assert(lo_value_matches)
            .assert(hi_value_matches)
            .assert(has_arg_type)
            .assert(context_of)
    }

    fn bool_const_fold_pat<PR: PatRecSgl>() -> BoolConstFoldPat<PR> {
        let expr = schema_dsl::Expr::query_leaf();
        let ty = type_leaf::<PR>();
        let ctx = assumption_leaf::<PR>();
        let value = BaseVar::<bool, PR>::query_named("interval_const_fold_bool");
        let lo = lo_bound::query(&expr);
        let hi = hi_bound::query(&expr);
        let lo_bool = bool_bound::<PR>();
        let hi_bool = bool_bound::<PR>();
        let lo_matches = lo.handle().eq(&lo_bool.handle());
        let hi_matches = hi.handle().eq(&hi_bool.handle());
        let lo_value_matches = lo_bool.handle_value().eq(&value.handle());
        let hi_value_matches = hi_bool.handle_value().eq(&value.handle());
        let has_arg_type = eggplant::wrap::FactCallConstraint {
            op: "HasArgType",
            operands: vec![expr.handle().into_handle_ty(), ty.handle().into_handle_ty()],
        };
        let context_of = eggplant::wrap::FactCallConstraint {
            op: "ContextOf",
            operands: vec![
                expr.handle().into_handle_ty(),
                ctx.handle().into_handle_ty(),
            ],
        };

        BoolConstFoldPat::new(expr, ty, ctx, value)
            .assert(lo_matches)
            .assert(hi_matches)
            .assert(lo_value_matches)
            .assert(hi_value_matches)
            .assert(has_arg_type)
            .assert(context_of)
    }

    fn lower_true_pat<PR: PatRecSgl>() -> BoolKnownPat<PR> {
        let expr = schema_dsl::Expr::query_leaf();
        let ty = type_leaf::<PR>();
        let ctx = assumption_leaf::<PR>();
        let lo = lo_bound::query(&expr);
        let lo_bool = bool_bound::<PR>();
        let lo_matches = lo.handle().eq(&lo_bool.handle());
        let lo_value_matches = lo_bool.handle_value().eq(&true);
        let has_arg_type = eggplant::wrap::FactCallConstraint {
            op: "HasArgType",
            operands: vec![expr.handle().into_handle_ty(), ty.handle().into_handle_ty()],
        };
        let context_of = eggplant::wrap::FactCallConstraint {
            op: "ContextOf",
            operands: vec![
                expr.handle().into_handle_ty(),
                ctx.handle().into_handle_ty(),
            ],
        };

        BoolKnownPat::new(expr, ty, ctx)
            .assert(lo_matches)
            .assert(lo_value_matches)
            .assert(has_arg_type)
            .assert(context_of)
    }

    fn upper_false_pat<PR: PatRecSgl>() -> BoolKnownPat<PR> {
        let expr = schema_dsl::Expr::query_leaf();
        let ty = type_leaf::<PR>();
        let ctx = assumption_leaf::<PR>();
        let hi = hi_bound::query(&expr);
        let hi_bool = bool_bound::<PR>();
        let hi_matches = hi.handle().eq(&hi_bool.handle());
        let hi_value_matches = hi_bool.handle_value().eq(&false);
        let has_arg_type = eggplant::wrap::FactCallConstraint {
            op: "HasArgType",
            operands: vec![expr.handle().into_handle_ty(), ty.handle().into_handle_ty()],
        };
        let context_of = eggplant::wrap::FactCallConstraint {
            op: "ContextOf",
            operands: vec![
                expr.handle().into_handle_ty(),
                ctx.handle().into_handle_ty(),
            ],
        };

        BoolKnownPat::new(expr, ty, ctx)
            .assert(hi_matches)
            .assert(hi_value_matches)
            .assert(has_arg_type)
            .assert(context_of)
    }

    pub(crate) fn register_native_rules() -> RuleSetId {
        let ruleset = RuleSetId("interval-analysis");

        PeepholeTx::add_rule(
            "interval_analysis_const_fold_int",
            ruleset,
            int_const_fold_pat,
            |ctx, pat| {
                let value = ctx._intern_base::<i64, i64>(ctx.devalue(pat.value));
                let constant = insert_call::<schema_dsl::Constant>(&ctx.ctx, "Int", &[value]);
                let folded = ctx.ctx.insert_const(constant, pat.ty, pat.ctx);
                ctx.union(pat.expr, folded);
            },
        );
        PeepholeTx::add_rule(
            "interval_analysis_const_fold_bool",
            ruleset,
            bool_const_fold_pat,
            |ctx, pat| {
                let value = ctx._intern_base::<bool, bool>(ctx.devalue(pat.value));
                let constant = insert_call::<schema_dsl::Constant>(&ctx.ctx, "Bool", &[value]);
                let folded = ctx.ctx.insert_const(constant, pat.ty, pat.ctx);
                ctx.union(pat.expr, folded);
            },
        );
        PeepholeTx::add_rule(
            "interval_analysis_lower_true",
            ruleset,
            lower_true_pat,
            |ctx, pat| {
                let value = ctx._intern_base::<bool, bool>(true);
                let constant = insert_call::<schema_dsl::Constant>(&ctx.ctx, "Bool", &[value]);
                let folded = ctx.ctx.insert_const(constant, pat.ty, pat.ctx);
                ctx.union(pat.expr, folded);
            },
        );
        PeepholeTx::add_rule(
            "interval_analysis_upper_false",
            ruleset,
            upper_false_pat,
            |ctx, pat| {
                let value = ctx._intern_base::<bool, bool>(false);
                let constant = insert_call::<schema_dsl::Constant>(&ctx.ctx, "Bool", &[value]);
                let folded = ctx.ctx.insert_const(constant, pat.ty, pat.ctx);
                ctx.union(pat.expr, folded);
            },
        );

        ruleset
    }
}

#[cfg(all(test, feature = "eggplant"))]
mod native_tests {
    use crate::eggplant_backend::test_lock;
    use egglog::EGraph;

    fn candidate_parts() -> (String, String) {
        let expr =
            "(Bop (Add) (Const (Int 1) (Base (IntT)) (InFunc \"RLCR\")) (Const (Int 2) (Base (IntT)) (InFunc \"RLCR\")))"
                .to_string();
        let expected = "(Const (Int 3) (Base (IntT)) (InFunc \"RLCR\"))".to_string();
        (expr, expected)
    }

    fn interval_schedule() -> String {
        let schedule = crate::schedule::types_and_indexing();
        format!("(run-schedule {schedule})\n(run-schedule (saturate interval-analysis))\n")
    }

    fn text_interval_holds(prologue: &str, expr: &str, expected: &str, schedule: &str) {
        let program = format!(
            "{prologue}\n(let __rlcr_expr {expr})\n{schedule}(check (= __rlcr_expr {expected}))\n"
        );
        let mut egraph = EGraph::default();
        egraph.parse_and_run_program(None, &program).unwrap();
    }

    fn native_interval_holds(
        prologue: &str,
        expr: &str,
        expected: &str,
        schedule: &str,
        ablate: Option<&str>,
    ) -> std::result::Result<(), eggplant::egglog::Error> {
        let initialization = format!("(let __rlcr_expr {expr})");
        crate::with_native_rules_egraph(prologue, &initialization, schedule, ablate, |egraph| {
            egraph.parse_and_run_program(None, &format!("(check (= __rlcr_expr {expected}))"))?;
            Ok(())
        })
    }

    #[test]
    fn native_feature_path_matches_text_backend_for_interval_analysis_case() {
        let _guard = test_lock::lock();
        let (expr, expected) = candidate_parts();
        let schedule = interval_schedule();

        text_interval_holds(&crate::prologue_egglog_text(), &expr, &expected, &schedule);
        native_interval_holds(
            &crate::feature_execution_prologue(true, None),
            &expr,
            &expected,
            &schedule,
            None,
        )
        .unwrap();

        let ablated = native_interval_holds(
            &crate::feature_execution_prologue(true, Some("interval-analysis")),
            &expr,
            &expected,
            &crate::ablate_schedule(&schedule, "interval-analysis"),
            Some("interval-analysis"),
        );

        assert!(
            ablated.is_err(),
            "ablating interval-analysis should make the constant-fold witness fail",
        );
    }
}
