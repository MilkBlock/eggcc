pub(crate) fn fragment() -> String {
    let mut out = String::new();
    out.push_str(GENERATED_MARKER);
    out.push_str(NON_WEAKLY_LINEAR);
    out.push('\n');
    out
}

const GENERATED_MARKER: &str =
    "; (Generated from eggplant Rust: src/eggplant_backend/non_weakly_linear.rs)\n";
const NON_WEAKLY_LINEAR: &str = r#"(ruleset non-weakly-linear)

; Eliminate if when predicate is statically true
(rule (
  (= pred (Const (Bool true) _ty ctx))
  (= if_e (If pred inputs thn els))
) (
  (union if_e (Subst ctx inputs thn))
) :ruleset non-weakly-linear)

; Eliminate if when predicate is statically false
(rule (
  (= pred (Const (Bool false) _ty ctx))
  (= if_e (If pred inputs thn els))
) (
  (union if_e (Subst ctx inputs els))
) :ruleset non-weakly-linear)


; Make every load unioned with the input
(rule ((= load (Bop (Load) load-addr state)))
      ((union (Get load 1) state))
      :ruleset non-weakly-linear)


; Pass through of state edges for ifs, regardless of type
(rule ((= if (If pred inputs then_ else_))
       (= then-branch (Get then_ i))
       (= else-branch (Get else_ i))
       (= then-branch (Get (Arg arg_ty _then_ctx) j))
       (= else-branch (Get (Arg arg_ty _else_ctx) j)))
      ((union (Get if i) (Get inputs j)))
      :ruleset non-weakly-linear)

;; peel a loop once 
(rule
 ((= lhs (DoWhile inputs outputs))
  (ContextOf lhs ctx)
  (HasType inputs inputs-ty)
  (= outputs-len (tuple-length outputs))
  (= old_cost (loop_num_iters_guess inputs outputs))
  (<= old_cost 5)
  )
 (
  (let executed-once
    (Subst ctx inputs outputs))
  (let executed-once-body
     (SubTuple executed-once 1 (- outputs-len 1)))
  (let then-ctx
    (InIf true (Get executed-once 0) executed-once-body))
  (let else-ctx
    (InIf false (Get executed-once 0) executed-once-body))

  (let new-loop-input
    (Arg inputs-ty then-ctx))
  (let new-loop-body
    (Subst (TmpCtx) (Arg inputs-ty (TmpCtx)) outputs))
  (union (InLoop new-loop-input new-loop-body) (TmpCtx))
  (delete (TmpCtx))

  (union lhs
    ;; check if we need to continue executing the loop
    (If (Get executed-once 0)
      executed-once-body ;; inputs are the body executed once
      (DoWhile new-loop-input new-loop-body)
      (Arg inputs-ty else-ctx)))

  (set (loop_num_iters_guess new-loop-input new-loop-body) (- old_cost 1))
  )
 :ruleset non-weakly-linear)"#;

#[cfg(feature = "eggplant")]
pub(crate) mod native {
    use super::super::schema_dsl;
    use crate::eggplant_backend::loop_invariant::native::loop_num_iters_guessRuleCtx;
    use crate::eggplant_backend::peepholes::native::PeepholeTx;
    use eggplant::prelude::{AsHandle, Insertable, PEq, PatRecSgl, RuleRunnerSgl, RuleSetId};

    #[eggplant::pat_vars]
    struct IfConstPat<PR: PatRecSgl> {
        if_e: schema_dsl::If,
        ctx: schema_dsl::Assumption,
        inputs: schema_dsl::Expr,
        branch: schema_dsl::Expr,
        pred_ty: schema_dsl::Type,
    }

    fn if_const_pat<PR: PatRecSgl>(pred_value: bool) -> IfConstPat<PR> {
        let pred_ty = schema_dsl::Type::query_leaf();
        let ctx = schema_dsl::Assumption::query_leaf();
        let pred_bool = schema_dsl::Bool::query();
        let pred_matches_value = pred_bool.handle_value().eq(&pred_value);
        let pred = schema_dsl::Const::query(&pred_bool, &pred_ty, &ctx);
        let inputs = schema_dsl::Expr::query_leaf();
        let thn = schema_dsl::Expr::query_leaf();
        let els = schema_dsl::Expr::query_leaf();
        let if_e = schema_dsl::If::query(&pred, &inputs, &thn, &els);
        let branch = if pred_value { thn } else { els };

        IfConstPat::new(if_e, ctx, inputs, branch, pred_ty).assert(pred_matches_value)
    }

    #[eggplant::pat_vars]
    struct LoadStatePat<PR: PatRecSgl> {
        state: schema_dsl::Expr,
        load_out: schema_dsl::Get,
    }

    fn load_state_pat<PR: PatRecSgl>() -> LoadStatePat<PR> {
        let load_addr = schema_dsl::Expr::query_leaf();
        let state = schema_dsl::Expr::query_leaf();
        let load = schema_dsl::Bop::query(&schema_dsl::Load::query(), &load_addr, &state);
        let load_out = schema_dsl::Get::query(&load);
        let output_is_state = load_out.handle_index().eq(&(&1_i64).as_handle());

        LoadStatePat::new(state, load_out).assert(output_is_state)
    }

    #[eggplant::pat_vars]
    struct IfPassthroughPat<PR: PatRecSgl> {
        inputs: schema_dsl::Expr,
        then_arg_out: schema_dsl::Get,
        lhs: schema_dsl::Get,
        arg_ty: schema_dsl::Type,
    }

    fn if_passthrough_pat<PR: PatRecSgl>() -> IfPassthroughPat<PR> {
        let pred = schema_dsl::Expr::query_leaf();
        let inputs = schema_dsl::Expr::query_leaf();
        let then_ = schema_dsl::Expr::query_leaf();
        let else_ = schema_dsl::Expr::query_leaf();
        let if_expr = schema_dsl::If::query(&pred, &inputs, &then_, &else_);
        let lhs = schema_dsl::Get::query(&if_expr);
        let then_branch = schema_dsl::Get::query(&then_);
        let else_branch = schema_dsl::Get::query(&else_);
        let arg_ty = schema_dsl::Type::query_leaf();
        let then_ctx = schema_dsl::Assumption::query_leaf();
        let else_ctx = schema_dsl::Assumption::query_leaf();
        let then_arg = schema_dsl::Arg::query(&arg_ty, &then_ctx);
        let else_arg = schema_dsl::Arg::query(&arg_ty, &else_ctx);
        let then_arg_out = schema_dsl::Get::query(&then_arg);
        let else_arg_out = schema_dsl::Get::query(&else_arg);

        let same_then_index = then_branch.handle_index().eq(&lhs.handle_index());
        let same_else_index = else_branch.handle_index().eq(&lhs.handle_index());
        let same_arg_index = then_arg_out.handle_index().eq(&else_arg_out.handle_index());
        let same_then_value = then_branch.handle().eq(&then_arg_out.handle());
        let same_else_value = else_branch.handle().eq(&else_arg_out.handle());

        IfPassthroughPat::new(inputs, then_arg_out, lhs, arg_ty)
            .assert(same_then_index)
            .assert(same_else_index)
            .assert(same_arg_index)
            .assert(same_then_value)
            .assert(same_else_value)
    }

    #[eggplant::pat_vars]
    struct LoopPeelPat<PR: PatRecSgl> {
        lhs: schema_dsl::DoWhile,
        inputs: schema_dsl::Expr,
        outputs: schema_dsl::Expr,
        ctx: schema_dsl::Assumption,
        inputs_ty: schema_dsl::Type,
        loop_context: schema_dsl::ContextOf,
        inputs_have_type: schema_dsl::HasType,
    }

    fn loop_peel_pat<PR: PatRecSgl>() -> LoopPeelPat<PR> {
        let inputs = schema_dsl::Expr::query_leaf();
        let outputs = schema_dsl::Expr::query_leaf();
        let lhs = schema_dsl::DoWhile::query(&inputs, &outputs);
        let ctx = schema_dsl::Assumption::query_leaf();
        let inputs_ty = schema_dsl::Type::query_leaf();
        let loop_context = schema_dsl::ContextOf::query();
        let inputs_have_type = schema_dsl::HasType::query();
        let loop_context_matches_lhs = loop_context.expr.handle().eq(&lhs.handle());
        let loop_context_matches_ctx = loop_context.ctx.handle().eq(&ctx.handle());
        let inputs_have_type_matches_inputs = inputs_have_type.expr.handle().eq(&inputs.handle());
        let inputs_have_type_matches_ty = inputs_have_type.ty.handle().eq(&inputs_ty.handle());

        LoopPeelPat::new(
            lhs,
            inputs,
            outputs,
            ctx,
            inputs_ty,
            loop_context,
            inputs_have_type,
        )
        .assert(loop_context_matches_lhs)
        .assert(loop_context_matches_ctx)
        .assert(inputs_have_type_matches_inputs)
        .assert(inputs_have_type_matches_ty)
    }

    pub(crate) fn register_native_rules() -> RuleSetId {
        let ruleset = PeepholeTx::new_ruleset("non-weakly-linear");

        PeepholeTx::add_rule(
            "non_weakly_linear_if_true",
            ruleset,
            || {
                if_const_pat(true)
            },
            |ctx, pat| {
                let rewritten = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "Subst",
                    &[
                        pat.ctx.to_value(&ctx).val,
                        pat.inputs.to_value(&ctx).val,
                        pat.branch.to_value(&ctx).val,
                    ],
                ));
                ctx.union(pat.if_e, rewritten);
            },
        );

        PeepholeTx::add_rule(
            "non_weakly_linear_if_false",
            ruleset,
            || {
                if_const_pat(false)
            },
            |ctx, pat| {
                let rewritten = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "Subst",
                    &[
                        pat.ctx.to_value(&ctx).val,
                        pat.inputs.to_value(&ctx).val,
                        pat.branch.to_value(&ctx).val,
                    ],
                ));
                ctx.union(pat.if_e, rewritten);
            },
        );

        PeepholeTx::add_rule(
            "non_weakly_linear_load_state",
            ruleset,
            load_state_pat,
            |ctx, pat| {
                ctx.union(pat.load_out, pat.state);
            },
        );

        PeepholeTx::add_rule(
            "non_weakly_linear_if_passthrough",
            ruleset,
            if_passthrough_pat,
            |ctx, pat| {
                let passthrough = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "Get",
                    &[pat.inputs.to_value(&ctx).val, pat.then_arg_out.index.val],
                ));
                ctx.union(pat.lhs, passthrough);
            },
        );

        PeepholeTx::add_rule(
            "non_weakly_linear_loop_peel_once",
            ruleset,
            loop_peel_pat,
            |ctx, pat| {
                let Some(old_cost) = ctx.try_read_loop_num_iters_guess(pat.inputs, pat.outputs) else {
                    return;
                };
                if old_cost > 5 {
                    return;
                }

                let zero = ctx._intern_base::<i64, i64>(0);
                let one = ctx._intern_base::<i64, i64>(1);
                let outputs_len_value =
                    ctx.lookup_expect("tuple-length", &[pat.outputs.to_value(&ctx).val]);
                let outputs_len: i64 = ctx._devalue_base(outputs_len_value);
                let outputs_body_len = ctx._intern_base::<i64, i64>(outputs_len - 1);

                let executed_once = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "Subst",
                    &[
                        pat.ctx.to_value(&ctx).val,
                        pat.inputs.to_value(&ctx).val,
                        pat.outputs.to_value(&ctx).val,
                    ],
                ));
                let executed_once_pred = eggplant::wrap::Value::<schema_dsl::Expr>::new(
                    (&ctx).insert("Get", &[executed_once.to_value(&ctx).val, zero]),
                );
                let executed_once_body =
                    eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                        "SubTuple",
                        &[executed_once.to_value(&ctx).val, one, outputs_body_len],
                    ));
                let then_ctx = eggplant::wrap::Value::<schema_dsl::Assumption>::new((&ctx).insert(
                    "InIf",
                    &[
                        true.to_value(&ctx).val,
                        executed_once_pred.to_value(&ctx).val,
                        executed_once_body.to_value(&ctx).val,
                    ],
                ));
                let else_ctx = eggplant::wrap::Value::<schema_dsl::Assumption>::new((&ctx).insert(
                    "InIf",
                    &[
                        false.to_value(&ctx).val,
                        executed_once_pred.to_value(&ctx).val,
                        executed_once_body.to_value(&ctx).val,
                    ],
                ));
                let new_loop_input = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "Arg",
                    &[
                        pat.inputs_ty.to_value(&ctx).val,
                        then_ctx.to_value(&ctx).val,
                    ],
                ));
                let tmp_ctx = eggplant::wrap::Value::<schema_dsl::Assumption>::new(
                    (ctx).insert("TmpCtx", &[]),
                );
                let tmp_arg = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "Arg",
                    &[pat.inputs_ty.to_value(&ctx).val, tmp_ctx.to_value(&ctx).val],
                ));
                let new_loop_body = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "Subst",
                    &[
                        tmp_ctx.to_value(&ctx).val,
                        tmp_arg.to_value(&ctx).val,
                        pat.outputs.to_value(&ctx).val,
                    ],
                ));
                let in_loop = eggplant::wrap::Value::<schema_dsl::Assumption>::new((&ctx).insert(
                    "InLoop",
                    &[
                        new_loop_input.to_value(&ctx).val,
                        new_loop_body.to_value(&ctx).val,
                    ],
                ));
                ctx.union(tmp_ctx, in_loop);

                let else_arg = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "Arg",
                    &[
                        pat.inputs_ty.to_value(&ctx).val,
                        else_ctx.to_value(&ctx).val,
                    ],
                ));
                let peeled_loop = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "DoWhile",
                    &[
                        new_loop_input.to_value(&ctx).val,
                        new_loop_body.to_value(&ctx).val,
                    ],
                ));
                let peeled_if = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "If",
                    &[
                        executed_once_pred.to_value(&ctx).val,
                        executed_once_body.to_value(&ctx).val,
                        peeled_loop.to_value(&ctx).val,
                        else_arg.to_value(&ctx).val,
                    ],
                ));

                ctx.union(pat.lhs, peeled_if);
                ctx.set_loop_num_iters_guess(new_loop_input, new_loop_body, old_cost - 1);
                ctx.remove("TmpCtx", &[]);
            },
        );

        ruleset
    }
}
