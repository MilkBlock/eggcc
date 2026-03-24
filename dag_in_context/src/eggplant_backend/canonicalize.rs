pub(crate) fn fragment() -> String {
    let mut out = String::new();
    out.push_str(GENERATED_MARKER);
    out.push_str(CANONICALIZE);
    out.push('\n');
    out
}

#[cfg(feature = "eggplant")]
pub(crate) fn native_fragment() -> String {
    let mut out = String::new();
    out.push_str(GENERATED_MARKER);
    out.push_str(CANONICALIZE_SUPPORT);
    out.push('\n');
    out
}

const GENERATED_MARKER: &str =
    "; (Generated from eggplant Rust: src/eggplant_backend/canonicalize.rs)\n";
const CANONICALIZE: &str = r#"(ruleset canon)

; Commutativity
(rewrite (Bop (Add) x y) (Bop (Add) y x) :ruleset canon)
(rewrite (Bop (Mul) x y) (Bop (Mul) y x) :ruleset canon)
(rewrite (Bop (Eq) x y) (Bop (Eq) y x) :ruleset canon)
(rewrite (Bop (And) x y) (Bop (And) y x) :ruleset canon)
(rewrite (Bop (Or) x y) (Bop (Or) y x) :ruleset canon)

; Canonicalize to <
; x > y ==> y < x
(rewrite (Bop (GreaterThan) x y) (Bop (LessThan) y x) :ruleset canon)

; x >= y ==> y < x + 1
; x >= y ==> y - 1 < x
(rule (
        (= lhs (Bop (GreaterEq) x y))
        (HasArgType x ty)
        (ContextOf lhs ctx)
      )
      (
        (union lhs (Bop (LessThan) y (Bop (Add) x (Const (Int 1) ty ctx))))
        (union lhs (Bop (LessThan) (Bop (Sub) y (Const (Int 1) ty ctx)) x))
      )
      :ruleset canon)

; x <= y ==> x < y + 1
; x <= y ==> x - 1 < y
(rule (
        (= lhs (Bop (LessEq) x y))
        (HasArgType y ty)
        (ContextOf lhs ctx)
      )
      (
        (union lhs (Bop (LessThan) x (Bop (Add) y (Const (Int 1) ty ctx))))
        (union lhs (Bop (LessThan) (Bop (Sub) x (Const (Int 1) ty ctx)) y))
      )
      :ruleset canon)


; Make Concats right-deep
(rewrite (Concat (Concat a b) c)
         (Concat a (Concat b c))
         :ruleset always-run)
; Simplify Concat's with empty
(rewrite (Concat (Empty ty ctx) x)
         x
         :ruleset always-run)
(rewrite (Concat x (Empty ty ctx))
         x
         :ruleset always-run)

; Make a tuple that is a sub-range of another tuple
;                   tuple start len
(constructor SubTuple (Expr  i64   i64) Expr :unextractable)

(rewrite (SubTuple expr x 0)
         (Empty ty ctx)
         :when ((HasArgType expr ty) (ContextOf expr ctx))
         :ruleset always-run)

(rewrite (SubTuple expr x 1)
         (Single (Get expr x))
         :ruleset always-run)

(rewrite (SubTuple expr a b)
         (Concat (Single (Get expr a)) (SubTuple expr (+ a 1) (- b 1)))
         :when ((> b 1))
         :ruleset always-run)

; Some of our rules (like ivt.egg) match on `Concat`.
; These may be missing if a tuple is used directly (i.e. (DoWhile inputs (If pred thn else))).
; So add these concats for every region in the database
(rule ((= lhs (DoWhile inputs body))
       (= size (tuple-length lhs)))
      ((union lhs (SubTuple lhs 0 size)))
      :ruleset always-run)
(rule ((= lhs (If pred inputs thn els))
       (= size (tuple-length lhs)))
      ((union lhs (SubTuple lhs 0 size)))
      :ruleset always-run)
(rule ((= lhs (Switch pred inputs bodies))
       (= size (tuple-length lhs)))
      ((union lhs (SubTuple lhs 0 size)))
      :ruleset always-run)
(rule ((= lhs (Arg ty ctx))
       (= size (tuple-length lhs)))
      ((union lhs (SubTuple lhs 0 size)))
      :ruleset always-run)

; Also figure out what existing expressions are subtuples of other things
; this helps remove concat layers
(rule ((Get expr i))
      ((union (Single (Get expr i))
              (SubTuple expr i 1)))
      :ruleset always-run)

(rewrite (Concat (SubTuple expr a b)
                 (SubTuple expr (+ a b) c))
         (SubTuple expr a (+ b c))
         :ruleset always-run)
;; a subtuple which is the entire tuple is the tuple itself
;; this removes unecessary layers of concat
(rewrite (SubTuple expr 0 len)
         expr
         :when ((= len (tuple-length expr)))
         :ruleset always-run)

; Helper functions to remove one element from a tuple or type list
;                           tuple    idx
(constructor TupleRemoveAt    (Expr     i64) Expr     :unextractable)
(rewrite (TupleRemoveAt tuple idx)
         (Concat (SubTuple tuple 0 idx)
                 (SubTuple tuple (+ idx 1) (- len (+ idx 1))))
         :when ((= len (tuple-length tuple)))
         :ruleset always-run)
(rule ((TupleRemoveAt tuple idx)
       (= len (tuple-length tuple))
       (>= idx len))
      ((panic "Index out of bounds for TupleRemoveAt")) :ruleset always-run)

(constructor TypeListRemoveAt (TypeList i64) TypeList :unextractable)
(rule ((TypeListRemoveAt (TNil) _idx))
      ((panic "Index out of bounds for TypeListRemoveAt.")) :ruleset type-helpers)
(rewrite (TypeListRemoveAt (TCons x xs) 0)
         xs
         :ruleset type-helpers)
(rewrite (TypeListRemoveAt (TCons x xs) idx)
         (TCons x (TypeListRemoveAt xs (- idx 1)))
         :when ((> idx 0))
         :ruleset type-helpers)"#;

#[cfg(feature = "eggplant")]
const CANONICALIZE_SUPPORT: &str = r#"(ruleset canon)

; Make Concats right-deep
(rewrite (Concat (Concat a b) c)
         (Concat a (Concat b c))
         :ruleset always-run)
; Simplify Concat's with empty
(rewrite (Concat (Empty ty ctx) x)
         x
         :ruleset always-run)
(rewrite (Concat x (Empty ty ctx))
         x
         :ruleset always-run)

; Make a tuple that is a sub-range of another tuple
;                   tuple start len
(constructor SubTuple (Expr  i64   i64) Expr :unextractable)

(rewrite (SubTuple expr x 0)
         (Empty ty ctx)
         :when ((HasArgType expr ty) (ContextOf expr ctx))
         :ruleset always-run)

(rewrite (SubTuple expr x 1)
         (Single (Get expr x))
         :ruleset always-run)

(rewrite (SubTuple expr a b)
         (Concat (Single (Get expr a)) (SubTuple expr (+ a 1) (- b 1)))
         :when ((> b 1))
         :ruleset always-run)

; Some of our rules (like ivt.egg) match on `Concat`.
; These may be missing if a tuple is used directly (i.e. (DoWhile inputs (If pred thn else))).
; So add these concats for every region in the database
(rule ((= lhs (DoWhile inputs body))
       (= size (tuple-length lhs)))
      ((union lhs (SubTuple lhs 0 size)))
      :ruleset always-run)
(rule ((= lhs (If pred inputs thn els))
       (= size (tuple-length lhs)))
      ((union lhs (SubTuple lhs 0 size)))
      :ruleset always-run)
(rule ((= lhs (Switch pred inputs bodies))
       (= size (tuple-length lhs)))
      ((union lhs (SubTuple lhs 0 size)))
      :ruleset always-run)
(rule ((= lhs (Arg ty ctx))
       (= size (tuple-length lhs)))
      ((union lhs (SubTuple lhs 0 size)))
      :ruleset always-run)

; Also figure out what existing expressions are subtuples of other things
; this helps remove concat layers
(rule ((Get expr i))
      ((union (Single (Get expr i))
              (SubTuple expr i 1)))
      :ruleset always-run)

(rewrite (Concat (SubTuple expr a b)
                 (SubTuple expr (+ a b) c))
         (SubTuple expr a (+ b c))
         :ruleset always-run)
;; a subtuple which is the entire tuple is the tuple itself
;; this removes unecessary layers of concat
(rewrite (SubTuple expr 0 len)
         expr
         :when ((= len (tuple-length expr)))
         :ruleset always-run)

; Helper functions to remove one element from a tuple or type list
;                           tuple    idx
(constructor TupleRemoveAt    (Expr     i64) Expr     :unextractable)
(rewrite (TupleRemoveAt tuple idx)
         (Concat (SubTuple tuple 0 idx)
                 (SubTuple tuple (+ idx 1) (- len (+ idx 1))))
         :when ((= len (tuple-length tuple)))
         :ruleset always-run)
(rule ((TupleRemoveAt tuple idx)
       (= len (tuple-length tuple))
       (>= idx len))
      ((panic "Index out of bounds for TupleRemoveAt")) :ruleset always-run)

(constructor TypeListRemoveAt (TypeList i64) TypeList :unextractable)
(rule ((TypeListRemoveAt (TNil) _idx))
      ((panic "Index out of bounds for TypeListRemoveAt.")) :ruleset type-helpers)
(rewrite (TypeListRemoveAt (TCons x xs) 0)
         xs
         :ruleset type-helpers)
(rewrite (TypeListRemoveAt (TCons x xs) idx)
         (TCons x (TypeListRemoveAt xs (- idx 1)))
         :when ((> idx 0))
         :ruleset type-helpers)"#;

#[cfg(feature = "eggplant")]
pub(crate) mod native {
    use super::super::native_rule_helpers::{insert_call, Inserted};
    use super::super::schema_dsl;
    use crate::eggplant_backend::peepholes::native::PeepholeTx;
    use crate::eggplant_backend::schema_dsl::{ConstantRuleCtx, ExprRuleCtx};
    use eggplant::prelude::{prim_fact, IntoHandleTy, PEq, PatRecSgl, RuleRunnerSgl, RuleSetId};

    fn insert_bop(
        ctx: &eggplant::wrap::RuleCtx,
        op_name: &'static str,
        lhs: eggplant::egglog::Value,
        rhs: eggplant::egglog::Value,
    ) -> Inserted<schema_dsl::Expr> {
        let op = insert_call::<schema_dsl::BinaryOp>(ctx, op_name, &[]);
        insert_call::<schema_dsl::Expr>(ctx, "Bop", &[op.0.val, lhs, rhs])
    }

    #[eggplant::pat_vars]
    struct CanonBopPat<PR: PatRecSgl> {
        x: schema_dsl::Expr,
        y: schema_dsl::Expr,
        bop: schema_dsl::Bop,
    }

    fn add_commute_pat<PR: PatRecSgl>() -> CanonBopPat<PR> {
        let x = schema_dsl::Expr::query_leaf();
        let y = schema_dsl::Expr::query_leaf();
        let bop = schema_dsl::Bop::query(&schema_dsl::Add::query(), &x, &y);

        CanonBopPat::new(x, y, bop)
    }

    fn mul_commute_pat<PR: PatRecSgl>() -> CanonBopPat<PR> {
        let x = schema_dsl::Expr::query_leaf();
        let y = schema_dsl::Expr::query_leaf();
        let bop = schema_dsl::Bop::query(&schema_dsl::Mul::query(), &x, &y);

        CanonBopPat::new(x, y, bop)
    }

    fn eq_commute_pat<PR: PatRecSgl>() -> CanonBopPat<PR> {
        let x = schema_dsl::Expr::query_leaf();
        let y = schema_dsl::Expr::query_leaf();
        let bop = schema_dsl::Bop::query(&schema_dsl::Eq::query(), &x, &y);

        CanonBopPat::new(x, y, bop)
    }

    fn and_commute_pat<PR: PatRecSgl>() -> CanonBopPat<PR> {
        let x = schema_dsl::Expr::query_leaf();
        let y = schema_dsl::Expr::query_leaf();
        let bop = schema_dsl::Bop::query(&schema_dsl::And::query(), &x, &y);

        CanonBopPat::new(x, y, bop)
    }

    fn or_commute_pat<PR: PatRecSgl>() -> CanonBopPat<PR> {
        let x = schema_dsl::Expr::query_leaf();
        let y = schema_dsl::Expr::query_leaf();
        let bop = schema_dsl::Bop::query(&schema_dsl::Or::query(), &x, &y);

        CanonBopPat::new(x, y, bop)
    }

    fn greater_than_pat<PR: PatRecSgl>() -> CanonBopPat<PR> {
        let x = schema_dsl::Expr::query_leaf();
        let y = schema_dsl::Expr::query_leaf();
        let bop = schema_dsl::Bop::query(&schema_dsl::GreaterThan::query(), &x, &y);

        CanonBopPat::new(x, y, bop)
    }

    #[eggplant::pat_vars]
    struct CanonTypedBopPat<PR: PatRecSgl> {
        expr: schema_dsl::Expr,
        x: schema_dsl::Expr,
        y: schema_dsl::Expr,
        ty: schema_dsl::Type,
        ctx: schema_dsl::Assumption,
    }

    fn greater_eq_pat<PR: PatRecSgl>() -> CanonTypedBopPat<PR> {
        let expr = schema_dsl::Expr::query_leaf();
        let x = schema_dsl::Expr::query_leaf();
        let y = schema_dsl::Expr::query_leaf();
        let ty = schema_dsl::Type::query_leaf();
        let ctx = schema_dsl::Assumption::query_leaf();
        let bop = schema_dsl::Bop::query(&schema_dsl::GreaterEq::query(), &x, &y);
        let expr_is_bop = expr.handle().eq(&bop.handle());
        let has_arg_type = prim_fact(
            "HasArgType",
            vec![x.handle().into_handle_ty(), ty.handle().into_handle_ty()],
        );
        let context_of = prim_fact(
            "ContextOf",
            vec![expr.handle().into_handle_ty(), ctx.handle().into_handle_ty()],
        );

        CanonTypedBopPat::new(expr, x, y, ty, ctx)
            .assert(expr_is_bop)
            .assert(has_arg_type)
            .assert(context_of)
    }

    fn less_eq_pat<PR: PatRecSgl>() -> CanonTypedBopPat<PR> {
        let expr = schema_dsl::Expr::query_leaf();
        let x = schema_dsl::Expr::query_leaf();
        let y = schema_dsl::Expr::query_leaf();
        let ty = schema_dsl::Type::query_leaf();
        let ctx = schema_dsl::Assumption::query_leaf();
        let bop = schema_dsl::Bop::query(&schema_dsl::LessEq::query(), &x, &y);
        let expr_is_bop = expr.handle().eq(&bop.handle());
        let has_arg_type = prim_fact(
            "HasArgType",
            vec![y.handle().into_handle_ty(), ty.handle().into_handle_ty()],
        );
        let context_of = prim_fact(
            "ContextOf",
            vec![expr.handle().into_handle_ty(), ctx.handle().into_handle_ty()],
        );

        CanonTypedBopPat::new(expr, x, y, ty, ctx)
            .assert(expr_is_bop)
            .assert(has_arg_type)
            .assert(context_of)
    }

    pub(crate) fn register_native_rules() -> RuleSetId {
        let ruleset = RuleSetId("canon");

        PeepholeTx::add_rule(
            "canonicalize_commute_add",
            ruleset,
            add_commute_pat,
            |ctx, pat| {
                ctx.union(pat.bop, insert_bop(&ctx.ctx, "Add", pat.y.val, pat.x.val));
            },
        );
        PeepholeTx::add_rule(
            "canonicalize_commute_mul",
            ruleset,
            mul_commute_pat,
            |ctx, pat| {
                ctx.union(pat.bop, insert_bop(&ctx.ctx, "Mul", pat.y.val, pat.x.val));
            },
        );
        PeepholeTx::add_rule(
            "canonicalize_commute_eq",
            ruleset,
            eq_commute_pat,
            |ctx, pat| {
                ctx.union(pat.bop, insert_bop(&ctx.ctx, "Eq", pat.y.val, pat.x.val));
            },
        );
        PeepholeTx::add_rule(
            "canonicalize_commute_and",
            ruleset,
            and_commute_pat,
            |ctx, pat| {
                ctx.union(pat.bop, insert_bop(&ctx.ctx, "And", pat.y.val, pat.x.val));
            },
        );
        PeepholeTx::add_rule(
            "canonicalize_commute_or",
            ruleset,
            or_commute_pat,
            |ctx, pat| {
                ctx.union(pat.bop, insert_bop(&ctx.ctx, "Or", pat.y.val, pat.x.val));
            },
        );
        PeepholeTx::add_rule(
            "canonicalize_greater_than",
            ruleset,
            greater_than_pat,
            |ctx, pat| {
                ctx.union(
                    pat.bop,
                    insert_bop(&ctx.ctx, "LessThan", pat.y.val, pat.x.val),
                );
            },
        );
        PeepholeTx::add_rule(
            "canonicalize_greater_eq",
            ruleset,
            greater_eq_pat,
            |ctx, pat| {
                let one = ctx.ctx.insert_int(1);
                let one_const = ctx.ctx.insert_const(one, pat.ty, pat.ctx);
                let x_plus_one = insert_bop(&ctx.ctx, "Add", pat.x.val, one_const.val);
                let y_minus_one = insert_bop(&ctx.ctx, "Sub", pat.y.val, one_const.val);

                ctx.union(
                    pat.expr,
                    insert_bop(&ctx.ctx, "LessThan", pat.y.val, x_plus_one.0.val),
                );
                ctx.union(
                    pat.expr,
                    insert_bop(&ctx.ctx, "LessThan", y_minus_one.0.val, pat.x.val),
                );
            },
        );
        PeepholeTx::add_rule("canonicalize_less_eq", ruleset, less_eq_pat, |ctx, pat| {
            let one = ctx.ctx.insert_int(1);
            let one_const = ctx.ctx.insert_const(one, pat.ty, pat.ctx);
            let y_plus_one = insert_bop(&ctx.ctx, "Add", pat.y.val, one_const.val);
            let x_minus_one = insert_bop(&ctx.ctx, "Sub", pat.x.val, one_const.val);

            ctx.union(
                pat.expr,
                insert_bop(&ctx.ctx, "LessThan", pat.x.val, y_plus_one.0.val),
            );
            ctx.union(
                pat.expr,
                insert_bop(&ctx.ctx, "LessThan", x_minus_one.0.val, pat.y.val),
            );
        });

        ruleset
    }
}

#[cfg(all(test, feature = "eggplant"))]
mod native_tests {
    use crate::ast::*;
    use crate::eggplant_backend::test_lock;

    fn candidate_parts() -> (String, String, String, String, String) {
        let input_ty = tuplet!(intt());
        let lhs = getat(0);
        let rhs = int(5);
        let one = int(1);
        let gt_expr = greater_than(lhs.clone(), rhs.clone())
            .with_arg_types(input_ty.clone(), base(boolt()))
            .add_ctx(infunc("RLCR"))
            .0
            .to_string();
        let gt_expected = less_than(rhs.clone(), lhs.clone())
            .with_arg_types(input_ty.clone(), base(boolt()))
            .add_ctx(infunc("RLCR"))
            .0
            .to_string();
        let ge_expr = greater_eq(lhs.clone(), rhs.clone())
            .with_arg_types(input_ty.clone(), base(boolt()))
            .add_ctx(infunc("RLCR"))
            .0
            .to_string();
        let ge_expected_a = less_than(
            rhs.clone(),
            add(lhs.clone(), one.clone()).with_arg_types(input_ty.clone(), base(intt())),
        )
        .with_arg_types(input_ty.clone(), base(boolt()))
        .add_ctx(infunc("RLCR"))
        .0
        .to_string();
        let ge_expected_b = less_than(
            sub(rhs, one).with_arg_types(input_ty.clone(), base(intt())),
            lhs,
        )
        .with_arg_types(input_ty, base(boolt()))
        .add_ctx(infunc("RLCR"))
        .0
        .to_string();

        (gt_expr, gt_expected, ge_expr, ge_expected_a, ge_expected_b)
    }

    fn canonicalize_schedule() -> String {
        let types = crate::schedule::types_and_indexing();
        format!("(run-schedule {types})\n(run-schedule (saturate canon))\n")
    }

    fn text_canonicalize_holds(
        prologue: &str,
        gt_expr: &str,
        gt_expected: &str,
        ge_expr: &str,
        ge_expected_a: &str,
        ge_expected_b: &str,
        schedule: &str,
    ) {
        let program = format!(
            "{prologue}\n(let __rlcr_gt {gt_expr})\n(let __rlcr_ge {ge_expr})\n{schedule}(check (= __rlcr_gt {gt_expected}))\n(check (= __rlcr_ge {ge_expected_a}))\n(check (= __rlcr_ge {ge_expected_b}))\n"
        );
        let mut egraph = egglog::EGraph::default();
        egraph.parse_and_run_program(None, &program).unwrap();
    }

    fn native_canonicalize_holds(
        prologue: &str,
        gt_expr: &str,
        gt_expected: &str,
        ge_expr: &str,
        ge_expected_a: &str,
        ge_expected_b: &str,
        schedule: &str,
        ablate: Option<&str>,
    ) -> std::result::Result<(), eggplant::egglog::Error> {
        let initialization = format!("(let __rlcr_gt {gt_expr})\n(let __rlcr_ge {ge_expr})");
        crate::with_native_rules_egraph(prologue, &initialization, schedule, ablate, |egraph| {
            egraph.parse_and_run_program(
                None,
                &format!(
                    "(check (= __rlcr_gt {gt_expected}))\n(check (= __rlcr_ge {ge_expected_a}))\n(check (= __rlcr_ge {ge_expected_b}))"
                ),
            )?;
            Ok(())
        })
    }

    #[test]
    fn native_feature_path_matches_text_backend_for_canonicalize_case() {
        let _guard = test_lock::lock();
        let (gt_expr, gt_expected, ge_expr, ge_expected_a, ge_expected_b) = candidate_parts();
        let schedule = canonicalize_schedule();

        text_canonicalize_holds(
            &crate::prologue_egglog_text(),
            &gt_expr,
            &gt_expected,
            &ge_expr,
            &ge_expected_a,
            &ge_expected_b,
            &schedule,
        );
        native_canonicalize_holds(
            &crate::feature_execution_prologue(true, None),
            &gt_expr,
            &gt_expected,
            &ge_expr,
            &ge_expected_a,
            &ge_expected_b,
            &schedule,
            None,
        )
        .unwrap();

        let ablated = native_canonicalize_holds(
            &crate::feature_execution_prologue(true, Some("canon")),
            &gt_expr,
            &gt_expected,
            &ge_expr,
            &ge_expected_a,
            &ge_expected_b,
            &crate::ablate_schedule(&schedule, "canon"),
            Some("canon"),
        );

        assert!(
            ablated.is_err(),
            "ablating canon should make the canonicalize witness fail",
        );
    }
}
