pub(crate) fn fragment() -> String {
    let mut out = String::new();
    out.push_str(GENERATED_MARKER);
    out.push_str(CONDITIONAL_PUSH_IN);
    out.push('\n');
    out
}

const GENERATED_MARKER: &str =
    "; (Generated from eggplant Rust: src/eggplant_backend/conditional_push_in.rs)\n";
const CONDITIONAL_PUSH_IN: &str = r#"(ruleset push-in)

; new version of the rule where one side of bop is constant
(rule (
        (= if_e (If pred orig_inputs thn els))
        (ContextOf if_e outer_ctx)
        (= (Bop o (Const c ty outer_ctx) x) (Get orig_inputs i))
        (HasArgType thn (TupleT tylist))
        (HasArgType els (TupleT tylist))
        (HasType x (Base x_ty))
        ; ensure pure type for weak linearity
        (PureBaseType x_ty)
        (= orig_ins_len (TypeList-length tylist))
      )
      (
        (RELIESONCONTEXT)
        ; New inputs
        (let new_ins (Concat orig_inputs (Single x)))
        (let new_ins_ty (TupleT (TLConcat tylist (TCons x_ty (TNil)))))

        ; New contexts
        (let if_tr (InIf true  pred new_ins))
        (let if_fa (InIf false pred new_ins))

        ; New args
        (let arg_tr (Arg new_ins_ty if_tr))
        (let arg_fa (Arg new_ins_ty if_fa))

        ; SubTuple
        (let st_tr (SubTuple arg_tr 0 orig_ins_len))
        (let st_fa (SubTuple arg_fa 0 orig_ins_len))

        ; New regions
        (let new_thn (Subst if_tr st_tr thn))
        (let new_els (Subst if_fa st_fa els))

        ; Union the original input with Bop(c, x) in the new regions
        (union (Get arg_tr i) (Bop o (Const c new_ins_ty if_tr) (Get arg_tr orig_ins_len)))
        (union (Get arg_fa i) (Bop o (Const c new_ins_ty if_fa) (Get arg_fa orig_ins_len)))

        ; Union the ifs
        (union if_e (If pred new_ins new_thn new_els))
      )
      :ruleset push-in)


; OLD VERSION - Too slow for now
; ; push bop input into region
; (rule (
;         (= if_e (If pred orig_inputs thn els))
;         (ContextOf if_e outer_ctx)
;         (= (Bop o x y) (Get orig_inputs i))
;         (HasArgType thn (TupleT tylist))
;         (HasArgType els (TupleT tylist))
;         (HasType x (Base x_ty))
;         (HasType y (Base y_ty))
;       )
;       (
;         ; New inputs
;         (let new_ins (Concat orig_inputs (Concat (Single x) (Single y))))
;         (let new_ins_ty (TupleT (TLConcat tylist (TCons x_ty (TCons y_ty (TNil))))))

;         ; New contexts
;         (let if_tr (InIf true  pred new_ins))
;         (let if_fa (InIf false pred new_ins))
        
;         ; New args
;         (let arg_tr (Arg new_ins_ty if_tr))
;         (let arg_fa (Arg new_ins_ty if_fa))

;         ; SubTuple
;         (let orig_ins_len (TypeList-length tylist))
;         (let st_tr (SubTuple arg_tr 0 orig_ins_len))
;         (let st_fa (SubTuple arg_fa 0 orig_ins_len))

;         ; New regions
;         (let new_thn (Subst if_tr st_tr thn))
;         (let new_els (Subst if_fa st_fa els))

;         ; Union the original input with Bop(x, y) in the new regions
;         (union (Get arg_tr i) (Bop o (Get arg_tr orig_ins_len) (Get arg_tr (+ orig_ins_len 1))))
;         (union (Get arg_fa i) (Bop o (Get arg_fa orig_ins_len) (Get arg_fa (+ orig_ins_len 1))))

;         ; Union the ifs
;         (union if_e (If pred new_ins new_thn new_els))
;       )
;       :ruleset push-in)"#;

#[cfg(feature = "eggplant")]
pub(crate) mod native {
    use super::super::native_rule_helpers::insert_call;
    use super::super::schema_dsl;
    use crate::eggplant_backend::peepholes::native::PeepholeTx;
    use eggplant::prelude::{
        prim_fact, Insertable, IntoHandleTy, PEq, PatRecSgl, RuleRunnerSgl, RuleSetId,
    };

    #[eggplant::pat_vars]
    struct PushInPat<PR: PatRecSgl> {
        if_e: schema_dsl::If,
        pred: schema_dsl::Expr,
        orig_inputs: schema_dsl::Expr,
        thn: schema_dsl::Expr,
        els: schema_dsl::Expr,
        op: schema_dsl::BinaryOp,
        constant: schema_dsl::Constant,
        x: schema_dsl::Expr,
        x_ty: schema_dsl::BaseType,
        tylist: schema_dsl::TypeList,
        orig_input_i: schema_dsl::Get,
    }

    fn push_in_pat<PR: PatRecSgl>() -> PushInPat<PR> {
        let pred = schema_dsl::Expr::query_leaf();
        let orig_inputs = schema_dsl::Expr::query_leaf();
        let thn = schema_dsl::Expr::query_leaf();
        let els = schema_dsl::Expr::query_leaf();
        let if_e = schema_dsl::If::query(&pred, &orig_inputs, &thn, &els);
        let outer_ctx = schema_dsl::Assumption::query_leaf();
        let op = schema_dsl::BinaryOp::query_leaf();
        let constant = schema_dsl::Constant::query_leaf();
        let const_ty = schema_dsl::Type::query_leaf();
        let x = schema_dsl::Expr::query_leaf();
        let const_expr = schema_dsl::Const::query(&constant, &const_ty, &outer_ctx);
        let pushed_expr = schema_dsl::Bop::query(&op, &const_expr, &x);
        let orig_input_i = schema_dsl::Get::query(&orig_inputs);

        let tylist = schema_dsl::TypeList::query_leaf();
        let branch_arg_ty = schema_dsl::TupleT::query(&tylist);
        let x_ty = schema_dsl::BaseType::query_leaf();
        let x_ty_expr = schema_dsl::Base::query(&x_ty);

        let if_context = prim_fact(
            "ContextOf",
            vec![
                if_e.handle().into_handle_ty(),
                outer_ctx.handle().into_handle_ty(),
            ],
        );
        let input_matches = orig_input_i.handle().eq(&pushed_expr.handle());
        let then_arg_type = prim_fact(
            "HasArgType",
            vec![
                thn.handle().into_handle_ty(),
                branch_arg_ty.handle().into_handle_ty(),
            ],
        );
        let else_arg_type = prim_fact(
            "HasArgType",
            vec![
                els.handle().into_handle_ty(),
                branch_arg_ty.handle().into_handle_ty(),
            ],
        );
        let x_has_type = prim_fact(
            "HasType",
            vec![
                x.handle().into_handle_ty(),
                x_ty_expr.handle().into_handle_ty(),
            ],
        );
        let pure_base = prim_fact("PureBaseType", vec![x_ty.handle().into_handle_ty()]);

        PushInPat::new(
            if_e,
            pred,
            orig_inputs,
            thn,
            els,
            op,
            constant,
            x,
            x_ty,
            tylist,
            orig_input_i,
        )
        .assert(if_context)
        .assert(input_matches)
        .assert(then_arg_type)
        .assert(else_arg_type)
        .assert(x_has_type)
        .assert(pure_base)
    }

    pub(crate) fn register_native_rules() -> RuleSetId {
        let ruleset = PeepholeTx::new_ruleset("push-in");

        PeepholeTx::add_rule(
            "conditional_push_in_const_operand",
            ruleset,
            push_in_pat,
            |ctx, pat| {
                let zero = ctx._intern_base::<i64, i64>(0);
                let orig_ins_len =
                    ctx.lookup_expect("TypeList-length", &[pat.tylist.to_value(&ctx.ctx).val]);

                let tnil = insert_call::<schema_dsl::TypeList>(&ctx.ctx, "TNil", &[]);
                let appended_tail = insert_call::<schema_dsl::TypeList>(
                    &ctx.ctx,
                    "TCons",
                    &[pat.x_ty.to_value(&ctx.ctx).val, tnil.0.val],
                );
                let new_tylist = insert_call::<schema_dsl::TypeList>(
                    &ctx.ctx,
                    "TLConcat",
                    &[
                        pat.tylist.to_value(&ctx.ctx).val,
                        appended_tail.to_value(&ctx.ctx).val,
                    ],
                );
                let new_ins_ty =
                    insert_call::<schema_dsl::Type>(&ctx.ctx, "TupleT", &[new_tylist.0.val]);
                let new_ins = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "Concat",
                    &[
                        pat.orig_inputs.to_value(&ctx.ctx).val,
                        insert_call::<schema_dsl::Expr>(
                            &ctx.ctx,
                            "Single",
                            &[pat.x.to_value(&ctx.ctx).val],
                        )
                        .to_value(&ctx.ctx)
                        .val,
                    ],
                );

                let if_tr = insert_call::<schema_dsl::Assumption>(
                    &ctx.ctx,
                    "InIf",
                    &[
                        true.to_value(&ctx.ctx).val,
                        pat.pred.to_value(&ctx.ctx).val,
                        new_ins.to_value(&ctx.ctx).val,
                    ],
                );
                let if_fa = insert_call::<schema_dsl::Assumption>(
                    &ctx.ctx,
                    "InIf",
                    &[
                        false.to_value(&ctx.ctx).val,
                        pat.pred.to_value(&ctx.ctx).val,
                        new_ins.to_value(&ctx.ctx).val,
                    ],
                );

                let arg_tr = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "Arg",
                    &[
                        new_ins_ty.to_value(&ctx.ctx).val,
                        if_tr.to_value(&ctx.ctx).val,
                    ],
                );
                let arg_fa = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "Arg",
                    &[
                        new_ins_ty.to_value(&ctx.ctx).val,
                        if_fa.to_value(&ctx.ctx).val,
                    ],
                );
                let st_tr = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "SubTuple",
                    &[arg_tr.to_value(&ctx.ctx).val, zero, orig_ins_len],
                );
                let st_fa = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "SubTuple",
                    &[arg_fa.to_value(&ctx.ctx).val, zero, orig_ins_len],
                );
                let new_thn = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "Subst",
                    &[
                        if_tr.to_value(&ctx.ctx).val,
                        st_tr.to_value(&ctx.ctx).val,
                        pat.thn.to_value(&ctx.ctx).val,
                    ],
                );
                let new_els = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "Subst",
                    &[
                        if_fa.to_value(&ctx.ctx).val,
                        st_fa.to_value(&ctx.ctx).val,
                        pat.els.to_value(&ctx.ctx).val,
                    ],
                );

                let tr_replaced = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "Get",
                    &[arg_tr.to_value(&ctx.ctx).val, pat.orig_input_i.index.val],
                );
                let fa_replaced = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "Get",
                    &[arg_fa.to_value(&ctx.ctx).val, pat.orig_input_i.index.val],
                );
                let tr_const = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "Const",
                    &[
                        pat.constant.to_value(&ctx.ctx).val,
                        new_ins_ty.to_value(&ctx.ctx).val,
                        if_tr.to_value(&ctx.ctx).val,
                    ],
                );
                let fa_const = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "Const",
                    &[
                        pat.constant.to_value(&ctx.ctx).val,
                        new_ins_ty.to_value(&ctx.ctx).val,
                        if_fa.to_value(&ctx.ctx).val,
                    ],
                );
                let tr_rhs = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "Bop",
                    &[
                        pat.op.to_value(&ctx.ctx).val,
                        tr_const.to_value(&ctx.ctx).val,
                        insert_call::<schema_dsl::Expr>(
                            &ctx.ctx,
                            "Get",
                            &[arg_tr.to_value(&ctx.ctx).val, orig_ins_len],
                        )
                        .to_value(&ctx.ctx)
                        .val,
                    ],
                );
                let fa_rhs = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "Bop",
                    &[
                        pat.op.to_value(&ctx.ctx).val,
                        fa_const.to_value(&ctx.ctx).val,
                        insert_call::<schema_dsl::Expr>(
                            &ctx.ctx,
                            "Get",
                            &[arg_fa.to_value(&ctx.ctx).val, orig_ins_len],
                        )
                        .to_value(&ctx.ctx)
                        .val,
                    ],
                );
                ctx.union(tr_replaced, tr_rhs);
                ctx.union(fa_replaced, fa_rhs);

                let new_if = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "If",
                    &[
                        pat.pred.to_value(&ctx.ctx).val,
                        new_ins.to_value(&ctx.ctx).val,
                        new_thn.to_value(&ctx.ctx).val,
                        new_els.to_value(&ctx.ctx).val,
                    ],
                );
                ctx.union(pat.if_e, new_if);
            },
        );

        ruleset
    }
}
