pub(crate) fn fragment() -> String {
    let mut out = String::new();
    out.push_str(GENERATED_MARKER);
    out.push_str(include_str!("../utility/expr_size.egg"));
    out.push('\n');
    out
}

#[cfg(feature = "eggplant")]
pub(crate) fn native_fragment() -> String {
    let mut out = String::new();
    out.push_str(GENERATED_MARKER);
    out.push_str(EXPR_SIZE_DECLS);
    out.push('\n');
    out
}

const GENERATED_MARKER: &str =
    "; (Generated from eggplant Rust: src/eggplant_backend/expr_size.rs)\n";
const EXPR_SIZE_DECLS: &str = r#";; Compute the tree size of program, not dag size
(function expr_size (Expr) i64 :merge (min old new) )
(function list_expr_size (ListExpr) i64 :merge (min old new))
"#;

#[cfg(feature = "eggplant")]
pub(crate) mod native {
    #![allow(non_camel_case_types)]

    use super::super::schema_dsl;
    use super::super::schema_dsl::{Expr, ListExpr};
    use crate::eggplant_backend::peepholes::native::PeepholeTx;
    use eggplant::prelude::{
        prim_call, BaseVar, PEq, PatRecSgl, RuleRunnerSgl, RuleSetId,
    };

    #[allow(non_camel_case_types)]
    #[eggplant::func(output = i64, merge = "(min old new)")]
    pub(crate) struct expr_size {
        expr: Expr,
    }

    #[allow(non_camel_case_types)]
    #[eggplant::func(output = i64, merge = "(min old new)")]
    pub(crate) struct list_expr_size {
        list: ListExpr,
    }

    fn expr_leaf<PR: PatRecSgl>() -> schema_dsl::Expr<PR> {
        schema_dsl::Expr::query_leaf()
    }

    fn list_expr_leaf<PR: PatRecSgl>() -> schema_dsl::ListExpr<PR> {
        schema_dsl::ListExpr::query_leaf()
    }

    fn type_leaf<PR: PatRecSgl>() -> schema_dsl::Type<PR> {
        schema_dsl::Type::query_leaf()
    }

    fn assumption_leaf<PR: PatRecSgl>() -> schema_dsl::Assumption<PR> {
        schema_dsl::Assumption::query_leaf()
    }

    fn base_type_leaf<PR: PatRecSgl>() -> schema_dsl::BaseType<PR> {
        schema_dsl::BaseType::query_leaf()
    }

    fn constant_leaf<PR: PatRecSgl>() -> schema_dsl::Constant<PR> {
        schema_dsl::Constant::query_leaf()
    }

    fn expr_size_query<PR: PatRecSgl>(
        expr: &schema_dsl::Expr<PR>,
        out: &BaseVar<i64, PR>,
    ) -> impl eggplant::wrap::constraint::IntoConstraintFact {
        out.handle().eq(&expr_size::query(expr).handle())
    }

    fn list_expr_size_query<PR: PatRecSgl>(
        list: &schema_dsl::ListExpr<PR>,
        out: &BaseVar<i64, PR>,
    ) -> impl eggplant::wrap::constraint::IntoConstraintFact {
        out.handle().eq(&list_expr_size::query(list).handle())
    }

    fn ternary_op_leaf<PR: PatRecSgl>() -> schema_dsl::TernaryOp<PR> {
        schema_dsl::TernaryOp::query_leaf()
    }

    fn binary_op_leaf<PR: PatRecSgl>() -> schema_dsl::BinaryOp<PR> {
        schema_dsl::BinaryOp::query_leaf()
    }

    fn unary_op_leaf<PR: PatRecSgl>() -> schema_dsl::UnaryOp<PR> {
        schema_dsl::UnaryOp::query_leaf()
    }

    #[eggplant::pat_vars]
    struct ExprLeafPat<PR: PatRecSgl> {
        expr: schema_dsl::Expr,
    }

    #[eggplant::pat_vars]
    struct ExprUnarySizePat<PR: PatRecSgl> {
        expr: schema_dsl::Expr,
        child_size: i64,
    }

    #[eggplant::pat_vars]
    struct ExprBinarySizePat<PR: PatRecSgl> {
        expr: schema_dsl::Expr,
        lhs_size: i64,
        rhs_size: i64,
    }

    #[eggplant::pat_vars]
    struct ExprTernarySizePat<PR: PatRecSgl> {
        expr: schema_dsl::Expr,
        a_size: i64,
        b_size: i64,
        c_size: i64,
    }

    #[eggplant::pat_vars]
    struct ExprSwitchSizePat<PR: PatRecSgl> {
        expr: schema_dsl::Expr,
        pred_size: i64,
        inputs_size: i64,
        branches_size: i64,
    }

    #[eggplant::pat_vars]
    struct ExprIfSizePat<PR: PatRecSgl> {
        expr: schema_dsl::Expr,
        pred_size: i64,
        inputs_size: i64,
        then_size: i64,
        else_size: i64,
    }

    #[eggplant::pat_vars]
    struct ListLeafPat<PR: PatRecSgl> {
        list: schema_dsl::ListExpr,
    }

    #[eggplant::pat_vars]
    struct ListBinarySizePat<PR: PatRecSgl> {
        list: schema_dsl::ListExpr,
        head_size: i64,
        tail_size: i64,
    }

    fn function_size_pat<PR: PatRecSgl>() -> ExprUnarySizePat<PR> {
        let expr = expr_leaf::<PR>();
        let output = expr_leaf::<PR>();
        let child_size = BaseVar::<i64, PR>::query_named("expr_size_function_out");
        let expr_is_function = expr.handle().eq(&schema_dsl::Function::query(
            &type_leaf::<PR>(),
            &type_leaf::<PR>(),
            &output,
        )
        .handle());
        let output_size = expr_size_query(&output, &child_size);

        ExprUnarySizePat::new(expr, child_size)
            .assert(expr_is_function)
            .assert(output_size)
    }

    fn const_size_pat<PR: PatRecSgl>() -> ExprLeafPat<PR> {
        let expr = expr_leaf::<PR>();
        let expr_is_const = expr.handle().eq(&schema_dsl::Const::query(
            &constant_leaf::<PR>(),
            &type_leaf::<PR>(),
            &assumption_leaf::<PR>(),
        )
        .handle());

        ExprLeafPat::new(expr).assert(expr_is_const)
    }

    fn top_size_pat<PR: PatRecSgl>() -> ExprTernarySizePat<PR> {
        let expr = expr_leaf::<PR>();
        let a = expr_leaf::<PR>();
        let b = expr_leaf::<PR>();
        let c = expr_leaf::<PR>();
        let a_size = BaseVar::<i64, PR>::query_named("expr_size_top_a");
        let b_size = BaseVar::<i64, PR>::query_named("expr_size_top_b");
        let c_size = BaseVar::<i64, PR>::query_named("expr_size_top_c");
        let expr_is_top =
            expr.handle()
                .eq(&schema_dsl::Top::query(&ternary_op_leaf::<PR>(), &a, &b, &c).handle());
        let a_size_query = expr_size_query(&a, &a_size);
        let b_size_query = expr_size_query(&b, &b_size);
        let c_size_query = expr_size_query(&c, &c_size);

        ExprTernarySizePat::new(expr, a_size, b_size, c_size)
            .assert(expr_is_top)
            .assert(a_size_query)
            .assert(b_size_query)
            .assert(c_size_query)
    }

    fn bop_size_pat<PR: PatRecSgl>() -> ExprBinarySizePat<PR> {
        let expr = expr_leaf::<PR>();
        let lhs = expr_leaf::<PR>();
        let rhs = expr_leaf::<PR>();
        let lhs_size = BaseVar::<i64, PR>::query_named("expr_size_bop_lhs");
        let rhs_size = BaseVar::<i64, PR>::query_named("expr_size_bop_rhs");
        let expr_is_bop =
            expr.handle()
                .eq(&schema_dsl::Bop::query(&binary_op_leaf::<PR>(), &lhs, &rhs).handle());
        let lhs_size_query = expr_size_query(&lhs, &lhs_size);
        let rhs_size_query = expr_size_query(&rhs, &rhs_size);

        ExprBinarySizePat::new(expr, lhs_size, rhs_size)
            .assert(expr_is_bop)
            .assert(lhs_size_query)
            .assert(rhs_size_query)
    }

    fn uop_size_pat<PR: PatRecSgl>() -> ExprUnarySizePat<PR> {
        let expr = expr_leaf::<PR>();
        let inner = expr_leaf::<PR>();
        let child_size = BaseVar::<i64, PR>::query_named("expr_size_uop_inner");
        let expr_is_uop = expr
            .handle()
            .eq(&schema_dsl::Uop::query(&unary_op_leaf::<PR>(), &inner).handle());
        let inner_size = expr_size_query(&inner, &child_size);

        ExprUnarySizePat::new(expr, child_size)
            .assert(expr_is_uop)
            .assert(inner_size)
    }

    fn get_size_pat<PR: PatRecSgl>() -> ExprUnarySizePat<PR> {
        let expr = expr_leaf::<PR>();
        let tuple = expr_leaf::<PR>();
        let child_size = BaseVar::<i64, PR>::query_named("expr_size_get_tuple");
        let expr_is_get = expr.handle().eq(&schema_dsl::Get::query(&tuple).handle());
        let tuple_size = expr_size_query(&tuple, &child_size);

        ExprUnarySizePat::new(expr, child_size)
            .assert(expr_is_get)
            .assert(tuple_size)
    }

    fn concat_size_pat<PR: PatRecSgl>() -> ExprBinarySizePat<PR> {
        let expr = expr_leaf::<PR>();
        let lhs = expr_leaf::<PR>();
        let rhs = expr_leaf::<PR>();
        let lhs_size = BaseVar::<i64, PR>::query_named("expr_size_concat_lhs");
        let rhs_size = BaseVar::<i64, PR>::query_named("expr_size_concat_rhs");
        let expr_is_concat = expr
            .handle()
            .eq(&schema_dsl::Concat::query(&lhs, &rhs).handle());
        let lhs_size_query = expr_size_query(&lhs, &lhs_size);
        let rhs_size_query = expr_size_query(&rhs, &rhs_size);

        ExprBinarySizePat::new(expr, lhs_size, rhs_size)
            .assert(expr_is_concat)
            .assert(lhs_size_query)
            .assert(rhs_size_query)
    }

    fn single_size_pat<PR: PatRecSgl>() -> ExprUnarySizePat<PR> {
        let expr = expr_leaf::<PR>();
        let inner = expr_leaf::<PR>();
        let child_size = BaseVar::<i64, PR>::query_named("expr_size_single_inner");
        let expr_is_single = expr
            .handle()
            .eq(&schema_dsl::Single::query(&inner).handle());
        let inner_size = expr_size_query(&inner, &child_size);

        ExprUnarySizePat::new(expr, child_size)
            .assert(expr_is_single)
            .assert(inner_size)
    }

    fn switch_size_pat<PR: PatRecSgl>() -> ExprSwitchSizePat<PR> {
        let expr = expr_leaf::<PR>();
        let pred = expr_leaf::<PR>();
        let inputs = expr_leaf::<PR>();
        let branches = list_expr_leaf::<PR>();
        let pred_size = BaseVar::<i64, PR>::query_named("expr_size_switch_pred");
        let inputs_size = BaseVar::<i64, PR>::query_named("expr_size_switch_inputs");
        let branches_size = BaseVar::<i64, PR>::query_named("expr_size_switch_branches");
        let expr_is_switch = expr
            .handle()
            .eq(&schema_dsl::Switch::query(&pred, &inputs, &branches).handle());
        let pred_size_query = expr_size_query(&pred, &pred_size);
        let inputs_size_query = expr_size_query(&inputs, &inputs_size);
        let branches_size_query = list_expr_size_query(&branches, &branches_size);

        ExprSwitchSizePat::new(expr, pred_size, inputs_size, branches_size)
            .assert(expr_is_switch)
            .assert(pred_size_query)
            .assert(inputs_size_query)
            .assert(branches_size_query)
    }

    fn if_size_pat<PR: PatRecSgl>() -> ExprIfSizePat<PR> {
        let expr = expr_leaf::<PR>();
        let pred = expr_leaf::<PR>();
        let inputs = expr_leaf::<PR>();
        let then_branch = expr_leaf::<PR>();
        let else_branch = expr_leaf::<PR>();
        let pred_size = BaseVar::<i64, PR>::query_named("expr_size_if_pred");
        let inputs_size = BaseVar::<i64, PR>::query_named("expr_size_if_inputs");
        let then_size = BaseVar::<i64, PR>::query_named("expr_size_if_then");
        let else_size = BaseVar::<i64, PR>::query_named("expr_size_if_else");
        let expr_is_if =
            expr.handle()
                .eq(&schema_dsl::If::query(&pred, &inputs, &then_branch, &else_branch).handle());
        let pred_size_query = expr_size_query(&pred, &pred_size);
        let inputs_size_query = expr_size_query(&inputs, &inputs_size);
        let then_size_query = expr_size_query(&then_branch, &then_size);
        let else_size_query = expr_size_query(&else_branch, &else_size);

        ExprIfSizePat::new(expr, pred_size, inputs_size, then_size, else_size)
            .assert(expr_is_if)
            .assert(pred_size_query)
            .assert(inputs_size_query)
            .assert(then_size_query)
            .assert(else_size_query)
    }

    fn dowhile_size_pat<PR: PatRecSgl>() -> ExprBinarySizePat<PR> {
        let expr = expr_leaf::<PR>();
        let input = expr_leaf::<PR>();
        let pred_and_body = expr_leaf::<PR>();
        let lhs_size = BaseVar::<i64, PR>::query_named("expr_size_dowhile_input");
        let rhs_size = BaseVar::<i64, PR>::query_named("expr_size_dowhile_body");
        let expr_is_dowhile = expr
            .handle()
            .eq(&schema_dsl::DoWhile::query(&input, &pred_and_body).handle());
        let input_size_query = expr_size_query(&input, &lhs_size);
        let body_size_query = expr_size_query(&pred_and_body, &rhs_size);

        ExprBinarySizePat::new(expr, lhs_size, rhs_size)
            .assert(expr_is_dowhile)
            .assert(input_size_query)
            .assert(body_size_query)
    }

    fn arg_size_pat<PR: PatRecSgl>() -> ExprLeafPat<PR> {
        let expr = expr_leaf::<PR>();
        let expr_is_arg =
            expr.handle()
                .eq(&schema_dsl::Arg::query(&type_leaf::<PR>(), &assumption_leaf::<PR>()).handle());

        ExprLeafPat::new(expr).assert(expr_is_arg)
    }

    fn call_size_pat<PR: PatRecSgl>() -> ExprUnarySizePat<PR> {
        let expr = expr_leaf::<PR>();
        let arg = expr_leaf::<PR>();
        let child_size = BaseVar::<i64, PR>::query_named("expr_size_call_arg");
        let expr_is_call = expr.handle().eq(&schema_dsl::Call::query(&arg).handle());
        let arg_size_query = expr_size_query(&arg, &child_size);

        ExprUnarySizePat::new(expr, child_size)
            .assert(expr_is_call)
            .assert(arg_size_query)
    }

    fn empty_size_pat<PR: PatRecSgl>() -> ExprLeafPat<PR> {
        let expr = expr_leaf::<PR>();
        let expr_is_empty = expr.handle().eq(&schema_dsl::Empty::query(
            &type_leaf::<PR>(),
            &assumption_leaf::<PR>(),
        )
        .handle());

        ExprLeafPat::new(expr).assert(expr_is_empty)
    }

    fn cons_size_pat<PR: PatRecSgl>() -> ListBinarySizePat<PR> {
        let list = list_expr_leaf::<PR>();
        let head = expr_leaf::<PR>();
        let tail = list_expr_leaf::<PR>();
        let head_size = BaseVar::<i64, PR>::query_named("list_expr_size_head");
        let tail_size = BaseVar::<i64, PR>::query_named("list_expr_size_tail");
        let list_is_cons = list
            .handle()
            .eq(&schema_dsl::Cons::query(&head, &tail).handle());
        let head_size_query = expr_size_query(&head, &head_size);
        let tail_size_query = list_expr_size_query(&tail, &tail_size);

        ListBinarySizePat::new(list, head_size, tail_size)
            .assert(list_is_cons)
            .assert(head_size_query)
            .assert(tail_size_query)
    }

    fn nil_size_pat<PR: PatRecSgl>() -> ListLeafPat<PR> {
        let list = list_expr_leaf::<PR>();
        let list_is_nil = list
            .handle()
            .eq(&prim_call::<schema_dsl::ListExpr>("Nil", vec![]));

        ListLeafPat::new(list).assert(list_is_nil)
    }

    fn alloc_size_pat<PR: PatRecSgl>() -> ExprUnarySizePat<PR> {
        let expr = expr_leaf::<PR>();
        let amount = expr_leaf::<PR>();
        let state_edge = expr_leaf::<PR>();
        let child_size = BaseVar::<i64, PR>::query_named("expr_size_alloc_amount");
        let expr_is_alloc = expr.handle().eq(&schema_dsl::Alloc::query(
            &amount,
            &state_edge,
            &base_type_leaf::<PR>(),
        )
        .handle());
        let amount_size_query = expr_size_query(&amount, &child_size);

        ExprUnarySizePat::new(expr, child_size)
            .assert(expr_is_alloc)
            .assert(amount_size_query)
    }

    pub(crate) fn register_native_support_rules() -> RuleSetId {
        let always_run = RuleSetId("always-run");

        PeepholeTx::add_rule(
            "expr_size_function",
            always_run,
            function_size_pat,
            |ctx, pat| {
                let size = ctx.devalue(pat.child_size) + 1;
                ctx.set_expr_size(pat.expr, size);
            },
        );
        PeepholeTx::add_rule("expr_size_const", always_run, const_size_pat, |ctx, pat| {
            ctx.set_expr_size(pat.expr, 1);
        });
        PeepholeTx::add_rule("expr_size_top", always_run, top_size_pat, |ctx, pat| {
            let size =
                ctx.devalue(pat.a_size) + ctx.devalue(pat.b_size) + ctx.devalue(pat.c_size) + 1;
            ctx.set_expr_size(pat.expr, size);
        });
        PeepholeTx::add_rule("expr_size_bop", always_run, bop_size_pat, |ctx, pat| {
            let size = ctx.devalue(pat.lhs_size) + ctx.devalue(pat.rhs_size) + 1;
            ctx.set_expr_size(pat.expr, size);
        });
        PeepholeTx::add_rule("expr_size_uop", always_run, uop_size_pat, |ctx, pat| {
            let size = ctx.devalue(pat.child_size) + 1;
            ctx.set_expr_size(pat.expr, size);
        });
        PeepholeTx::add_rule("expr_size_get", always_run, get_size_pat, |ctx, pat| {
            ctx.set_expr_size(pat.expr, ctx.devalue(pat.child_size));
        });
        PeepholeTx::add_rule(
            "expr_size_concat",
            always_run,
            concat_size_pat,
            |ctx, pat| {
                let size = ctx.devalue(pat.lhs_size) + ctx.devalue(pat.rhs_size);
                ctx.set_expr_size(pat.expr, size);
            },
        );
        PeepholeTx::add_rule(
            "expr_size_single",
            always_run,
            single_size_pat,
            |ctx, pat| {
                ctx.set_expr_size(pat.expr, ctx.devalue(pat.child_size));
            },
        );
        PeepholeTx::add_rule(
            "expr_size_switch",
            always_run,
            switch_size_pat,
            |ctx, pat| {
                let size = ctx.devalue(pat.pred_size)
                    + ctx.devalue(pat.inputs_size)
                    + ctx.devalue(pat.branches_size)
                    + 1;
                ctx.set_expr_size(pat.expr, size);
            },
        );
        PeepholeTx::add_rule("expr_size_if", always_run, if_size_pat, |ctx, pat| {
            let size = ctx.devalue(pat.pred_size)
                + ctx.devalue(pat.inputs_size)
                + ctx.devalue(pat.then_size)
                + ctx.devalue(pat.else_size)
                + 1;
            ctx.set_expr_size(pat.expr, size);
        });
        PeepholeTx::add_rule(
            "expr_size_dowhile",
            always_run,
            dowhile_size_pat,
            |ctx, pat| {
                let size = ctx.devalue(pat.lhs_size) + ctx.devalue(pat.rhs_size) + 1;
                ctx.set_expr_size(pat.expr, size);
            },
        );
        PeepholeTx::add_rule("expr_size_arg", always_run, arg_size_pat, |ctx, pat| {
            ctx.set_expr_size(pat.expr, 1);
        });
        PeepholeTx::add_rule("expr_size_call", always_run, call_size_pat, |ctx, pat| {
            let size = ctx.devalue(pat.child_size) + 1;
            ctx.set_expr_size(pat.expr, size);
        });
        PeepholeTx::add_rule("expr_size_empty", always_run, empty_size_pat, |ctx, pat| {
            ctx.set_expr_size(pat.expr, 0);
        });
        PeepholeTx::add_rule(
            "list_expr_size_cons",
            always_run,
            cons_size_pat,
            |ctx, pat| {
                let size = ctx.devalue(pat.head_size) + ctx.devalue(pat.tail_size);
                ctx.set_list_expr_size(pat.list, size);
            },
        );
        PeepholeTx::add_rule(
            "list_expr_size_nil",
            always_run,
            nil_size_pat,
            |ctx, pat| {
                ctx.set_list_expr_size(pat.list, 0);
            },
        );
        PeepholeTx::add_rule("expr_size_alloc", always_run, alloc_size_pat, |ctx, pat| {
            let size = ctx.devalue(pat.child_size) + 1;
            ctx.set_expr_size(pat.expr, size);
        });

        always_run
    }
}

#[cfg(all(test, feature = "eggplant"))]
mod native_tests {
    use crate::eggplant_backend::test_lock;
    use egglog::EGraph;

    fn candidate_parts() -> (String, String) {
        let expr = "(Concat (Single (Const (Int 3) (Base (IntT)) (InFunc \"RLCR\"))) (Single (Const (Int 4) (Base (IntT)) (InFunc \"RLCR\"))))".to_string();
        let branches = "(Cons (Const (Int 9) (Base (IntT)) (InFunc \"RLCR\")) (Nil))".to_string();
        (expr, branches)
    }

    fn expr_size_schedule() -> String {
        "(run-schedule (saturate always-run))".to_string()
    }

    fn text_expr_size_holds(prologue: &str, expr: &str, branches: &str, schedule: &str) {
        let program = format!(
            "{prologue}\n(let __rlcr_expr {expr})\n(let __rlcr_branches {branches})\n{schedule}\n(check (= (expr_size __rlcr_expr) 2))\n(check (= (list_expr_size __rlcr_branches) 1))\n"
        );
        let mut egraph = EGraph::default();
        egraph.parse_and_run_program(None, &program).unwrap();
    }

    fn native_expr_size_holds(
        prologue: &str,
        expr: &str,
        branches: &str,
        schedule: &str,
        ablate: Option<&str>,
    ) -> std::result::Result<(), eggplant::egglog::Error> {
        let initialization = format!("(let __rlcr_expr {expr})\n(let __rlcr_branches {branches})");
        crate::with_native_rules_egraph(prologue, &initialization, schedule, ablate, |egraph| {
            egraph.parse_and_run_program(
                None,
                "(check (= (expr_size __rlcr_expr) 2))\n(check (= (list_expr_size __rlcr_branches) 1))",
            )?;
            Ok(())
        })
    }

    #[test]
    fn native_feature_path_matches_text_backend_for_expr_size_case() {
        let _guard = test_lock::lock();
        let (expr, branches) = candidate_parts();
        let schedule = expr_size_schedule();

        text_expr_size_holds(&crate::prologue_egglog_text(), &expr, &branches, &schedule);
        native_expr_size_holds(
            &crate::feature_execution_prologue(true, None),
            &expr,
            &branches,
            &schedule,
            None,
        )
        .unwrap();

        let ablated = native_expr_size_holds(
            &crate::feature_execution_prologue(true, Some("expr-size")),
            &expr,
            &branches,
            &schedule,
            Some("expr-size"),
        );

        assert!(
            ablated.is_err(),
            "ablating expr-size should make the expr_size witness fail",
        );
    }
}
