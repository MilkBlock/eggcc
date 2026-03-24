pub(crate) fn fragment() -> String {
    let mut out = String::new();
    out.push_str(GENERATED_MARKER);
    out.push_str(SWITCH_REWRITES);
    out.push('\n');
    out
}

const GENERATED_MARKER: &str =
    "; (Generated from eggplant Rust: src/eggplant_backend/switch_rewrites.rs)\n";
const SWITCH_REWRITES: &str = r#"(ruleset switch_rewrite)
(ruleset always-switch-rewrite)

; if a < b then a else b ~~> (min a b)
(rule (
       (= pred (Bop (LessThan) a b))
       (= if_e (If pred inputs thn els))
       ; a is an input to the if region
       (= a (Get inputs i))
       ; b is an input to the if region
       (= b (Get inputs j))
       ; if a < b then a else b
       (= (Get thn k) (Get (Arg ty (InIf true pred inputs)) i))
       (= (Get els k) (Get (Arg ty (InIf false pred inputs)) j))
      )
      ((union (Get if_e k) (Bop (Smin) a b)))
      :ruleset switch_rewrite)

; if a < b then b else a ~~> (max a b)
(rule (
       (= pred (Bop (LessThan) a b))
       (= if_e (If pred inputs thn els))
       ; a is an input to the if region
       (= a (Get inputs i))
       ; b is an input to the if region
       (= b (Get inputs j))
       ; if a < b then b else a
       (= (Get thn k) (Get (Arg ty (InIf true pred inputs)) j))
       (= (Get els k) (Get (Arg ty (InIf false pred inputs)) i))
      )
      ((union (Get if_e k) (Bop (Smax) a b)))
      :ruleset switch_rewrite) 

; if pred then a else b ~~> (select pred a b)
; where a and b are inputs to the region
(rule (
       (= if_e (If pred inputs thn els))
       (= a (Get inputs i))
       (= b (Get inputs j))

       ; if pred then a else b
       (= (Get thn k) (Get (Arg ty (InIf true pred inputs)) i))
       (= (Get els k) (Get (Arg ty (InIf false pred inputs)) j))

       ; If i = j, then the arg is just passed through the if, and we
       ; don't need a select. This will get handled by the passthrough rules.
       (!= i j)
       )
       (
       (union (Get if_e k) (Top (Select) pred a b))
       )
       :ruleset switch_rewrite)

(rule (
       (= if_e (If pred inputs thn els))
       (ContextOf if_e ctx)
       (HasArgType if_e ty)
       (= (Get thn i) (Const x _ty (InIf true pred inputs)))
       (= (Get els i) (Const y _ty (InIf false pred inputs)))
      )
      ((union (Get if_e i) (Top (Select) pred (Const x ty ctx) (Const y ty ctx))))
      :ruleset switch_rewrite)

; if pred then A else Const -> select pred A Const
; where A is an input to the region
(rule (
       (= if_e (If pred inputs thn els))
       (ContextOf if_e ctx)
       (HasArgType if_e ty)

       ; input to the if
       (= a (Get inputs i))
       (= (Get thn k) (Get (Arg _ty (InIf true pred inputs)) i))

       (= els_out (Get els k))
       (= (IntB y) (lo_bound els_out))
       (= (IntB y) (hi_bound els_out))
       )
       (
       (union (Get if_e k) (Top (Select) pred a (Const (Int y) ty ctx)))
       )
       :ruleset switch_rewrite
)

; if pred then Const else B -> select pred Const B
; where B is an input to the region
(rule (
       (= if_e (If pred inputs thn els))
       (ContextOf if_e ctx)
       (HasArgType if_e ty)

       (= thn_out (Get thn k))
       (= (IntB y) (lo_bound thn_out))
       (= (IntB y) (hi_bound thn_out))

       ; input to the if
       (= b (Get inputs i))
       (= (Get els k) (Get (Arg _ty (InIf false pred inputs)) i))
      )
      (
       (union (Get if_e k) (Top (Select) pred (Const (Int y) ty ctx) b))
      )
      :ruleset switch_rewrite
)

; if (a and b) X Y ~~> if a (if b X Y) Y
(rule ((= lhs (If (Bop (And) a b) ins X Y))
       (HasType ins (TupleT ins_ty))
       (= len (tuple-length ins))
       
       ;; For early returns, Y might be fairly large
       ;; limit the size we match on here
       (< (Expr-size Y) 100))

      ((let outer_ins (Concat (Single b) ins))
       (let outer_ins_ty (TupleT (TCons (BoolT) ins_ty)))

       (let inner_pred    (Get      (Arg outer_ins_ty (InIf true  a outer_ins)) 0))
       (let sub_arg_true  (SubTuple (Arg outer_ins_ty (InIf true  a outer_ins)) 1 len))
       (let sub_arg_false (SubTuple (Arg outer_ins_ty (InIf false a outer_ins)) 1 len))

       (let inner_Y (AddContext (InIf false inner_pred sub_arg_true) Y))
       (let outer_Y (Subst      (InIf false a          outer_ins) sub_arg_false Y))

       (let inner (If inner_pred sub_arg_true X inner_Y))
       (union lhs (If a          outer_ins    inner   outer_Y)))

       :ruleset switch_rewrite)

; if (a or b) X Y ~~> if a X (if b X Y)
(rule ((= lhs (If (Bop (Or) a b) ins X Y))
       (HasType ins (TupleT ins_ty))
       (= len (tuple-length ins))
       
       ;; limit the size, since X and Y get new contexts
       (< (Expr-size X) 100)
       (< (Expr-size Y) 100)
       )

      ((let outer_ins (Concat (Single b) ins))
       (let outer_ins_ty (TupleT (TCons (BoolT) ins_ty)))

       (let inner_pred    (Get      (Arg outer_ins_ty (InIf false a outer_ins)) 0))
       (let sub_arg_true  (SubTuple (Arg outer_ins_ty (InIf true  a outer_ins)) 1 len))
       (let sub_arg_false (SubTuple (Arg outer_ins_ty (InIf false a outer_ins)) 1 len))

       (let outer_X (Subst      (InIf true  a          outer_ins) sub_arg_true X))
       (let inner_X (AddContext (InIf true  inner_pred sub_arg_false) X))
       (let inner_Y (AddContext (InIf false inner_pred sub_arg_false) Y))

       (let inner (If inner_pred sub_arg_false inner_X inner_Y))
       (union lhs (If a          outer_ins     outer_X inner  )))

       :ruleset switch_rewrite)"#;

#[cfg(feature = "eggplant")]
pub(crate) mod native {
    // Required by the `#[eggplant::dsl]` expansion below.
    use super::super::native_rule_helpers::insert_call;
    use super::super::schema_dsl;
    use crate::eggplant_backend::interval_bounds::{hi_bound, lo_bound, IntB};
    use crate::eggplant_backend::peepholes::native::PeepholeTx;
    use eggplant::prelude::{
        prim_call, BaseVar, Compare, Insertable, IntoHandleTy, PEq, PatRecSgl, RuleRunnerSgl,
        RuleSetId,
    };

    pub(crate) fn ensure_always_native_ruleset() -> RuleSetId {
        PeepholeTx::new_ruleset("always-switch-rewrite")
    }

    #[eggplant::pat_vars]
    struct SwitchMinPat<PR: PatRecSgl> {
        a: schema_dsl::Expr,
        b: schema_dsl::Expr,
        if_out: schema_dsl::Expr,
        ty: schema_dsl::Type,
    }

    fn switch_min_pat<PR: PatRecSgl>() -> SwitchMinPat<PR> {
        let a = schema_dsl::Expr::query_leaf();
        let b = schema_dsl::Expr::query_leaf();
        let pred = schema_dsl::Bop::query(&schema_dsl::LessThan::query(), &a, &b);
        let inputs = schema_dsl::Expr::query_leaf();
        let thn = schema_dsl::Expr::query_leaf();
        let els = schema_dsl::Expr::query_leaf();
        let if_e = schema_dsl::If::query(&pred, &inputs, &thn, &els);
        let if_out = schema_dsl::Expr::query_leaf();
        let if_out_get = schema_dsl::Get::query(&if_e);
        let a_get = schema_dsl::Get::query(&inputs);
        let b_get = schema_dsl::Get::query(&inputs);
        let thn_out = schema_dsl::Get::query(&thn);
        let els_out = schema_dsl::Get::query(&els);
        let ty = schema_dsl::Type::query_leaf();
        let thn_ctx = schema_dsl::Assumption::query_leaf();
        let els_ctx = schema_dsl::Assumption::query_leaf();
        let thn_arg = schema_dsl::Arg::query(&ty, &thn_ctx);
        let els_arg = schema_dsl::Arg::query(&ty, &els_ctx);
        let thn_arg_out = schema_dsl::Get::query(&thn_arg);
        let els_arg_out = schema_dsl::Get::query(&els_arg);
        let if_out_handle = if_out.handle();
        let a_handle = a.handle();
        let b_handle = b.handle();
        let true_ctx = prim_call::<schema_dsl::Assumption>(
            "InIf",
            vec![
                (&true).into_handle_ty(),
                pred.handle().into_handle_ty(),
                inputs.handle().into_handle_ty(),
            ],
        );
        let false_ctx = prim_call::<schema_dsl::Assumption>(
            "InIf",
            vec![
                (&false).into_handle_ty(),
                pred.handle().into_handle_ty(),
                inputs.handle().into_handle_ty(),
            ],
        );

        SwitchMinPat::new(a, b, if_out, ty)
            .assert(if_out_handle.eq(&if_out_get.handle()))
            .assert(a_handle.eq(&a_get.handle()))
            .assert(b_handle.eq(&b_get.handle()))
            .assert(if_out_get.handle_index().eq(&thn_out.handle_index()))
            .assert(if_out_get.handle_index().eq(&els_out.handle_index()))
            .assert(thn_ctx.handle().eq(&true_ctx))
            .assert(els_ctx.handle().eq(&false_ctx))
            .assert(thn_out.handle().eq(&thn_arg_out.handle()))
            .assert(els_out.handle().eq(&els_arg_out.handle()))
    }

    #[eggplant::pat_vars]
    struct SwitchMaxPat<PR: PatRecSgl> {
        a: schema_dsl::Expr,
        b: schema_dsl::Expr,
        if_out: schema_dsl::Expr,
        ty: schema_dsl::Type,
    }

    fn switch_max_pat<PR: PatRecSgl>() -> SwitchMaxPat<PR> {
        let a = schema_dsl::Expr::query_leaf();
        let b = schema_dsl::Expr::query_leaf();
        let pred = schema_dsl::Bop::query(&schema_dsl::LessThan::query(), &a, &b);
        let inputs = schema_dsl::Expr::query_leaf();
        let thn = schema_dsl::Expr::query_leaf();
        let els = schema_dsl::Expr::query_leaf();
        let if_e = schema_dsl::If::query(&pred, &inputs, &thn, &els);
        let if_out = schema_dsl::Expr::query_leaf();
        let if_out_get = schema_dsl::Get::query(&if_e);
        let a_get = schema_dsl::Get::query(&inputs);
        let b_get = schema_dsl::Get::query(&inputs);
        let thn_out = schema_dsl::Get::query(&thn);
        let els_out = schema_dsl::Get::query(&els);
        let ty = schema_dsl::Type::query_leaf();
        let thn_ctx = schema_dsl::Assumption::query_leaf();
        let els_ctx = schema_dsl::Assumption::query_leaf();
        let thn_arg = schema_dsl::Arg::query(&ty, &thn_ctx);
        let els_arg = schema_dsl::Arg::query(&ty, &els_ctx);
        let thn_arg_out = schema_dsl::Get::query(&thn_arg);
        let els_arg_out = schema_dsl::Get::query(&els_arg);
        let if_out_handle = if_out.handle();
        let a_handle = a.handle();
        let b_handle = b.handle();
        let true_ctx = prim_call::<schema_dsl::Assumption>(
            "InIf",
            vec![
                (&true).into_handle_ty(),
                pred.handle().into_handle_ty(),
                inputs.handle().into_handle_ty(),
            ],
        );
        let false_ctx = prim_call::<schema_dsl::Assumption>(
            "InIf",
            vec![
                (&false).into_handle_ty(),
                pred.handle().into_handle_ty(),
                inputs.handle().into_handle_ty(),
            ],
        );

        SwitchMaxPat::new(a, b, if_out, ty)
            .assert(if_out_handle.eq(&if_out_get.handle()))
            .assert(a_handle.eq(&a_get.handle()))
            .assert(b_handle.eq(&b_get.handle()))
            .assert(if_out_get.handle_index().eq(&thn_out.handle_index()))
            .assert(if_out_get.handle_index().eq(&els_out.handle_index()))
            .assert(thn_ctx.handle().eq(&true_ctx))
            .assert(els_ctx.handle().eq(&false_ctx))
            .assert(thn_out.handle_index().eq(&b_get.handle_index()))
            .assert(els_out.handle_index().eq(&a_get.handle_index()))
            .assert(thn_out.handle().eq(&thn_arg_out.handle()))
            .assert(els_out.handle().eq(&els_arg_out.handle()))
    }

    #[eggplant::pat_vars]
    struct SwitchSelectPat<PR: PatRecSgl> {
        pred: schema_dsl::Expr,
        a: schema_dsl::Expr,
        b: schema_dsl::Expr,
        if_out: schema_dsl::Expr,
        ty: schema_dsl::Type,
    }

    fn switch_select_pat<PR: PatRecSgl>() -> SwitchSelectPat<PR> {
        let pred = schema_dsl::Expr::query_leaf();
        let inputs = schema_dsl::Expr::query_leaf();
        let thn = schema_dsl::Expr::query_leaf();
        let els = schema_dsl::Expr::query_leaf();
        let if_e = schema_dsl::If::query(&pred, &inputs, &thn, &els);
        let if_out = schema_dsl::Expr::query_leaf();
        let a = schema_dsl::Expr::query_leaf();
        let b = schema_dsl::Expr::query_leaf();
        let if_out_get = schema_dsl::Get::query(&if_e);
        let a_get = schema_dsl::Get::query(&inputs);
        let b_get = schema_dsl::Get::query(&inputs);
        let thn_out = schema_dsl::Get::query(&thn);
        let els_out = schema_dsl::Get::query(&els);
        let ty = schema_dsl::Type::query_leaf();
        let thn_ctx = schema_dsl::Assumption::query_leaf();
        let els_ctx = schema_dsl::Assumption::query_leaf();
        let thn_arg = schema_dsl::Arg::query(&ty, &thn_ctx);
        let els_arg = schema_dsl::Arg::query(&ty, &els_ctx);
        let thn_arg_out = schema_dsl::Get::query(&thn_arg);
        let els_arg_out = schema_dsl::Get::query(&els_arg);
        let if_out_handle = if_out.handle();
        let a_handle = a.handle();
        let b_handle = b.handle();
        let true_ctx = prim_call::<schema_dsl::Assumption>(
            "InIf",
            vec![
                (&true).into_handle_ty(),
                pred.handle().into_handle_ty(),
                inputs.handle().into_handle_ty(),
            ],
        );
        let false_ctx = prim_call::<schema_dsl::Assumption>(
            "InIf",
            vec![
                (&false).into_handle_ty(),
                pred.handle().into_handle_ty(),
                inputs.handle().into_handle_ty(),
            ],
        );

        SwitchSelectPat::new(pred, a, b, if_out, ty)
            .assert(if_out_handle.eq(&if_out_get.handle()))
            .assert(a_handle.eq(&a_get.handle()))
            .assert(b_handle.eq(&b_get.handle()))
            .assert(if_out_get.handle_index().eq(&thn_out.handle_index()))
            .assert(if_out_get.handle_index().eq(&els_out.handle_index()))
            .assert(thn_ctx.handle().eq(&true_ctx))
            .assert(els_ctx.handle().eq(&false_ctx))
            .assert(thn_out.handle().eq(&thn_arg_out.handle()))
            .assert(els_out.handle().eq(&els_arg_out.handle()))
            .assert(a_get.handle_index().ne(&b_get.handle_index()))
    }

    #[eggplant::pat_vars]
    struct SwitchSelectConstPat<PR: PatRecSgl> {
        pred: schema_dsl::Expr,
        ctx: schema_dsl::Assumption,
        ty: schema_dsl::Type,
        x: schema_dsl::Constant,
        y: schema_dsl::Constant,
        if_out: schema_dsl::Expr,
    }

    fn switch_select_const_pat<PR: PatRecSgl>() -> SwitchSelectConstPat<PR> {
        let pred = schema_dsl::Expr::query_leaf();
        let inputs = schema_dsl::Expr::query_leaf();
        let thn = schema_dsl::Expr::query_leaf();
        let els = schema_dsl::Expr::query_leaf();
        let if_e = schema_dsl::If::query(&pred, &inputs, &thn, &els);
        let if_out = schema_dsl::Expr::query_leaf();
        let ctx = schema_dsl::Assumption::query_leaf();
        let ty = schema_dsl::Type::query_leaf();
        let x = schema_dsl::Constant::query_leaf();
        let y = schema_dsl::Constant::query_leaf();
        let branch_ty = schema_dsl::Type::query_leaf();
        let thn_ctx = schema_dsl::Assumption::query_leaf();
        let els_ctx = schema_dsl::Assumption::query_leaf();
        let thn_const = schema_dsl::Const::query(&x, &branch_ty, &thn_ctx);
        let els_const = schema_dsl::Const::query(&y, &branch_ty, &els_ctx);
        let if_out_get = schema_dsl::Get::query(&if_e);
        let thn_out = schema_dsl::Get::query(&thn);
        let els_out = schema_dsl::Get::query(&els);

        let ctx_of_if = eggplant::wrap::FactCallConstraint {
            op: "ContextOf",
            operands: vec![
                if_e.handle().into_handle_ty(),
                ctx.handle().into_handle_ty(),
            ],
        };
        let has_arg_ty = eggplant::wrap::FactCallConstraint {
            op: "HasArgType",
            operands: vec![if_e.handle().into_handle_ty(), ty.handle().into_handle_ty()],
        };
        let if_out_matches = if_out.handle().eq(&if_out_get.handle());
        let thn_const_matches = thn_const.handle().eq(&thn_out.handle());
        let els_const_matches = els_const.handle().eq(&els_out.handle());
        let same_thn_index = if_out_get.handle_index().eq(&thn_out.handle_index());
        let same_els_index = if_out_get.handle_index().eq(&els_out.handle_index());
        let thn_ctx_matches = thn_ctx.handle().eq(&prim_call::<schema_dsl::Assumption>(
            "InIf",
            vec![
                (&true).into_handle_ty(),
                pred.handle().into_handle_ty(),
                inputs.handle().into_handle_ty(),
            ],
        ));
        let els_ctx_matches = els_ctx.handle().eq(&prim_call::<schema_dsl::Assumption>(
            "InIf",
            vec![
                (&false).into_handle_ty(),
                pred.handle().into_handle_ty(),
                inputs.handle().into_handle_ty(),
            ],
        ));

        SwitchSelectConstPat::new(pred, ctx, ty, x, y, if_out)
            .assert(ctx_of_if)
            .assert(has_arg_ty)
            .assert(if_out_matches)
            .assert(thn_const_matches)
            .assert(els_const_matches)
            .assert(same_thn_index)
            .assert(same_els_index)
            .assert(thn_ctx_matches)
            .assert(els_ctx_matches)
    }

    #[eggplant::pat_vars]
    struct SwitchSelectElseConstPat<PR: PatRecSgl> {
        pred: schema_dsl::Expr,
        a: schema_dsl::Expr,
        ctx: schema_dsl::Assumption,
        ty: schema_dsl::Type,
        thn_ctx: schema_dsl::Assumption,
        y: IntB,
        if_out: schema_dsl::Expr,
    }

    fn switch_select_else_const_pat<PR: PatRecSgl>() -> SwitchSelectElseConstPat<PR> {
        let pred = schema_dsl::Expr::query_leaf();
        let inputs = schema_dsl::Expr::query_leaf();
        let thn = schema_dsl::Expr::query_leaf();
        let els = schema_dsl::Expr::query_leaf();
        let if_e = schema_dsl::If::query(&pred, &inputs, &thn, &els);
        let if_out = schema_dsl::Expr::query_leaf();
        let a = schema_dsl::Expr::query_leaf();
        let ctx = schema_dsl::Assumption::query_leaf();
        let ty = schema_dsl::Type::query_leaf();
        let thn_out = schema_dsl::Get::query(&thn);
        let els_out = schema_dsl::Get::query(&els);
        let thn_ctx = schema_dsl::Assumption::query_leaf();
        let branch_ty = schema_dsl::Type::query_leaf();
        let y = IntB::query();
        let if_out_get = schema_dsl::Get::query(&if_e);
        let a_get = schema_dsl::Get::query(&inputs);
        let thn_arg_out = schema_dsl::Get::query(&schema_dsl::Arg::query(&branch_ty, &thn_ctx));
        let ctx_of_if = eggplant::wrap::FactCallConstraint {
            op: "ContextOf",
            operands: vec![
                if_e.handle().into_handle_ty(),
                ctx.handle().into_handle_ty(),
            ],
        };
        let has_arg_ty = eggplant::wrap::FactCallConstraint {
            op: "HasArgType",
            operands: vec![if_e.handle().into_handle_ty(), ty.handle().into_handle_ty()],
        };
        let if_out_matches = if_out.handle().eq(&if_out_get.handle());
        let a_matches = a.handle().eq(&a_get.handle());
        let same_thn_index = if_out_get.handle_index().eq(&thn_out.handle_index());
        let same_els_index = if_out_get.handle_index().eq(&els_out.handle_index());
        let same_arg_index = a_get.handle_index().eq(&thn_arg_out.handle_index());
        let thn_ctx_matches = thn_ctx.handle().eq(&prim_call::<schema_dsl::Assumption>(
            "InIf",
            vec![
                (&true).into_handle_ty(),
                pred.handle().into_handle_ty(),
                inputs.handle().into_handle_ty(),
            ],
        ));
        let thn_matches_arg = thn_out.handle().eq(&thn_arg_out.handle());
        let lo_bound = lo_bound::query(&els_out).handle().eq(&y.handle());
        let hi_bound = hi_bound::query(&els_out).handle().eq(&y.handle());

        SwitchSelectElseConstPat::new(pred, a, ctx, ty, thn_ctx, y, if_out)
            .assert(ctx_of_if)
            .assert(has_arg_ty)
            .assert(if_out_matches)
            .assert(a_matches)
            .assert(same_thn_index)
            .assert(same_els_index)
            .assert(same_arg_index)
            .assert(thn_ctx_matches)
            .assert(thn_matches_arg)
            .assert(lo_bound)
            .assert(hi_bound)
    }

    #[eggplant::pat_vars]
    struct SwitchSelectThenConstPat<PR: PatRecSgl> {
        pred: schema_dsl::Expr,
        b: schema_dsl::Expr,
        ctx: schema_dsl::Assumption,
        ty: schema_dsl::Type,
        els_ctx: schema_dsl::Assumption,
        y: IntB,
        if_out: schema_dsl::Expr,
    }

    fn switch_select_then_const_pat<PR: PatRecSgl>() -> SwitchSelectThenConstPat<PR> {
        let pred = schema_dsl::Expr::query_leaf();
        let inputs = schema_dsl::Expr::query_leaf();
        let thn = schema_dsl::Expr::query_leaf();
        let els = schema_dsl::Expr::query_leaf();
        let if_e = schema_dsl::If::query(&pred, &inputs, &thn, &els);
        let if_out = schema_dsl::Expr::query_leaf();
        let b = schema_dsl::Expr::query_leaf();
        let ctx = schema_dsl::Assumption::query_leaf();
        let ty = schema_dsl::Type::query_leaf();
        let thn_out = schema_dsl::Get::query(&thn);
        let els_out = schema_dsl::Get::query(&els);
        let els_ctx = schema_dsl::Assumption::query_leaf();
        let branch_ty = schema_dsl::Type::query_leaf();
        let y = IntB::query();
        let if_out_get = schema_dsl::Get::query(&if_e);
        let b_get = schema_dsl::Get::query(&inputs);
        let els_arg_out = schema_dsl::Get::query(&schema_dsl::Arg::query(&branch_ty, &els_ctx));
        let ctx_of_if = eggplant::wrap::FactCallConstraint {
            op: "ContextOf",
            operands: vec![
                if_e.handle().into_handle_ty(),
                ctx.handle().into_handle_ty(),
            ],
        };
        let has_arg_ty = eggplant::wrap::FactCallConstraint {
            op: "HasArgType",
            operands: vec![if_e.handle().into_handle_ty(), ty.handle().into_handle_ty()],
        };
        let if_out_matches = if_out.handle().eq(&if_out_get.handle());
        let b_matches = b.handle().eq(&b_get.handle());
        let same_thn_index = if_out_get.handle_index().eq(&thn_out.handle_index());
        let same_els_index = if_out_get.handle_index().eq(&els_out.handle_index());
        let same_arg_index = b_get.handle_index().eq(&els_arg_out.handle_index());
        let lo_bound = lo_bound::query(&thn_out).handle().eq(&y.handle());
        let hi_bound = hi_bound::query(&thn_out).handle().eq(&y.handle());
        let els_ctx_matches = els_ctx.handle().eq(&prim_call::<schema_dsl::Assumption>(
            "InIf",
            vec![
                (&false).into_handle_ty(),
                pred.handle().into_handle_ty(),
                inputs.handle().into_handle_ty(),
            ],
        ));
        let els_matches_arg = els_out.handle().eq(&els_arg_out.handle());

        SwitchSelectThenConstPat::new(pred, b, ctx, ty, els_ctx, y, if_out)
            .assert(ctx_of_if)
            .assert(has_arg_ty)
            .assert(if_out_matches)
            .assert(b_matches)
            .assert(same_thn_index)
            .assert(same_els_index)
            .assert(same_arg_index)
            .assert(lo_bound)
            .assert(hi_bound)
            .assert(els_ctx_matches)
            .assert(els_matches_arg)
    }

    #[eggplant::pat_vars]
    struct SwitchAndPat<PR: PatRecSgl> {
        lhs: schema_dsl::If,
        a: schema_dsl::Expr,
        b: schema_dsl::Expr,
        ins: schema_dsl::Expr,
        x: schema_dsl::Expr,
        y: schema_dsl::Expr,
        ins_ty: schema_dsl::TypeList,
    }

    fn switch_and_pat<PR: PatRecSgl>() -> SwitchAndPat<PR> {
        let a = schema_dsl::Expr::query_leaf();
        let b = schema_dsl::Expr::query_leaf();
        let ins = schema_dsl::Expr::query_leaf();
        let x = schema_dsl::Expr::query_leaf();
        let y = schema_dsl::Expr::query_leaf();
        let lhs = schema_dsl::If::query(
            &schema_dsl::Bop::query(&schema_dsl::And::query(), &a, &b),
            &ins,
            &x,
            &y,
        );
        let ins_ty = schema_dsl::TypeList::query_leaf();
        let switch_and_len = BaseVar::<i64, PR>::query_named("switch_and_len");

        let ins_has_type = eggplant::wrap::FactCallConstraint {
            op: "HasType",
            operands: vec![
                ins.handle().into_handle_ty(),
                schema_dsl::TupleT::query(&ins_ty).handle().into_handle_ty(),
            ],
        };
        let tuple_len_known = switch_and_len.handle().eq(&prim_call::<i64>(
            "tuple-length",
            vec![ins.handle().into_handle_ty()],
        ));
        let rhs_small =
            prim_call::<i64>("Expr-size", vec![y.handle().into_handle_ty()]).lt(&100_i64);

        SwitchAndPat::new(lhs, a, b, ins, x, y, ins_ty)
            .assert(ins_has_type)
            .assert(tuple_len_known)
            .assert(rhs_small)
    }

    #[eggplant::pat_vars]
    struct SwitchOrPat<PR: PatRecSgl> {
        lhs: schema_dsl::If,
        a: schema_dsl::Expr,
        b: schema_dsl::Expr,
        ins: schema_dsl::Expr,
        x: schema_dsl::Expr,
        y: schema_dsl::Expr,
        ins_ty: schema_dsl::TypeList,
    }

    fn switch_or_pat<PR: PatRecSgl>() -> SwitchOrPat<PR> {
        let a = schema_dsl::Expr::query_leaf();
        let b = schema_dsl::Expr::query_leaf();
        let ins = schema_dsl::Expr::query_leaf();
        let x = schema_dsl::Expr::query_leaf();
        let y = schema_dsl::Expr::query_leaf();
        let lhs = schema_dsl::If::query(
            &schema_dsl::Bop::query(&schema_dsl::Or::query(), &a, &b),
            &ins,
            &x,
            &y,
        );
        let ins_ty = schema_dsl::TypeList::query_leaf();
        let switch_or_len = BaseVar::<i64, PR>::query_named("switch_or_len");

        let ins_has_type = eggplant::wrap::FactCallConstraint {
            op: "HasType",
            operands: vec![
                ins.handle().into_handle_ty(),
                schema_dsl::TupleT::query(&ins_ty).handle().into_handle_ty(),
            ],
        };
        let tuple_len_known = switch_or_len.handle().eq(&prim_call::<i64>(
            "tuple-length",
            vec![ins.handle().into_handle_ty()],
        ));
        let lhs_small =
            prim_call::<i64>("Expr-size", vec![x.handle().into_handle_ty()]).lt(&100_i64);
        let rhs_small =
            prim_call::<i64>("Expr-size", vec![y.handle().into_handle_ty()]).lt(&100_i64);

        SwitchOrPat::new(lhs, a, b, ins, x, y, ins_ty)
            .assert(ins_has_type)
            .assert(tuple_len_known)
            .assert(lhs_small)
            .assert(rhs_small)
    }

    pub(crate) fn register_native_rules() -> RuleSetId {
        let ruleset = PeepholeTx::new_ruleset("switch_rewrite");

        PeepholeTx::add_rule("switch_min", ruleset, switch_min_pat, |ctx, pat| {
            let op = insert_call::<schema_dsl::BinaryOp>(&ctx.ctx, "Smin", &[]);
            let rhs =
                insert_call::<schema_dsl::Expr>(&ctx.ctx, "Bop", &[op.0.val, pat.a.val, pat.b.val]);
            ctx.union(pat.if_out, rhs);
        });

        PeepholeTx::add_rule("switch_max", ruleset, switch_max_pat, |ctx, pat| {
            let op = insert_call::<schema_dsl::BinaryOp>(&ctx.ctx, "Smax", &[]);
            let rhs =
                insert_call::<schema_dsl::Expr>(&ctx.ctx, "Bop", &[op.0.val, pat.a.val, pat.b.val]);
            ctx.union(pat.if_out, rhs);
        });

        PeepholeTx::add_rule(
            "switch_select_from_if",
            ruleset,
            switch_select_pat,
            |ctx, pat| {
                let op = insert_call::<schema_dsl::TernaryOp>(&ctx.ctx, "Select", &[]);
                let rhs = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "Top",
                    &[op.0.val, pat.pred.val, pat.a.val, pat.b.val],
                );
                ctx.union(pat.if_out, rhs);
            },
        );

        PeepholeTx::add_rule(
            "switch_select_from_const_branches",
            ruleset,
            switch_select_const_pat,
            |ctx, pat| {
                let lhs = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "Const",
                    &[pat.x.val, pat.ty.val, pat.ctx.val],
                );
                let rhs = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "Const",
                    &[pat.y.val, pat.ty.val, pat.ctx.val],
                );
                let op = insert_call::<schema_dsl::TernaryOp>(&ctx.ctx, "Select", &[]);
                let result = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "Top",
                    &[op.0.val, pat.pred.val, lhs.0.val, rhs.0.val],
                );
                ctx.union(pat.if_out, result);
            },
        );

        PeepholeTx::add_rule(
            "switch_select_with_else_const",
            ruleset,
            switch_select_else_const_pat,
            |ctx, pat| {
                let constant =
                    insert_call::<schema_dsl::Constant>(&ctx.ctx, "Int", &[pat.y.value.val]);
                let rhs_const = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "Const",
                    &[constant.0.val, pat.ty.val, pat.ctx.val],
                );
                let op = insert_call::<schema_dsl::TernaryOp>(&ctx.ctx, "Select", &[]);
                let rhs = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "Top",
                    &[op.0.val, pat.pred.val, pat.a.val, rhs_const.0.val],
                );
                ctx.union(pat.if_out, rhs);
            },
        );

        PeepholeTx::add_rule(
            "switch_select_with_then_const",
            ruleset,
            switch_select_then_const_pat,
            |ctx, pat| {
                let constant =
                    insert_call::<schema_dsl::Constant>(&ctx.ctx, "Int", &[pat.y.value.val]);
                let lhs_const = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "Const",
                    &[constant.0.val, pat.ty.val, pat.ctx.val],
                );
                let op = insert_call::<schema_dsl::TernaryOp>(&ctx.ctx, "Select", &[]);
                let rhs = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "Top",
                    &[op.0.val, pat.pred.val, lhs_const.0.val, pat.b.val],
                );
                ctx.union(pat.if_out, rhs);
            },
        );

        PeepholeTx::add_rule(
            "switch_and_reassociate",
            ruleset,
            switch_and_pat,
            |ctx, pat| {
                let len = insert_call::<i64>(&ctx.ctx, "tuple-length", &[pat.ins.val]);
                let single_b = insert_call::<schema_dsl::Expr>(&ctx.ctx, "Single", &[pat.b.val]);
                let outer_ins = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "Concat",
                    &[single_b.0.val, pat.ins.val],
                );
                let bool_ty = insert_call::<schema_dsl::BaseType>(&ctx.ctx, "BoolT", &[]);
                let outer_ins_ty_list = insert_call::<schema_dsl::TypeList>(
                    &ctx.ctx,
                    "TCons",
                    &[bool_ty.0.val, pat.ins_ty.val],
                );
                let outer_ins_ty =
                    insert_call::<schema_dsl::Type>(&ctx.ctx, "TupleT", &[outer_ins_ty_list.0.val]);
                let if_true = insert_call::<schema_dsl::Assumption>(
                    &ctx.ctx,
                    "InIf",
                    &[true.to_value(&ctx.ctx).val, pat.a.val, outer_ins.0.val],
                );
                let if_false = insert_call::<schema_dsl::Assumption>(
                    &ctx.ctx,
                    "InIf",
                    &[false.to_value(&ctx.ctx).val, pat.a.val, outer_ins.0.val],
                );
                let arg_true = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "Arg",
                    &[outer_ins_ty.0.val, if_true.0.val],
                );
                let arg_false = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "Arg",
                    &[outer_ins_ty.0.val, if_false.0.val],
                );
                let inner_pred = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "Get",
                    &[arg_true.0.val, 0_i64.to_value(&ctx.ctx).val],
                );
                let sub_arg_true = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "SubTuple",
                    &[arg_true.0.val, 1_i64.to_value(&ctx.ctx).val, len.0.val],
                );
                let sub_arg_false = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "SubTuple",
                    &[arg_false.0.val, 1_i64.to_value(&ctx.ctx).val, len.0.val],
                );
                let inner_false_ctx = insert_call::<schema_dsl::Assumption>(
                    &ctx.ctx,
                    "InIf",
                    &[
                        false.to_value(&ctx.ctx).val,
                        inner_pred.0.val,
                        sub_arg_true.0.val,
                    ],
                );
                let inner_y = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "AddContext",
                    &[inner_false_ctx.0.val, pat.y.val],
                );
                let outer_y = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "Subst",
                    &[if_false.0.val, sub_arg_false.0.val, pat.y.val],
                );
                let inner = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "If",
                    &[
                        inner_pred.0.val,
                        sub_arg_true.0.val,
                        pat.x.val,
                        inner_y.0.val,
                    ],
                );
                let outer = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "If",
                    &[pat.a.val, outer_ins.0.val, inner.0.val, outer_y.0.val],
                );
                ctx.union(pat.lhs, outer);
            },
        );

        PeepholeTx::add_rule(
            "switch_or_reassociate",
            ruleset,
            switch_or_pat,
            |ctx, pat| {
                let len = insert_call::<i64>(&ctx.ctx, "tuple-length", &[pat.ins.val]);
                let single_b = insert_call::<schema_dsl::Expr>(&ctx.ctx, "Single", &[pat.b.val]);
                let outer_ins = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "Concat",
                    &[single_b.0.val, pat.ins.val],
                );
                let bool_ty = insert_call::<schema_dsl::BaseType>(&ctx.ctx, "BoolT", &[]);
                let outer_ins_ty_list = insert_call::<schema_dsl::TypeList>(
                    &ctx.ctx,
                    "TCons",
                    &[bool_ty.0.val, pat.ins_ty.val],
                );
                let outer_ins_ty =
                    insert_call::<schema_dsl::Type>(&ctx.ctx, "TupleT", &[outer_ins_ty_list.0.val]);
                let if_true = insert_call::<schema_dsl::Assumption>(
                    &ctx.ctx,
                    "InIf",
                    &[true.to_value(&ctx.ctx).val, pat.a.val, outer_ins.0.val],
                );
                let if_false = insert_call::<schema_dsl::Assumption>(
                    &ctx.ctx,
                    "InIf",
                    &[false.to_value(&ctx.ctx).val, pat.a.val, outer_ins.0.val],
                );
                let arg_true = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "Arg",
                    &[outer_ins_ty.0.val, if_true.0.val],
                );
                let arg_false = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "Arg",
                    &[outer_ins_ty.0.val, if_false.0.val],
                );
                let inner_pred = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "Get",
                    &[arg_false.0.val, 0_i64.to_value(&ctx.ctx).val],
                );
                let sub_arg_true = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "SubTuple",
                    &[arg_true.0.val, 1_i64.to_value(&ctx.ctx).val, len.0.val],
                );
                let sub_arg_false = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "SubTuple",
                    &[arg_false.0.val, 1_i64.to_value(&ctx.ctx).val, len.0.val],
                );
                let outer_x = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "Subst",
                    &[if_true.0.val, sub_arg_true.0.val, pat.x.val],
                );
                let inner_true_ctx = insert_call::<schema_dsl::Assumption>(
                    &ctx.ctx,
                    "InIf",
                    &[
                        true.to_value(&ctx.ctx).val,
                        inner_pred.0.val,
                        sub_arg_false.0.val,
                    ],
                );
                let inner_false_ctx = insert_call::<schema_dsl::Assumption>(
                    &ctx.ctx,
                    "InIf",
                    &[
                        false.to_value(&ctx.ctx).val,
                        inner_pred.0.val,
                        sub_arg_false.0.val,
                    ],
                );
                let inner_x = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "AddContext",
                    &[inner_true_ctx.0.val, pat.x.val],
                );
                let inner_y = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "AddContext",
                    &[inner_false_ctx.0.val, pat.y.val],
                );
                let inner = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "If",
                    &[
                        inner_pred.0.val,
                        sub_arg_false.0.val,
                        inner_x.0.val,
                        inner_y.0.val,
                    ],
                );
                let outer = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "If",
                    &[pat.a.val, outer_ins.0.val, outer_x.0.val, inner.0.val],
                );
                ctx.union(pat.lhs, outer);
            },
        );

        ruleset
    }
}

#[cfg(all(test, feature = "eggplant"))]
mod native_tests {
    use crate::eggplant_backend::test_lock;
    use egglog::{ast::Expr as EgglogExpr, EGraph, TermDag};

    fn eval_and_extract_expr(prologue: &str, expr: &str, schedule: &str) -> String {
        let binding = "__switch_rewrite_expr";
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
        let binding = "__switch_rewrite_native_expr";
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

    #[test]
    fn native_feature_path_matches_text_backend_for_switch_min_case() {
        let _guard = test_lock::lock();

        let tuple_ty = "(TupleT (TCons (IntT) (TCons (IntT) (TNil))))";
        let left = "(Const (Int 3) (Base (IntT)) (InFunc \"RLCR\"))";
        let right = "(Const (Int 5) (Base (IntT)) (InFunc \"RLCR\"))";
        let pred = format!("(Bop (LessThan) {left} {right})");
        let inputs = format!("(Concat (Single {left}) (Single {right}))");
        let then_branch = format!("(Single (Get (Arg {tuple_ty} (InIf true {pred} {inputs})) 0))");
        let else_branch = format!("(Single (Get (Arg {tuple_ty} (InIf false {pred} {inputs})) 1))");
        let expr = format!("(Get (If {pred} {inputs} {then_branch} {else_branch}) 0)");
        let schedule = format!(
            "(run-schedule\n{}\n(saturate switch_rewrite)\n)",
            crate::schedule::types_and_indexing()
        );

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
    fn ablating_switch_rewrite_changes_feature_native_result() {
        let _guard = test_lock::lock();

        let tuple_ty = "(TupleT (TCons (IntT) (TCons (IntT) (TNil))))";
        let left = "(Const (Int 3) (Base (IntT)) (InFunc \"RLCR\"))";
        let right = "(Const (Int 5) (Base (IntT)) (InFunc \"RLCR\"))";
        let pred = format!("(Bop (LessThan) {left} {right})");
        let inputs = format!("(Concat (Single {left}) (Single {right}))");
        let then_branch = format!("(Single (Get (Arg {tuple_ty} (InIf true {pred} {inputs})) 0))");
        let else_branch = format!("(Single (Get (Arg {tuple_ty} (InIf false {pred} {inputs})) 1))");
        let expr = format!("(Get (If {pred} {inputs} {then_branch} {else_branch}) 0)");
        let schedule = format!(
            "(run-schedule\n{}\n(saturate switch_rewrite)\n)",
            crate::schedule::types_and_indexing()
        );

        let simplified = eval_and_extract_native_expr(
            &crate::feature_execution_prologue(true, None),
            &expr,
            &schedule,
            None,
        );
        let ablated = eval_and_extract_native_expr(
            &crate::feature_execution_prologue(true, Some("switch_rewrite")),
            &expr,
            &crate::ablate_schedule(&schedule, "switch_rewrite"),
            Some("switch_rewrite"),
        );

        assert_ne!(ablated, simplified);
    }

    #[test]
    fn ablating_switch_rewrite_still_allows_default_feature_schedule() {
        let _guard = test_lock::lock();

        let expr = "(Const (Int 7) (Base (IntT)) (InFunc \"RLCR\"))";
        let schedule = format!(
            "(run-schedule\n{}\n(saturate always-switch-rewrite)\n)",
            crate::schedule::types_and_indexing()
        );

        let _ = eval_and_extract_native_expr(
            &crate::feature_execution_prologue(true, Some("switch_rewrite")),
            expr,
            &schedule,
            Some("switch_rewrite"),
        );
    }
}
