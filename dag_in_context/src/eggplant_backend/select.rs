pub(crate) fn fragment() -> String {
    let mut out = String::new();
    out.push_str(GENERATED_MARKER);
    out.push_str(SELECT);
    out.push('\n');
    out
}

#[cfg(all(test, feature = "eggplant"))]
mod native_tests {
    use crate::eggplant_backend::test_lock;
    use egglog::{ast::Expr as EgglogExpr, EGraph, TermDag};

    fn eval_and_extract_expr(prologue: &str, expr: &str, schedule: &str) -> String {
        let binding = "__select_opt_expr";
        let program = format!("{prologue}\n(let {binding} {expr})\n{schedule}\n");

        let mut egraph = EGraph::default();
        egraph.parse_and_run_program(None, &program).unwrap();

        let mut termdag = TermDag::default();
        let (sort, value) = egraph
            .eval_expr(&EgglogExpr::Var(egglog::ast::Span::Panic, binding.into()))
            .unwrap();
        let (_, extracted) = egraph.extract(value, &mut termdag, &sort).unwrap();
        termdag.to_string(&extracted)
    }

    fn eval_and_extract_native_expr(
        prologue: &str,
        expr: &str,
        schedule: &str,
        ablate: Option<&str>,
    ) -> String {
        let binding = "__select_opt_native_expr";
        let initialization = format!("(let {binding} {expr})");
        use eggplant::egglog::ast::Expr as NativeEgglogExpr;

        crate::with_native_rules_egraph(prologue, &initialization, schedule, ablate, |egraph| {
            let (sort, value) = egraph.eval_expr(&NativeEgglogExpr::Var(
                eggplant::egglog::ast::Span::Panic,
                binding.into(),
            ))?;
            let (termdag, extracted, _) = egraph.extract_value(&sort, value)?;
            Ok(termdag.to_string(extracted))
        })
        .unwrap()
    }

    fn select_case_expr() -> String {
        let arg_ty = "(TupleT (TNil))";
        let ctx = "(InFunc \"RLCR\")";
        let pred = "(Const (Bool true) (TupleT (TNil)) (InFunc \"RLCR\"))";
        let inputs = format!("(Empty {arg_ty} {ctx})");
        let then_ctx = format!("(InIf true {pred} {inputs})");
        let else_ctx = format!("(InIf false {pred} {inputs})");
        let then_branch = format!(
            "(Single (Bop (Add) (Const (Int 1) {arg_ty} {then_ctx}) (Const (Int 2) {arg_ty} {then_ctx})))"
        );
        let else_branch = format!(
            "(Single (Bop (Sub) (Const (Int 9) {arg_ty} {else_ctx}) (Const (Int 4) {arg_ty} {else_ctx})))"
        );
        format!("(Get (If {pred} {inputs} {then_branch} {else_branch}) 0)")
    }

    fn select_schedule() -> String {
        let helpers = crate::schedule::helpers();
        format!("(run-schedule {helpers})\n(run-schedule select_opt)\n(run-schedule {helpers})")
    }

    #[test]
    fn native_feature_path_matches_text_backend_for_select_case() {
        let _guard = test_lock::lock();
        let expr = select_case_expr();
        let schedule = select_schedule();

        let text_extracted =
            eval_and_extract_expr(&crate::prologue_egglog_text(), &expr, &schedule);
        let native_extracted = eval_and_extract_native_expr(
            &crate::feature_execution_prologue(true, None),
            &expr,
            &schedule,
            None,
        );

        assert_eq!(native_extracted, text_extracted);
    }

    #[test]
    fn ablating_select_opt_changes_feature_native_result() {
        let _guard = test_lock::lock();
        let expr = select_case_expr();
        let schedule = select_schedule();

        let simplified = eval_and_extract_native_expr(
            &crate::feature_execution_prologue(true, None),
            &expr,
            &schedule,
            None,
        );
        let ablated = eval_and_extract_native_expr(
            &crate::feature_execution_prologue(true, Some("select_opt")),
            &expr,
            &crate::ablate_schedule(&schedule, "select_opt"),
            Some("select_opt"),
        );

        assert_ne!(simplified, ablated);
    }
}

const GENERATED_MARKER: &str = "; (Generated from eggplant Rust: src/eggplant_backend/select.rs)\n";
const SELECT: &str = r#"(ruleset select_opt)


;; inlined (Get thn i) makes the query faster ):
(rule
       (
        (= if_e (If pred inputs thn els))

        (ExprIsPure (Get thn i))
        (ExprIsPure (Get els i))
        
        (> 10 (expr_size (Get thn i))) ; TODO: Tune these size limits
        (> 10 (expr_size (Get els i)))
        (= (TCPair t1 c1) (ExtractedExpr (Get thn i)))
        (= (TCPair t2 c2) (ExtractedExpr (Get els i)))

        (ContextOf if_e ctx)
       )
       (
        (union (Get if_e i)
               (Top (Select) pred (TermSubst ctx inputs t1) (TermSubst ctx inputs t2)))
       )
       :ruleset select_opt
)"#;

#[cfg(feature = "eggplant")]
pub(crate) mod native {
    use super::super::schema_dsl;
    use crate::eggplant_backend::peepholes::native::PeepholeTx;
    use eggplant::prelude::{
        prim_call, BaseVar, Compare, Insertable, IntoHandleTy, PEq, PatRecSgl, RuleRunnerSgl,
        RuleSetId,
    };
    use eggplant::wrap::EgglogTy;
    use schema_dsl::{ExprPRRuleCtx, TernaryOpPRRuleCtx};

    #[derive(Clone, Copy, Debug)]
    struct TermAndCostTy;

    impl EgglogTy for TermAndCostTy {
        const TY_NAME: &'static str = "TermAndCost";
        const TY_NAME_LOWER: &'static str = "term_and_cost";
        type Valued = eggplant::wrap::Value<Self>;
        type EnumVariantMarker = ();
    }

    #[eggplant::pat_vars]
    struct SelectOptPat<PR: PatRecSgl> {
        pred: schema_dsl::Expr,
        inputs: schema_dsl::Expr,
        ctx: schema_dsl::Assumption,
        t1: schema_dsl::Term,
        t2: schema_dsl::Term,
        if_out: schema_dsl::Get,
        thn_pure: schema_dsl::ExprIsPure,
        els_pure: schema_dsl::ExprIsPure,
        if_context: schema_dsl::ContextOf,
    }

    fn select_opt_pat<PR: PatRecSgl>() -> SelectOptPat<PR> {
        let pred = schema_dsl::Expr::query_leaf();
        let inputs = schema_dsl::Expr::query_leaf();
        let thn = schema_dsl::Expr::query_leaf();
        let els = schema_dsl::Expr::query_leaf();
        let if_e = schema_dsl::If::query(&pred, &inputs, &thn, &els);
        let if_out = schema_dsl::Get::query(&if_e);
        let thn_out = schema_dsl::Get::query(&thn);
        let els_out = schema_dsl::Get::query(&els);
        let ctx = schema_dsl::Assumption::query_leaf();
        let t1 = schema_dsl::Term::query_leaf();
        let t2 = schema_dsl::Term::query_leaf();
        let size1 = BaseVar::<i64, PR>::query_named("size1");
        let size2 = BaseVar::<i64, PR>::query_named("size2");
        let c1 = BaseVar::<i64, PR>::query_named("c1");
        let c2 = BaseVar::<i64, PR>::query_named("c2");

        let same_then_index = if_out.handle_index().eq(&thn_out.handle_index());
        let same_else_index = if_out.handle_index().eq(&els_out.handle_index());
        let thn_pure = schema_dsl::ExprIsPure::query_fields(&thn_out);
        let els_pure = schema_dsl::ExprIsPure::query_fields(&els_out);
        let if_context = schema_dsl::ContextOf::query_fields(&if_e, &ctx);
        let thn_size = size1
            .handle()
            .eq(&crate::eggplant_backend::expr_size::native::expr_size::query(&thn_out).handle());
        let els_size = size2
            .handle()
            .eq(&crate::eggplant_backend::expr_size::native::expr_size::query(&els_out).handle());
        let thn_small = size1.handle().lt(&10_i64);
        let els_small = size2.handle().lt(&10_i64);
        let thn_extracted = prim_call::<TermAndCostTy>(
            "TCPair",
            vec![t1.handle().into_handle_ty(), c1.handle().into_handle_ty()],
        )
        .eq(&prim_call::<TermAndCostTy>(
            "ExtractedExpr",
            vec![thn_out.handle().into_handle_ty()],
        ));
        let els_extracted = prim_call::<TermAndCostTy>(
            "TCPair",
            vec![t2.handle().into_handle_ty(), c2.handle().into_handle_ty()],
        )
        .eq(&prim_call::<TermAndCostTy>(
            "ExtractedExpr",
            vec![els_out.handle().into_handle_ty()],
        ));

        SelectOptPat::new(pred, inputs, ctx, t1, t2, if_out, thn_pure, els_pure, if_context)
            .assert(same_then_index)
            .assert(same_else_index)
            .assert(thn_size)
            .assert(els_size)
            .assert(thn_small)
            .assert(els_small)
            .assert(thn_extracted)
            .assert(els_extracted)
    }

    pub(crate) fn register_native_rules() -> RuleSetId {
        let ruleset = PeepholeTx::new_ruleset("select_opt");

        PeepholeTx::add_rule(
            "select_opt_inline_if_output",
            ruleset,
            select_opt_pat,
            |ctx, pat| {
                let then_subst = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "TermSubst",
                    &[
                        pat.ctx.to_value(&ctx).val,
                        pat.inputs.to_value(&ctx).val,
                        pat.t1.to_value(&ctx).val,
                    ],
                ));
                let else_subst = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "TermSubst",
                    &[
                        pat.ctx.to_value(&ctx).val,
                        pat.inputs.to_value(&ctx).val,
                        pat.t2.to_value(&ctx).val,
                    ],
                ));
                let op = ctx.insert_select();
                let select = ctx.insert_top(op, pat.pred, then_subst, else_subst);
                ctx.union(pat.if_out, select);
            },
        );

        ruleset
    }
}
