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
    use super::super::schema_dsl::{self, ExprRuleCtx};
    use crate::eggplant_backend::peepholes::native::PeepholeTx;
    use eggplant::prelude::{Insertable, PEq, PatRecSgl, RuleRunnerSgl, RuleSetId};

    #[eggplant::pat_vars]
    struct PushInPat<PR: PatRecSgl> {
        if_e: schema_dsl::If,
        pred: schema_dsl::Expr,
        orig_inputs: schema_dsl::Expr,
        thn: schema_dsl::Expr,
        els: schema_dsl::Expr,
        if_context: schema_dsl::ContextOf,
        then_arg_type: schema_dsl::HasArgType,
        else_arg_type: schema_dsl::HasArgType,
        op: schema_dsl::BinaryOp,
        constant: schema_dsl::Constant,
        x: schema_dsl::Expr,
        x_ty: schema_dsl::BaseType,
        x_has_type: schema_dsl::HasType,
        pure_base: schema_dsl::PureBaseType,
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

        let if_context = schema_dsl::ContextOf::query();
        let input_matches = orig_input_i.handle().eq(&pushed_expr.handle());
        let same_if_expr = if_context.expr.handle().eq(&if_e.handle());
        let same_if_ctx = if_context.ctx.handle().eq(&outer_ctx.handle());
        let then_arg_type = schema_dsl::HasArgType::query();
        let else_arg_type = schema_dsl::HasArgType::query();
        let same_then_expr = then_arg_type.expr.handle().eq(&thn.handle());
        let same_then_ty = then_arg_type.ty.handle().eq(&branch_arg_ty.handle());
        let same_else_expr = else_arg_type.expr.handle().eq(&els.handle());
        let same_else_ty = else_arg_type.ty.handle().eq(&branch_arg_ty.handle());
        let x_has_type = schema_dsl::HasType::query();
        let same_x_expr = x_has_type.expr.handle().eq(&x.handle());
        let same_x_ty = x_has_type.ty.handle().eq(&x_ty_expr.handle());
        let pure_base = schema_dsl::PureBaseType::query();
        let pure_base_matches = pure_base.ty.handle().eq(&x_ty.handle());

        PushInPat::new(
            if_e,
            pred,
            orig_inputs,
            thn,
            els,
            if_context,
            then_arg_type,
            else_arg_type,
            op,
            constant,
            x,
            x_ty,
            x_has_type,
            pure_base,
            tylist,
            orig_input_i,
        )
        .assert(input_matches)
        .assert(same_if_expr)
        .assert(same_if_ctx)
        .assert(same_then_expr)
        .assert(same_then_ty)
        .assert(same_else_expr)
        .assert(same_else_ty)
        .assert(same_x_expr)
        .assert(same_x_ty)
        .assert(pure_base_matches)
    }

    pub(crate) fn register_native_rules() -> RuleSetId {
        let ruleset = PeepholeTx::new_ruleset("push-in");

        PeepholeTx::add_rule(
            "conditional_push_in_const_operand",
            ruleset,
            push_in_pat,
            |ctx, pat| {
                let zero = ctx._intern_base::<i64, i64>(0);
                let orig_ins_len_val =
                    ctx.lookup_expect("TypeList-length", &[pat.tylist.to_value(&ctx).val]);
                let orig_ins_len = ctx.devalue(eggplant::wrap::Value::<i64>::new(orig_ins_len_val));
                let orig_input_index = ctx.devalue(pat.orig_input_i.index);

                let tnil =
                    eggplant::wrap::Value::<schema_dsl::TypeList>::new((ctx).insert("TNil", &[]));
                let appended_tail = eggplant::wrap::Value::<schema_dsl::TypeList>::new(
                    (&ctx).insert("TCons", &[pat.x_ty.to_value(&ctx).val, tnil.val]),
                );
                let new_tylist = eggplant::wrap::Value::<schema_dsl::TypeList>::new((&ctx).insert(
                    "TLConcat",
                    &[
                        pat.tylist.to_value(&ctx).val,
                        appended_tail.to_value(&ctx).val,
                    ],
                ));
                let new_ins_ty = eggplant::wrap::Value::<schema_dsl::Type>::new(
                    (ctx).insert("TupleT", &[new_tylist.val]),
                );
                let new_ins = ctx.insert_concat(pat.orig_inputs, ctx.insert_single(pat.x));

                let if_tr = eggplant::wrap::Value::<schema_dsl::Assumption>::new((&ctx).insert(
                    "InIf",
                    &[
                        true.to_value(&ctx).val,
                        pat.pred.to_value(&ctx).val,
                        new_ins.to_value(&ctx).val,
                    ],
                ));
                let if_fa = eggplant::wrap::Value::<schema_dsl::Assumption>::new((&ctx).insert(
                    "InIf",
                    &[
                        false.to_value(&ctx).val,
                        pat.pred.to_value(&ctx).val,
                        new_ins.to_value(&ctx).val,
                    ],
                ));

                let arg_tr = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "Arg",
                    &[new_ins_ty.to_value(&ctx).val, if_tr.to_value(&ctx).val],
                ));
                let arg_fa = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "Arg",
                    &[new_ins_ty.to_value(&ctx).val, if_fa.to_value(&ctx).val],
                ));
                let st_tr = eggplant::wrap::Value::<schema_dsl::Expr>::new(
                    (&ctx).insert("SubTuple", &[arg_tr.to_value(&ctx).val, zero, orig_ins_len_val]),
                );
                let st_fa = eggplant::wrap::Value::<schema_dsl::Expr>::new(
                    (&ctx).insert("SubTuple", &[arg_fa.to_value(&ctx).val, zero, orig_ins_len_val]),
                );
                let new_thn = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "Subst",
                    &[
                        if_tr.to_value(&ctx).val,
                        st_tr.to_value(&ctx).val,
                        pat.thn.to_value(&ctx).val,
                    ],
                ));
                let new_els = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "Subst",
                    &[
                        if_fa.to_value(&ctx).val,
                        st_fa.to_value(&ctx).val,
                        pat.els.to_value(&ctx).val,
                    ],
                ));

                let tr_replaced = ctx.insert_get(arg_tr, orig_input_index);
                let fa_replaced = ctx.insert_get(arg_fa, orig_input_index);
                let tr_const = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "Const",
                    &[
                        pat.constant.to_value(&ctx).val,
                        new_ins_ty.to_value(&ctx).val,
                        if_tr.to_value(&ctx).val,
                    ],
                ));
                let fa_const = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "Const",
                    &[
                        pat.constant.to_value(&ctx).val,
                        new_ins_ty.to_value(&ctx).val,
                        if_fa.to_value(&ctx).val,
                    ],
                ));
                let tr_rhs = eggplant::wrap::Value::<schema_dsl::Expr>::new(
                    (&ctx).insert(
                        "Bop",
                        &[
                            pat.op.to_value(&ctx).val,
                            tr_const.to_value(&ctx).val,
                            ctx.insert_get(arg_tr, orig_ins_len).val,
                        ],
                    ),
                );
                let fa_rhs = eggplant::wrap::Value::<schema_dsl::Expr>::new(
                    (&ctx).insert(
                        "Bop",
                        &[
                            pat.op.to_value(&ctx).val,
                            fa_const.to_value(&ctx).val,
                            ctx.insert_get(arg_fa, orig_ins_len).val,
                        ],
                    ),
                );
                ctx.union(tr_replaced, tr_rhs);
                ctx.union(fa_replaced, fa_rhs);

                let new_if = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "If",
                    &[
                        pat.pred.to_value(&ctx).val,
                        new_ins.to_value(&ctx).val,
                        new_thn.to_value(&ctx).val,
                        new_els.to_value(&ctx).val,
                    ],
                ));
                ctx.union(pat.if_e, new_if);
            },
        );

        ruleset
    }
}
