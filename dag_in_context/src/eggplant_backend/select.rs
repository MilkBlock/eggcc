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
            Ok(termdag.to_string(&extracted))
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
        format!(
            "(run-schedule {helpers})\n(run-schedule select_opt)\n(run-schedule {helpers})"
        )
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
        
        (> 10 (Expr-size (Get thn i))) ; TODO: Tune these size limits
        (> 10 (Expr-size (Get els i)))
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
    use super::super::native_rule_helpers::{
        call_expr, i64_expr, insert_call, node_expr, var_expr, EqCallConstraint, FactConstraint,
    };
    use super::super::schema_dsl;
    use crate::eggplant_backend::peepholes::native::{PeepholePatRec, PeepholeTx};
    use eggplant::prelude::{Insertable, PatRecSgl, RuleRunnerSgl, RuleSetId};

    pub(crate) fn register_native_rules() -> RuleSetId {
        let ruleset = PeepholeTx::new_ruleset("select_opt");

        PeepholeTx::add_rule(
            "select_opt_inline_if_output",
            ruleset,
            || {
                let pred = schema_dsl::Expr::query_leaf();
                let inputs = schema_dsl::Expr::query_leaf();
                let thn = schema_dsl::Expr::query_leaf();
                let els = schema_dsl::Expr::query_leaf();
                let if_e = schema_dsl::If::query(&pred, &inputs, &thn, &els);
                let if_out: schema_dsl::Expr<PeepholePatRec, _> = schema_dsl::Expr::query_leaf();
                let thn_out: schema_dsl::Expr<PeepholePatRec, _> = schema_dsl::Expr::query_leaf();
                let els_out: schema_dsl::Expr<PeepholePatRec, _> = schema_dsl::Expr::query_leaf();
                let ctx: schema_dsl::Assumption<PeepholePatRec, _> =
                    schema_dsl::Assumption::query_leaf();
                let t1: schema_dsl::Term<PeepholePatRec, _> = schema_dsl::Term::query_leaf();
                let t2: schema_dsl::Term<PeepholePatRec, _> = schema_dsl::Term::query_leaf();

                PeepholePatRec::on_new_constraint(EqCallConstraint::new(
                    node_expr(&if_out),
                    "Get",
                    vec![node_expr(&if_e), var_expr("i")],
                ));
                PeepholePatRec::on_new_constraint(EqCallConstraint::new(
                    node_expr(&thn_out),
                    "Get",
                    vec![node_expr(&thn), var_expr("i")],
                ));
                PeepholePatRec::on_new_constraint(EqCallConstraint::new(
                    node_expr(&els_out),
                    "Get",
                    vec![node_expr(&els), var_expr("i")],
                ));
                PeepholePatRec::on_new_constraint(FactConstraint::new(
                    "ExprIsPure",
                    vec![node_expr(&thn_out)],
                ));
                PeepholePatRec::on_new_constraint(FactConstraint::new(
                    "ExprIsPure",
                    vec![node_expr(&els_out)],
                ));
                PeepholePatRec::on_new_constraint(FactConstraint::new(
                    "ContextOf",
                    vec![node_expr(&if_e), node_expr(&ctx)],
                ));
                PeepholePatRec::on_new_constraint(EqCallConstraint::new(
                    var_expr("size1"),
                    "Expr-size",
                    vec![node_expr(&thn_out)],
                ));
                PeepholePatRec::on_new_constraint(EqCallConstraint::new(
                    var_expr("size2"),
                    "Expr-size",
                    vec![node_expr(&els_out)],
                ));
                PeepholePatRec::on_new_constraint(FactConstraint::new(
                    ">",
                    vec![i64_expr(10), var_expr("size1")],
                ));
                PeepholePatRec::on_new_constraint(FactConstraint::new(
                    ">",
                    vec![i64_expr(10), var_expr("size2")],
                ));
                PeepholePatRec::on_new_constraint(EqCallConstraint::new(
                    call_expr("TCPair", vec![node_expr(&t1), var_expr("c1")]),
                    "ExtractedExpr",
                    vec![node_expr(&thn_out)],
                ));
                PeepholePatRec::on_new_constraint(EqCallConstraint::new(
                    call_expr("TCPair", vec![node_expr(&t2), var_expr("c2")]),
                    "ExtractedExpr",
                    vec![node_expr(&els_out)],
                ));

                #[eggplant::pat_vars_catch]
                struct SelectOptPat {
                    pred: schema_dsl::Expr,
                    inputs: schema_dsl::Expr,
                    ctx: schema_dsl::Assumption,
                    t1: schema_dsl::Term,
                    t2: schema_dsl::Term,
                    if_out: schema_dsl::Expr,
                }
            },
            |ctx, pat| {
                let then_subst = insert_call::<schema_dsl::Expr>(
                    ctx,
                    "TermSubst",
                    &[
                        pat.ctx.to_value(ctx).val,
                        pat.inputs.to_value(ctx).val,
                        pat.t1.to_value(ctx).val,
                    ],
                );
                let else_subst = insert_call::<schema_dsl::Expr>(
                    ctx,
                    "TermSubst",
                    &[
                        pat.ctx.to_value(ctx).val,
                        pat.inputs.to_value(ctx).val,
                        pat.t2.to_value(ctx).val,
                    ],
                );
                let op = insert_call::<schema_dsl::TernaryOp>(ctx, "Select", &[]);
                let select = insert_call::<schema_dsl::Expr>(
                    ctx,
                    "Top",
                    &[
                        op.0.val,
                        pat.pred.to_value(ctx).val,
                        then_subst.to_value(ctx).val,
                        else_subst.to_value(ctx).val,
                    ],
                );
                ctx.union(pat.if_out, select);
            },
        );

        ruleset
    }
}
