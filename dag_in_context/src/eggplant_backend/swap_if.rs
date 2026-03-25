pub(crate) fn fragment() -> String {
    let mut out = String::new();
    out.push_str(GENERATED_MARKER);
    out.push_str(SWAP_IF);
    out.push('\n');
    out
}

const GENERATED_MARKER: &str =
    "; (Generated from eggplant Rust: src/eggplant_backend/swap_if.rs)\n";
const SWAP_IF: &str = r#"(ruleset swap-if)

;; swaps the order of the then and else branches
;; in an if using Not

(rule
  ((= lhs (If pred inputs then else)))
  (
    (union lhs (If (Uop (Not) pred) inputs else then))
  )
  :ruleset swap-if)


;; for if statements with two outputs, swaps the order
;; of the outputs
(rule
  ((= lhs (If pred inputs then else))
   (= (tuple-length then) 2)
   (= (tuple-length else) 2))
  (
    (union
      (Concat (Single (Get lhs 1)) (Single (Get lhs 0)))
      (If pred inputs
          (Concat (Single (Get then 1)) (Single (Get then 0)))
          (Concat (Single (Get else 1)) (Single (Get else 0)))))
  )
  :ruleset swap-if)"#;

#[cfg(feature = "eggplant")]
pub(crate) mod native {
    use super::super::schema_dsl;
    use crate::eggplant_backend::peepholes::native::PeepholeTx;
    use eggplant::prelude::{AsHandle, PEq, PatRecSgl, RuleRunnerSgl, RuleSetId};
    use schema_dsl::{ExprPRRuleCtx, UnaryOpPRRuleCtx};

    #[eggplant::pat_vars]
    struct SwapIfPat<PR: PatRecSgl> {
        pred: schema_dsl::Expr,
        inputs: schema_dsl::Expr,
        then_: schema_dsl::Expr,
        else_: schema_dsl::Expr,
        if_expr: schema_dsl::If,
    }

    fn swap_if_pat<PR: PatRecSgl>() -> SwapIfPat<PR> {
        let pred = schema_dsl::Expr::query_leaf();
        let inputs = schema_dsl::Expr::query_leaf();
        let then_ = schema_dsl::Expr::query_leaf();
        let else_ = schema_dsl::Expr::query_leaf();
        let if_expr = schema_dsl::If::query(&pred, &inputs, &then_, &else_);

        SwapIfPat::new(pred, inputs, then_, else_, if_expr)
    }

    #[eggplant::pat_vars]
    struct SwapIfTuplePat<PR: PatRecSgl> {
        pred: schema_dsl::Expr,
        inputs: schema_dsl::Expr,
        then0: schema_dsl::Get,
        then1: schema_dsl::Get,
        else0: schema_dsl::Get,
        else1: schema_dsl::Get,
        lhs0: schema_dsl::Get,
        lhs1: schema_dsl::Get,
        if_expr: schema_dsl::If,
    }

    fn swap_if_tuple_pat<PR: PatRecSgl>() -> SwapIfTuplePat<PR> {
        let pred = schema_dsl::Expr::query_leaf();
        let inputs = schema_dsl::Expr::query_leaf();
        let then_ = schema_dsl::Expr::query_leaf();
        let else_ = schema_dsl::Expr::query_leaf();
        let if_expr = schema_dsl::If::query(&pred, &inputs, &then_, &else_);

        let lhs0 = schema_dsl::Get::query(&if_expr);
        let lhs1 = schema_dsl::Get::query(&if_expr);
        let then0 = schema_dsl::Get::query(&then_);
        let then1 = schema_dsl::Get::query(&then_);
        let else0 = schema_dsl::Get::query(&else_);
        let else1 = schema_dsl::Get::query(&else_);

        let lhs_canonical = schema_dsl::Concat::query(
            &schema_dsl::Single::query(&lhs0),
            &schema_dsl::Single::query(&lhs1),
        );
        let then_canonical = schema_dsl::Concat::query(
            &schema_dsl::Single::query(&then0),
            &schema_dsl::Single::query(&then1),
        );
        let else_canonical = schema_dsl::Concat::query(
            &schema_dsl::Single::query(&else0),
            &schema_dsl::Single::query(&else1),
        );

        let lhs0_is_first = lhs0.handle_index().eq(&(&0_i64).as_handle());
        let lhs1_is_second = lhs1.handle_index().eq(&(&1_i64).as_handle());
        let then0_is_first = then0.handle_index().eq(&(&0_i64).as_handle());
        let then1_is_second = then1.handle_index().eq(&(&1_i64).as_handle());
        let else0_is_first = else0.handle_index().eq(&(&0_i64).as_handle());
        let else1_is_second = else1.handle_index().eq(&(&1_i64).as_handle());
        let lhs_is_two_tuple = if_expr.handle().eq(&lhs_canonical.handle());
        let then_is_two_tuple = then_.handle().eq(&then_canonical.handle());
        let else_is_two_tuple = else_.handle().eq(&else_canonical.handle());

        SwapIfTuplePat::new(
            pred, inputs, then0, then1, else0, else1, lhs0, lhs1, if_expr,
        )
        .assert(lhs0_is_first)
        .assert(lhs1_is_second)
        .assert(then0_is_first)
        .assert(then1_is_second)
        .assert(else0_is_first)
        .assert(else1_is_second)
        .assert(lhs_is_two_tuple)
        .assert(then_is_two_tuple)
        .assert(else_is_two_tuple)
    }

    pub(crate) fn register_native_rules() -> RuleSetId {
        let ruleset = PeepholeTx::new_ruleset("swap-if");

        PeepholeTx::add_rule("swap_if_not", ruleset, swap_if_pat, |ctx, pat| {
            let not_op = ctx.insert_not();
            let inverted_pred = ctx.insert_uop(not_op, pat.pred);
            let swapped_if = ctx.insert_if(inverted_pred, pat.inputs, pat.else_, pat.then_);

            ctx.union(pat.if_expr, swapped_if);
        });

        PeepholeTx::add_rule("swap_if_tuple", ruleset, swap_if_tuple_pat, |ctx, pat| {
            let swapped_then =
                ctx.insert_concat(ctx.insert_single(pat.then1), ctx.insert_single(pat.then0));
            let swapped_else =
                ctx.insert_concat(ctx.insert_single(pat.else1), ctx.insert_single(pat.else0));
            let swapped_if = ctx.insert_if(pat.pred, pat.inputs, swapped_then, swapped_else);
            let swapped_outputs =
                ctx.insert_concat(ctx.insert_single(pat.lhs1), ctx.insert_single(pat.lhs0));

            ctx.union(swapped_outputs, swapped_if);
        });

        ruleset
    }
}
