pub(crate) fn fragment() -> String {
    let mut out = String::new();
    out.push_str(GENERATED_MARKER);
    out.push_str(PEEPHOLES);
    out.push('\n');
    out
}

const GENERATED_MARKER: &str =
    "; (Generated from eggplant Rust: src/eggplant_backend/peepholes.rs)\n";
const PEEPHOLES: &str = r#"; Simple rewrites that don't do a ton with control flow.

(ruleset peepholes)

(rewrite (Bop (Mul) (Const (Int 0) ty ctx) e) (Const (Int 0) ty ctx) :ruleset peepholes)
(rewrite (Bop (Mul) e (Const (Int 0) ty ctx)) (Const (Int 0) ty ctx) :ruleset peepholes)
(rewrite (Bop (Mul) (Const (Int 1) ty ctx) e) e :ruleset peepholes)
(rewrite (Bop (Mul) e (Const (Int 1) ty ctx)) e :ruleset peepholes)
(rewrite (Bop (Add) (Const (Int 0) ty ctx) e) e :ruleset peepholes)
(rewrite (Bop (Add) e (Const (Int 0) ty ctx) ) e :ruleset peepholes)

(rewrite (Bop (Mul) (Const (Int j) ty ctx) (Const (Int i) ty ctx)) (Const (Int (* i j)) ty ctx) :ruleset peepholes)
(rewrite (Bop (Add) (Const (Int j) ty ctx) (Const (Int i) ty ctx)) (Const (Int (+ i j)) ty ctx) :ruleset peepholes)

(rewrite (Bop (And) (Const (Bool true) ty ctx) e) e :ruleset peepholes)
(rewrite (Bop (And) e (Const (Bool true) ty ctx)) e :ruleset peepholes)
(rewrite (Bop (And) (Const (Bool false) ty ctx) e) (Const (Bool false) ty ctx) :ruleset peepholes)
(rewrite (Bop (And) e (Const (Bool false) ty ctx)) (Const (Bool false) ty ctx) :ruleset peepholes)
(rewrite (Bop (Or) (Const (Bool false) ty ctx) e) e :ruleset peepholes)
(rewrite (Bop (Or) e (Const (Bool false) ty ctx)) e :ruleset peepholes)
(rewrite (Bop (Or) (Const (Bool true) ty ctx) e) (Const (Bool true) ty ctx) :ruleset peepholes)
(rewrite (Bop (Or) e (Const (Bool true) ty ctx)) (Const (Bool true) ty ctx) :ruleset peepholes)

(rule (
        (= expr (Bop (Sub) x x))
        (HasArgType expr ty)
        (ContextOf expr ctx)
      )
      ((union expr (Const (Int 0) ty ctx)))
      :ruleset peepholes)

; (x - y) + z => x + (z - y)
(rewrite (Bop (Add) (Bop (Sub) x y) z) (Bop (Add) x (Bop (Sub) z y)) :ruleset peepholes)

; (a + b) - c => a + (b - c)
(rewrite (Bop (Sub) (Bop (Add) a b) c) (Bop (Add) a (Bop (Sub) b c)) :ruleset peepholes)

; (a * x) + a => a * (x + 1)
(rule (
        (= expr (Bop (Add) (Bop (Mul) a x) a))
        (HasArgType expr ty)
        (ContextOf expr ctx)
      )
      ((union expr (Bop (Mul) a (Bop (Add) x (Const (Int 1) ty ctx)))))
      :ruleset peepholes)

(rewrite (Top (Select) pred x x) x :ruleset peepholes)

; constant fold `(x + const1) + const2` even when x is not constant
(rewrite (Bop (Add) (Bop (Add) x (Const (Int i) ty ctx)) (Const (Int j) ty ctx))
         (Bop (Add) x (Const (Int (+ i j)) ty ctx))
         :ruleset peepholes)

; ptradd(ptradd(p, x), y) => ptradd(p, x + y)
(rewrite (Bop (PtrAdd) (Bop (PtrAdd) p x) y)
         (Bop (PtrAdd) p (Bop (Add) x y))
         :ruleset peepholes)"#;

#[cfg(feature = "eggplant")]
pub(crate) mod native {
    use super::super::schema_dsl;
    use crate::eggplant_backend::schema_dsl::{ConstantRuleCtx, ExprRuleCtx};
    use eggplant::{prelude::*, tx_rx_vt_pr};

    tx_rx_vt_pr!(PeepholeTx, PeepholePatRec);

    #[allow(dead_code)]
    pub(crate) fn register_native_rules(ruleset_name: &'static str) -> RuleSetId {
        let ruleset = PeepholeTx::new_ruleset(ruleset_name);

        PeepholeTx::add_rule(
            "add_zero_lhs",
            ruleset,
            || {
                let x = schema_dsl::Expr::query_leaf();
                let ty = schema_dsl::Type::query_leaf();
                let ctx = schema_dsl::Assumption::query_leaf();
                let z = schema_dsl::Int::query().value(&0);
                let zero = schema_dsl::Const::query(&z, &ty, &ctx);
                let add_op = schema_dsl::Add::query();
                let add = schema_dsl::Bop::query(&add_op, &zero, &x);
                #[eggplant::pat_vars_catch]
                struct AddZeroLhsPat {
                    x: schema_dsl::Expr,
                    z: schema_dsl::Int,
                    add: schema_dsl::Bop,
                }
            },
            |ctx, pat| {
                let _ = ctx.devalue(pat.z.value);
                ctx.union(pat.add, pat.x);
            },
        );

        PeepholeTx::add_rule(
            "add_zero_rhs",
            ruleset,
            || {
                let x = schema_dsl::Expr::query_leaf();
                let ty = schema_dsl::Type::query_leaf();
                let ctx = schema_dsl::Assumption::query_leaf();
                let z = schema_dsl::Int::query().value(&0);
                let zero = schema_dsl::Const::query(&z, &ty, &ctx);
                let add_op = schema_dsl::Add::query();
                let add = schema_dsl::Bop::query(&add_op, &x, &zero);
                #[eggplant::pat_vars_catch]
                struct AddZeroRhsPat {
                    x: schema_dsl::Expr,
                    z: schema_dsl::Int,
                    add: schema_dsl::Bop,
                }
            },
            |ctx, pat| {
                let _ = ctx.devalue(pat.z.value);
                ctx.union(pat.add, pat.x);
            },
        );

        PeepholeTx::add_rule(
            "mul_one_rhs",
            ruleset,
            || {
                let x = schema_dsl::Expr::query_leaf();
                let ty = schema_dsl::Type::query_leaf();
                let ctx = schema_dsl::Assumption::query_leaf();
                let one = schema_dsl::Int::query().value(&1);
                let one_const = schema_dsl::Const::query(&one, &ty, &ctx);
                let mul_op = schema_dsl::Mul::query();
                let mul = schema_dsl::Bop::query(&mul_op, &x, &one_const);
                #[eggplant::pat_vars_catch]
                struct MulOneRhsPat {
                    x: schema_dsl::Expr,
                    one: schema_dsl::Int,
                    mul: schema_dsl::Bop,
                }
            },
            |ctx, pat| {
                let _ = ctx.devalue(pat.one.value);
                ctx.union(pat.mul, pat.x);
            },
        );

        PeepholeTx::add_rule(
            "const_fold_add",
            ruleset,
            || {
                let ty = schema_dsl::Type::query_leaf();
                let ctx = schema_dsl::Assumption::query_leaf();
                let lhs_int = schema_dsl::Int::query();
                let rhs_int = schema_dsl::Int::query();
                let lhs = schema_dsl::Const::query(&lhs_int, &ty, &ctx);
                let rhs = schema_dsl::Const::query(&rhs_int, &ty, &ctx);
                let add_op = schema_dsl::Add::query();
                let add = schema_dsl::Bop::query(&add_op, &lhs, &rhs);
                #[eggplant::pat_vars_catch]
                struct ConstFoldAddPat {
                    ty: schema_dsl::Type,
                    ctx: schema_dsl::Assumption,
                    lhs_int: schema_dsl::Int,
                    rhs_int: schema_dsl::Int,
                    add: schema_dsl::Bop,
                }
            },
            |ctx, pat| {
                let sum = ctx.devalue(pat.lhs_int.value) + ctx.devalue(pat.rhs_int.value);
                let folded = ctx.insert_int(sum);
                let folded = ctx.insert_const(folded, pat.ty, pat.ctx);
                ctx.union(pat.add, folded);
            },
        );

        PeepholeTx::add_rule(
            "select_same_arms",
            ruleset,
            || {
                let pred = schema_dsl::Expr::query_leaf();
                let x = schema_dsl::Expr::query_leaf();
                let select_op = schema_dsl::Select::query();
                let select = schema_dsl::Top::query(&select_op, &pred, &x, &x);
                #[eggplant::pat_vars_catch]
                struct SelectSamePat {
                    x: schema_dsl::Expr,
                    select: schema_dsl::Top,
                }
            },
            |ctx, pat| {
                ctx.union(pat.select, pat.x);
            },
        );

        ruleset
    }
}

#[cfg(all(test, feature = "eggplant"))]
mod native_tests {
    use super::native::{register_native_rules, PeepholeTx};
    use crate::ast;
    use crate::eggplant_backend::schema_dsl;
    use egglog::{ast::Expr as EgglogExpr, EGraph, TermDag};
    use eggplant::prelude::{Commit, RuleRunnerSgl, RunConfig, TxSgl};

    fn eval_and_extract_expr(prologue: &str, expr: &str, schedule: &str) -> String {
        let binding = "__typed_peepholes_expr";
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

    fn eval_and_extract_native_feature_expr(
        prologue: &str,
        expr: &str,
        schedule: &str,
        ablate: Option<&str>,
    ) -> String {
        let binding = "__feature_native_expr";
        let initialization = format!("(let {binding} {expr})");
        let egraph = crate::run_egglog_program_with_native_rules(
            prologue,
            &initialization,
            schedule,
            ablate,
        )
        .unwrap();

        let mut termdag = TermDag::default();
        let (sort, value) = egraph
            .eval_expr(&EgglogExpr::Var(egglog::ast::Span::Panic, binding.into()))
            .unwrap();
        let (_, extracted) = egraph.extract(value, &mut termdag, &sort).unwrap();
        termdag.to_string(&extracted)
    }

    fn dummy_ctx() -> schema_dsl::Assumption<PeepholeTx, schema_dsl::InFuncTy> {
        schema_dsl::InFunc::new("DUMMY".to_string())
    }

    fn int_type() -> schema_dsl::Type<PeepholeTx, schema_dsl::BaseTy> {
        schema_dsl::Base::new(&schema_dsl::IntT::new())
    }

    fn int_const(value: i64) -> schema_dsl::Expr<PeepholeTx, schema_dsl::ConstTy> {
        schema_dsl::Const::new(&schema_dsl::Int::new(value), &int_type(), &dummy_ctx())
    }

    #[test]
    fn native_peepholes_align_with_text_backend_arith_case() {
        let ruleset = register_native_rules("native_peepholes_round1_arith");

        let x_tuple_ty = ast::tuplet_vec(vec![ast::intt(), ast::intt(), ast::statet()]);
        let zero = ast::int_ty(0, x_tuple_ty.clone());
        let one = ast::int_ty(1, x_tuple_ty.clone());
        let two = ast::int_ty(2, x_tuple_ty.clone());
        let three = ast::int_ty(3, x_tuple_ty.clone());
        let x = ast::get(ast::arg_ty(x_tuple_ty.clone()), 0);
        let y = ast::get(ast::arg_ty(x_tuple_ty.clone()), 1);
        let text_expr = ast::add(
            ast::add(ast::add(zero.clone(), x.clone()), zero.clone()),
            ast::add(
                ast::add(one.clone(), two.clone()),
                ast::mul(y.clone(), one.clone()),
            ),
        );
        let text_expected = ast::add(x.clone(), ast::add(three.clone(), y.clone()));
        let text_schedule = format!(
            "(run-schedule\n{}\npeepholes\n{})",
            crate::schedule::helpers(),
            crate::schedule::helpers()
        );

        let text_backend_extracted = eval_and_extract_expr(
            &crate::prologue_egglog_text(),
            &text_expr.to_string(),
            &text_schedule,
        );
        let expected_backend_extracted = eval_and_extract_expr(
            &crate::prologue_egglog_text(),
            &text_expected.to_string(),
            "",
        );
        assert_eq!(
            text_backend_extracted, expected_backend_extracted,
            "text backend baseline for the arithmetic peephole case should match the expected form",
        );

        let x_node: schema_dsl::Expr<PeepholeTx, _> = schema_dsl::Opaque::new();
        let y_node: schema_dsl::Expr<PeepholeTx, _> = schema_dsl::Opaque::new();
        let arithmetic: schema_dsl::Expr<PeepholeTx, _> = schema_dsl::Bop::new(
            &schema_dsl::Add::new(),
            &schema_dsl::Bop::new(
                &schema_dsl::Add::new(),
                &schema_dsl::Bop::new(&schema_dsl::Add::new(), &int_const(0), &x_node),
                &int_const(0),
            ),
            &schema_dsl::Bop::new(
                &schema_dsl::Add::new(),
                &schema_dsl::Bop::new(&schema_dsl::Add::new(), &int_const(1), &int_const(2)),
                &schema_dsl::Bop::new(&schema_dsl::Mul::new(), &y_node, &int_const(1)),
            ),
        );
        arithmetic.commit();

        let arithmetic_expected: schema_dsl::Expr<PeepholeTx, _> = schema_dsl::Bop::new(
            &schema_dsl::Add::new(),
            &x_node,
            &schema_dsl::Bop::new(&schema_dsl::Add::new(), &int_const(3), &y_node),
        );
        arithmetic_expected.commit();

        PeepholeTx::run_ruleset(ruleset, RunConfig::Sat);

        assert_eq!(
            PeepholeTx::canonical_raw(&arithmetic),
            PeepholeTx::canonical_raw(&arithmetic_expected),
            "typed peepholes over the real schema should match the arithmetic regression shape",
        );
    }

    #[test]
    fn native_peepholes_run_without_text_prologue() {
        let ruleset = register_native_rules("native_peepholes_round1_select");

        let select_expr: schema_dsl::Expr<PeepholeTx, _> = schema_dsl::Top::new(
            &schema_dsl::Select::new(),
            &int_const(1),
            &int_const(9),
            &int_const(9),
        );
        select_expr.commit();
        let select_expected: schema_dsl::Expr<PeepholeTx, _> = int_const(9);
        select_expected.commit();

        PeepholeTx::run_ruleset(ruleset, RunConfig::Sat);

        assert_eq!(
            PeepholeTx::canonical_raw(&select_expr),
            PeepholeTx::canonical_raw(&select_expected),
            "typed select peephole should simplify without the text prologue",
        );
    }

    #[test]
    fn feature_path_uses_native_peepholes_when_text_rules_are_absent() {
        let x_tuple_ty = ast::tuplet_vec(vec![ast::intt(), ast::intt(), ast::statet()]);
        let expr = ast::add(
            ast::int_ty(0, x_tuple_ty.clone()),
            ast::get(ast::arg_ty(x_tuple_ty.clone()), 0),
        );
        let schedule = format!(
            "(run-schedule\n{}\npeepholes\n{})",
            crate::schedule::helpers(),
            crate::schedule::helpers()
        );

        let simplified = eval_and_extract_native_feature_expr(
            &crate::native_execution_prologue(),
            &expr.to_string(),
            &schedule,
            None,
        );
        let ablated = eval_and_extract_native_feature_expr(
            &crate::native_execution_prologue(),
            &expr.to_string(),
            &crate::ablate_schedule(&schedule, "peepholes"),
            Some("peepholes"),
        );
        let expected = eval_and_extract_expr(
            &crate::prologue_egglog_text(),
            &ast::get(ast::arg_ty(x_tuple_ty), 0).to_string(),
            "",
        );

        assert_eq!(simplified, expected);
        assert_ne!(
            ablated, expected,
            "ablating the native peepholes ruleset should remove the simplification from the feature path",
        );
    }

    #[test]
    fn native_feature_path_matches_text_backend_for_fixed_input() {
        let x_tuple_ty = ast::tuplet_vec(vec![ast::intt(), ast::intt(), ast::statet()]);
        let zero = ast::int_ty(0, x_tuple_ty.clone());
        let one = ast::int_ty(1, x_tuple_ty.clone());
        let two = ast::int_ty(2, x_tuple_ty.clone());
        let x = ast::get(ast::arg_ty(x_tuple_ty.clone()), 0);
        let y = ast::get(ast::arg_ty(x_tuple_ty.clone()), 1);
        let expr = ast::add(
            ast::add(ast::add(zero.clone(), x.clone()), zero),
            ast::add(
                ast::add(one, two),
                ast::mul(y, ast::int_ty(1, x_tuple_ty.clone())),
            ),
        );
        let schedule = format!(
            "(run-schedule\n{}\npeepholes\n{})",
            crate::schedule::helpers(),
            crate::schedule::helpers()
        );

        let text_extracted =
            eval_and_extract_expr(&crate::prologue_egglog_text(), &expr.to_string(), &schedule);
        let native_extracted = eval_and_extract_native_feature_expr(
            &crate::native_execution_prologue(),
            &expr.to_string(),
            &schedule,
            None,
        );

        assert_eq!(native_extracted, text_extracted);
    }
}
