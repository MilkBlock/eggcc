pub(crate) fn fragment() -> String {
    let mut out = String::new();
    out.push_str(GENERATED_MARKER);
    out.push_str(PASSTHROUGH_RULES);
    out.push('\n');
    out
}

#[cfg(feature = "eggplant")]
pub(crate) fn state_edge_fragment() -> String {
    String::new()
}

const GENERATED_MARKER: &str =
    "; (Generated from eggplant Rust: src/eggplant_backend/passthrough.rs)\n";
const PASSTHROUGH_RULES: &str = r#"(ruleset passthrough)


;; Pass through thetas
(rule ((= lhs (Get loop i))
        (= loop (DoWhile inputs pred-outputs))
        (= (Get pred-outputs (+ i 1)) (Get (Arg _ty _ctx) i))
        ;; only pass through pure types, since some loops don't terminate
        ;; so the state edge must pass through them
        (HasType lhs lhs_ty)
        (PureType lhs_ty)
        )
       ((union lhs (Get inputs i)))
       :ruleset passthrough)

;; Pass through switch arguments
(rule ((= lhs (Get switch i))
       (= switch (Switch pred inputs branches))
       (= (ListExpr-length branches) 2)
       (= branch0 (ListExpr-ith branches 0))
       (= branch1 (ListExpr-ith branches 1))
       (= (Get branch0 i) (Get (Arg _ _ctx0) j))
       (= (Get branch1 i) (Get (Arg _ _ctx1) j))
       (= passed-through (Get inputs j))
       (HasType lhs lhs_ty)
       (!= lhs_ty (Base (StateT))))
      ((union lhs passed-through))
      :ruleset passthrough)

;; Pass through switch predicate
(rule ((= lhs (Get switch i))
       (= switch (Switch pred inputs branches))
       (= (ListExpr-length branches) 2)
       (= branch0 (ListExpr-ith branches 0))
       (= branch1 (ListExpr-ith branches 1))
       (= (Get branch0 i) (Const (Bool false) _ _ctx0))
       (= (Get branch1 i) (Const (Bool true) _ _ctx1)))
      ((union lhs pred))
      :ruleset passthrough)

;; Pass through if arguments
(rule ((= if (If pred inputs then_ else_))
       (= then-branch (Get then_ i))
       (= else-branch (Get else_ i))
       (= then-branch (Get (Arg arg_ty _then_ctx) j))
       (= else-branch (Get (Arg arg_ty _else_ctx) j))
       (HasType then-branch lhs_ty)
       (!= lhs_ty (Base (StateT))))
      ((union (Get if i) (Get inputs j)))
      :ruleset passthrough)

; Pass through if state edge arguments
; To maintain the invariant, we have to union the other outputs with a pure if statement
(ruleset state-edge-passthrough)

(rule ((= outputs (If pred inputs then_ else_))

       (= (Get then_ i) (Get (Arg arg_ty then_ctx) j))
       (= (Get else_ i) (Get (Arg arg_ty else_ctx) j))

       (HasType (Get then_ i) (Base (StateT))))

      ((let lhs (Get outputs i))
       (let new_inputs (TupleRemoveAt inputs j))

       (let new_then_ctx (InIf true  pred new_inputs))
       (let new_else_ctx (InIf false pred new_inputs))

       (let old_then (TupleRemoveAt then_ i))
       (let old_else (TupleRemoveAt else_ i))

       (let new_then (DropAt new_then_ctx j old_then))
       (let new_else (DropAt new_else_ctx j old_else))

       (let old_outputs (TupleRemoveAt outputs i))
       (let new_if (If pred new_inputs new_then new_else))
       (union new_if old_outputs)

       (union lhs (Get inputs j))
       ;; Be careful not to subsume the original if statement immediately,
       ;;  since TupleRemoveAt still needs to match on it
       (ToSubsumeIf pred inputs then_ else_))
      :ruleset state-edge-passthrough)

;; Pass through if predicate
(rule ((= if (If pred inputs then_ else_))
       (= (Get then_ i) (Const (Bool true) _ _thenctx))
       (= (Get else_ i) (Const (Bool false) _ _elsectx)))

      ((let new_then (TupleRemoveAt then_ i))
       (let new_else (TupleRemoveAt else_ i))
       (let new_if (If pred inputs new_then new_else))

       (union (Get           if i) pred)
       (union (TupleRemoveAt if i) new_if)
       (ToSubsumeIf pred inputs then_ else_))
      :ruleset passthrough)

;; Pass through inverted if predicate
(rule ((= if (If pred inputs then_ else_))
       (= (Get then_ i) (Const (Bool false) _ _thenctx))
       (= (Get else_ i) (Const (Bool true) _ _elsectx)))

      ((let new_then (TupleRemoveAt then_ i))
       (let new_else (TupleRemoveAt else_ i))
       (let new_if (If pred inputs new_then new_else))

       (union (Get           if i) (Uop (Not) pred))
       (union (TupleRemoveAt if i) new_if)
       (ToSubsumeIf pred inputs then_ else_))
      :ruleset passthrough)"#;
#[cfg(feature = "eggplant")]
pub(crate) mod native {
    use super::super::schema_dsl;
    use crate::eggplant_backend::peepholes::native::PeepholeTx;
    use crate::eggplant_backend::schema_dsl::{
        AssumptionPRRuleCtx, ExprPRRuleCtx, ToSubsumeIfPRRuleCtx, UnaryOpPRRuleCtx,
    };
    use eggplant::prelude::{AsHandle, Insertable, PEq, PatRecSgl, RuleRunnerSgl, RuleSetId};

    fn type_leaf<PR: PatRecSgl>() -> schema_dsl::Type<PR> {
        schema_dsl::Type::query_leaf()
    }

    fn state_type<PR: PatRecSgl>() -> schema_dsl::Type<PR, schema_dsl::BaseTy> {
        schema_dsl::Base::query(&schema_dsl::StateT::query())
    }

    fn arg_expr<PR: PatRecSgl>() -> schema_dsl::Expr<PR, schema_dsl::ArgTy> {
        schema_dsl::Arg::query(&type_leaf(), &schema_dsl::Assumption::query_leaf())
    }

    fn bool_const<PR: PatRecSgl>(
        value: bool,
    ) -> (
        schema_dsl::Expr<PR, schema_dsl::ConstTy>,
        eggplant::wrap::EqConstraint<bool, bool>,
    ) {
        let bool_value = schema_dsl::Bool::query();
        let value_matches = bool_value.handle_value().eq(&value);
        let const_expr = schema_dsl::Const::query(
            &bool_value,
            &type_leaf(),
            &schema_dsl::Assumption::query_leaf(),
        );
        (const_expr, value_matches)
    }

    #[eggplant::pat_vars]
    struct LoopThetaPat<PR: PatRecSgl> {
        inputs: schema_dsl::Expr,
        lhs: schema_dsl::Get,
        has_type: schema_dsl::HasType,
        pure_type: schema_dsl::PureType,
    }

    fn loop_theta_pat<PR: PatRecSgl>() -> LoopThetaPat<PR> {
        let inputs = schema_dsl::Expr::query_leaf();
        let pred_outputs = schema_dsl::Expr::query_leaf();
        let loop_expr = schema_dsl::DoWhile::query(&inputs, &pred_outputs);
        let lhs = schema_dsl::Get::query(&loop_expr);
        let body_out = schema_dsl::Get::query(&pred_outputs);
        let arg_out = schema_dsl::Get::query(&arg_expr::<PR>());
        let lhs_ty = type_leaf::<PR>();
        let body_index_matches = body_out
            .handle_index()
            .eq(&(lhs.handle_index() + (&1_i64).as_handle()));
        let arg_index_matches = arg_out.handle_index().eq(&lhs.handle_index());
        let same_body_value = body_out.handle().eq(&arg_out.handle());
        let has_type = schema_dsl::HasType::query();
        let pure_type = schema_dsl::PureType::query();
        let same_typed_expr = has_type.expr.handle().eq(&lhs.handle());
        let same_lhs_ty = has_type.ty.handle().eq(&lhs_ty.handle());
        let pure_type_matches = pure_type.ty.handle().eq(&lhs_ty.handle());

        LoopThetaPat::new(inputs, lhs, has_type, pure_type)
            .assert(body_index_matches)
            .assert(arg_index_matches)
            .assert(same_body_value)
            .assert(same_typed_expr)
            .assert(same_lhs_ty)
            .assert(pure_type_matches)
    }

    #[eggplant::pat_vars]
    struct SwitchArgPat<PR: PatRecSgl> {
        inputs: schema_dsl::Expr,
        arg0_out: schema_dsl::Get,
        lhs: schema_dsl::Get,
        has_type: schema_dsl::HasType,
    }

    fn switch_arg_pat<PR: PatRecSgl>() -> SwitchArgPat<PR> {
        let pred = schema_dsl::Expr::query_leaf();
        let inputs = schema_dsl::Expr::query_leaf();
        let branch0 = schema_dsl::Expr::query_leaf();
        let branch1 = schema_dsl::Expr::query_leaf();
        let nil = schema_dsl::Nil::query();
        let tail = schema_dsl::Cons::query(&branch1, &nil);
        let branches = schema_dsl::Cons::query(&branch0, &tail);
        let switch = schema_dsl::Switch::query(&pred, &inputs, &branches);
        let lhs = schema_dsl::Get::query(&switch);
        let branch0_out = schema_dsl::Get::query(&branch0);
        let branch1_out = schema_dsl::Get::query(&branch1);
        let arg_ty = schema_dsl::Type::query_leaf();
        let ctx0 = schema_dsl::Assumption::query_leaf();
        let ctx1 = schema_dsl::Assumption::query_leaf();
        let arg0 = schema_dsl::Arg::query(&arg_ty, &ctx0);
        let arg1 = schema_dsl::Arg::query(&arg_ty, &ctx1);
        let arg0_out = schema_dsl::Get::query(&arg0);
        let arg1_out = schema_dsl::Get::query(&arg1);
        let lhs_ty = type_leaf::<PR>();
        let same_branch0_index = branch0_out.handle_index().eq(&lhs.handle_index());
        let same_branch1_index = branch1_out.handle_index().eq(&lhs.handle_index());
        let same_arg_index = arg0_out.handle_index().eq(&arg1_out.handle_index());
        let same_branch0_value = branch0_out.handle().eq(&arg0_out.handle());
        let same_branch1_value = branch1_out.handle().eq(&arg1_out.handle());
        let has_type = schema_dsl::HasType::query();
        let same_typed_expr = has_type.expr.handle().eq(&lhs.handle());
        let same_lhs_ty = has_type.ty.handle().eq(&lhs_ty.handle());
        let is_not_state = lhs_ty.handle().ne(&state_type::<PR>().handle());

        SwitchArgPat::new(inputs, arg0_out, lhs, has_type)
            .assert(same_branch0_index)
            .assert(same_branch1_index)
            .assert(same_arg_index)
            .assert(same_branch0_value)
            .assert(same_branch1_value)
            .assert(same_typed_expr)
            .assert(same_lhs_ty)
            .assert(is_not_state)
    }

    #[eggplant::pat_vars]
    struct SwitchPredicatePat<PR: PatRecSgl> {
        pred: schema_dsl::Expr,
        lhs: schema_dsl::Get,
    }

    fn switch_predicate_pat<PR: PatRecSgl>() -> SwitchPredicatePat<PR> {
        let pred = schema_dsl::Expr::query_leaf();
        let inputs = schema_dsl::Expr::query_leaf();
        let branch0 = schema_dsl::Expr::query_leaf();
        let branch1 = schema_dsl::Expr::query_leaf();
        let nil = schema_dsl::Nil::query();
        let tail = schema_dsl::Cons::query(&branch1, &nil);
        let branches = schema_dsl::Cons::query(&branch0, &tail);
        let switch = schema_dsl::Switch::query(&pred, &inputs, &branches);
        let lhs = schema_dsl::Get::query(&switch);
        let branch0_out = schema_dsl::Get::query(&branch0);
        let branch1_out = schema_dsl::Get::query(&branch1);
        let (false_const, false_matches_value) = bool_const::<PR>(false);
        let (true_const, true_matches_value) = bool_const::<PR>(true);
        let same_branch0_index = branch0_out.handle_index().eq(&lhs.handle_index());
        let same_branch1_index = branch1_out.handle_index().eq(&lhs.handle_index());
        let branch0_is_false = branch0_out.handle().eq(&false_const.handle());
        let branch1_is_true = branch1_out.handle().eq(&true_const.handle());

        SwitchPredicatePat::new(pred, lhs)
            .assert(same_branch0_index)
            .assert(same_branch1_index)
            .assert(branch0_is_false)
            .assert(false_matches_value)
            .assert(branch1_is_true)
            .assert(true_matches_value)
    }

    #[eggplant::pat_vars]
    struct IfArgPat<PR: PatRecSgl> {
        inputs: schema_dsl::Expr,
        then_arg_out: schema_dsl::Get,
        lhs: schema_dsl::Get,
        has_type: schema_dsl::HasType,
    }

    fn if_arg_pat<PR: PatRecSgl>() -> IfArgPat<PR> {
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
        let lhs_ty = type_leaf::<PR>();
        let same_then_index = then_branch.handle_index().eq(&lhs.handle_index());
        let same_else_index = else_branch.handle_index().eq(&lhs.handle_index());
        let same_arg_index = then_arg_out.handle_index().eq(&else_arg_out.handle_index());
        let same_then_value = then_branch.handle().eq(&then_arg_out.handle());
        let same_else_value = else_branch.handle().eq(&else_arg_out.handle());
        let has_type = schema_dsl::HasType::query();
        let same_typed_expr = has_type.expr.handle().eq(&then_branch.handle());
        let same_lhs_ty = has_type.ty.handle().eq(&lhs_ty.handle());
        let is_not_state = lhs_ty.handle().ne(&state_type::<PR>().handle());

        IfArgPat::new(inputs, then_arg_out, lhs, has_type)
            .assert(same_then_index)
            .assert(same_else_index)
            .assert(same_arg_index)
            .assert(same_then_value)
            .assert(same_else_value)
            .assert(same_typed_expr)
            .assert(same_lhs_ty)
            .assert(is_not_state)
    }

    #[eggplant::pat_vars]
    struct IfStateEdgePat<PR: PatRecSgl> {
        pred: schema_dsl::Expr,
        inputs: schema_dsl::Expr,
        then_: schema_dsl::Expr,
        else_: schema_dsl::Expr,
        outputs: schema_dsl::If,
        then_arg_out: schema_dsl::Get,
        lhs: schema_dsl::Get,
        has_state_type: schema_dsl::HasType,
    }

    fn if_state_edge_pat<PR: PatRecSgl>() -> IfStateEdgePat<PR> {
        let pred = schema_dsl::Expr::query_leaf();
        let inputs = schema_dsl::Expr::query_leaf();
        let then_ = schema_dsl::Expr::query_leaf();
        let else_ = schema_dsl::Expr::query_leaf();
        let outputs = schema_dsl::If::query(&pred, &inputs, &then_, &else_);
        let lhs = schema_dsl::Get::query(&outputs);
        let then_branch = schema_dsl::Get::query(&then_);
        let else_branch = schema_dsl::Get::query(&else_);
        let arg_ty = schema_dsl::Type::query_leaf();
        let then_ctx = schema_dsl::Assumption::query_leaf();
        let else_ctx = schema_dsl::Assumption::query_leaf();
        let then_arg = schema_dsl::Arg::query(&arg_ty, &then_ctx);
        let else_arg = schema_dsl::Arg::query(&arg_ty, &else_ctx);
        let then_arg_out = schema_dsl::Get::query(&then_arg);
        let else_arg_out = schema_dsl::Get::query(&else_arg);
        let state_ty = state_type::<PR>();
        let same_then_index = then_branch.handle_index().eq(&lhs.handle_index());
        let same_else_index = else_branch.handle_index().eq(&lhs.handle_index());
        let same_arg_index = then_arg_out.handle_index().eq(&else_arg_out.handle_index());
        let same_then_value = then_branch.handle().eq(&then_arg_out.handle());
        let same_else_value = else_branch.handle().eq(&else_arg_out.handle());
        let has_state_type = schema_dsl::HasType::query();
        let same_typed_expr = has_state_type.expr.handle().eq(&then_branch.handle());
        let same_state_ty = has_state_type.ty.handle().eq(&state_ty.handle());

        IfStateEdgePat::new(
            pred,
            inputs,
            then_,
            else_,
            outputs,
            then_arg_out,
            lhs,
            has_state_type,
        )
        .assert(same_then_index)
        .assert(same_else_index)
        .assert(same_arg_index)
        .assert(same_then_value)
        .assert(same_else_value)
        .assert(same_typed_expr)
        .assert(same_state_ty)
    }

    #[eggplant::pat_vars]
    struct IfPredicatePat<PR: PatRecSgl> {
        pred: schema_dsl::Expr,
        inputs: schema_dsl::Expr,
        then_: schema_dsl::Expr,
        else_: schema_dsl::Expr,
        if_expr: schema_dsl::If,
        lhs: schema_dsl::Get,
    }

    fn if_predicate_pat<PR: PatRecSgl>() -> IfPredicatePat<PR> {
        let pred = schema_dsl::Expr::query_leaf();
        let inputs = schema_dsl::Expr::query_leaf();
        let then_ = schema_dsl::Expr::query_leaf();
        let else_ = schema_dsl::Expr::query_leaf();
        let if_expr = schema_dsl::If::query(&pred, &inputs, &then_, &else_);
        let lhs = schema_dsl::Get::query(&if_expr);
        let then_out = schema_dsl::Get::query(&then_);
        let else_out = schema_dsl::Get::query(&else_);
        let (true_const, true_matches_value) = bool_const::<PR>(true);
        let (false_const, false_matches_value) = bool_const::<PR>(false);
        let same_then_index = then_out.handle_index().eq(&lhs.handle_index());
        let same_else_index = else_out.handle_index().eq(&lhs.handle_index());
        let then_is_true = then_out.handle().eq(&true_const.handle());
        let else_is_false = else_out.handle().eq(&false_const.handle());

        IfPredicatePat::new(pred, inputs, then_, else_, if_expr, lhs)
            .assert(same_then_index)
            .assert(same_else_index)
            .assert(then_is_true)
            .assert(true_matches_value)
            .assert(else_is_false)
            .assert(false_matches_value)
    }

    #[eggplant::pat_vars]
    struct IfPredicateInvertedPat<PR: PatRecSgl> {
        pred: schema_dsl::Expr,
        inputs: schema_dsl::Expr,
        then_: schema_dsl::Expr,
        else_: schema_dsl::Expr,
        if_expr: schema_dsl::If,
        lhs: schema_dsl::Get,
    }

    fn if_predicate_inverted_pat<PR: PatRecSgl>() -> IfPredicateInvertedPat<PR> {
        let pred = schema_dsl::Expr::query_leaf();
        let inputs = schema_dsl::Expr::query_leaf();
        let then_ = schema_dsl::Expr::query_leaf();
        let else_ = schema_dsl::Expr::query_leaf();
        let if_expr = schema_dsl::If::query(&pred, &inputs, &then_, &else_);
        let lhs = schema_dsl::Get::query(&if_expr);
        let then_out = schema_dsl::Get::query(&then_);
        let else_out = schema_dsl::Get::query(&else_);
        let (false_const, false_matches_value) = bool_const::<PR>(false);
        let (true_const, true_matches_value) = bool_const::<PR>(true);
        let same_then_index = then_out.handle_index().eq(&lhs.handle_index());
        let same_else_index = else_out.handle_index().eq(&lhs.handle_index());
        let then_is_false = then_out.handle().eq(&false_const.handle());
        let else_is_true = else_out.handle().eq(&true_const.handle());

        IfPredicateInvertedPat::new(pred, inputs, then_, else_, if_expr, lhs)
            .assert(same_then_index)
            .assert(same_else_index)
            .assert(then_is_false)
            .assert(false_matches_value)
            .assert(else_is_true)
            .assert(true_matches_value)
    }

    pub(crate) fn register_native_rules() -> RuleSetId {
        let ruleset = PeepholeTx::new_ruleset("passthrough");
        let state_edge_ruleset = PeepholeTx::new_ruleset("state-edge-passthrough");

        PeepholeTx::add_rule(
            "passthrough_loop_theta",
            ruleset,
            loop_theta_pat,
            |ctx, pat| {
                let passthrough = ctx.insert_get(pat.inputs, ctx.devalue(pat.lhs.index));
                ctx.union(pat.lhs, passthrough);
            },
        );

        PeepholeTx::add_rule(
            "passthrough_switch_arg",
            ruleset,
            switch_arg_pat,
            |ctx, pat| {
                let passthrough = ctx.insert_get(pat.inputs, ctx.devalue(pat.arg0_out.index));
                ctx.union(pat.lhs, passthrough);
            },
        );

        PeepholeTx::add_rule(
            "passthrough_switch_predicate",
            ruleset,
            switch_predicate_pat,
            |ctx, pat| {
                ctx.union(pat.lhs, pat.pred);
            },
        );

        PeepholeTx::add_rule("passthrough_if_arg", ruleset, if_arg_pat, |ctx, pat| {
            let passthrough = ctx.insert_get(pat.inputs, ctx.devalue(pat.then_arg_out.index));
            ctx.union(pat.lhs, passthrough);
        });

        PeepholeTx::add_rule(
            "passthrough_if_state_edge",
            state_edge_ruleset,
            if_state_edge_pat,
            |ctx, pat| {
                let passthrough_index = pat.then_arg_out.index.val;
                let output_index = pat.lhs.index.val;
                let new_inputs = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "TupleRemoveAt",
                    &[pat.inputs.to_value(&ctx).val, passthrough_index],
                ));
                let new_then_ctx = ctx.insert_in_if(true, pat.pred, new_inputs);
                let new_else_ctx = ctx.insert_in_if(false, pat.pred, new_inputs);
                let old_then = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "TupleRemoveAt",
                    &[pat.then_.to_value(&ctx).val, output_index],
                ));
                let old_else = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "TupleRemoveAt",
                    &[pat.else_.to_value(&ctx).val, output_index],
                ));
                let new_then = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "DropAt",
                    &[
                        new_then_ctx.to_value(&ctx).val,
                        passthrough_index,
                        old_then.to_value(&ctx).val,
                    ],
                ));
                let new_else = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "DropAt",
                    &[
                        new_else_ctx.to_value(&ctx).val,
                        passthrough_index,
                        old_else.to_value(&ctx).val,
                    ],
                ));
                let old_outputs = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "TupleRemoveAt",
                    &[pat.outputs.to_value(&ctx).val, output_index],
                ));
                let new_if = ctx.insert_if(pat.pred, new_inputs, new_then, new_else);
                let passthrough = ctx.insert_get(pat.inputs, ctx.devalue(pat.then_arg_out.index));

                ctx.union(new_if, old_outputs);
                ctx.union(pat.lhs, passthrough);
                ctx.insert_to_subsume_if(pat.pred, pat.inputs, pat.then_, pat.else_);
            },
        );

        PeepholeTx::add_rule(
            "passthrough_if_predicate",
            ruleset,
            if_predicate_pat,
            |ctx, pat| {
                let new_then = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "TupleRemoveAt",
                    &[pat.then_.to_value(&ctx).val, pat.lhs.index.val],
                ));
                let new_else = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "TupleRemoveAt",
                    &[pat.else_.to_value(&ctx).val, pat.lhs.index.val],
                ));
                let new_if = ctx.insert_if(pat.pred, pat.inputs, new_then, new_else);
                let removed = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "TupleRemoveAt",
                    &[pat.if_expr.to_value(&ctx).val, pat.lhs.index.val],
                ));
                ctx.union(pat.lhs, pat.pred);
                ctx.union(removed, new_if);
            },
        );

        PeepholeTx::add_rule(
            "passthrough_if_predicate_inverted",
            ruleset,
            if_predicate_inverted_pat,
            |ctx, pat| {
                let new_then = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "TupleRemoveAt",
                    &[pat.then_.to_value(&ctx).val, pat.lhs.index.val],
                ));
                let new_else = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "TupleRemoveAt",
                    &[pat.else_.to_value(&ctx).val, pat.lhs.index.val],
                ));
                let new_if = ctx.insert_if(pat.pred, pat.inputs, new_then, new_else);
                let removed = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "TupleRemoveAt",
                    &[pat.if_expr.to_value(&ctx).val, pat.lhs.index.val],
                ));
                let not_op = ctx.insert_not();
                let inverted = ctx.insert_uop(not_op, pat.pred);
                ctx.union(pat.lhs, inverted);
                ctx.union(removed, new_if);
            },
        );

        ruleset
    }
}
