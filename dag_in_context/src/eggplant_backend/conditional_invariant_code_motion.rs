pub(crate) fn fragment() -> String {
    let mut out = String::new();
    out.push_str(GENERATED_MARKER);
    out.push_str(CONDITIONAL_INVARIANT_CODE_MOTION);
    out.push('\n');
    out
}

#[cfg(feature = "eggplant")]
pub(crate) fn native_fragment() -> String {
    let mut out = String::new();
    out.push_str(GENERATED_MARKER);
    out.push_str(CONDITIONAL_INVARIANT_CODE_MOTION_SUPPORT);
    out.push('\n');
    out
}

const GENERATED_MARKER: &str =
    "; (Generated from eggplant Rust: src/eggplant_backend/conditional_invariant_code_motion.rs)\n";
const CONDITIONAL_INVARIANT_CODE_MOTION_SUPPORT: &str = r#"(ruleset cicm)
(ruleset cicm-index)

(relation InvCodeMotionCandidate (Expr Expr))

;; speeds up the InvCodeMotionCandidate relation by removing indirection
(relation ExtractedExprCache (Term Expr Assumption))"#;
const CONDITIONAL_INVARIANT_CODE_MOTION: &str = r#"(ruleset cicm)
(ruleset cicm-index)

(relation InvCodeMotionCandidate (Expr Expr))

;; speeds up the InvCodeMotionCandidate relation by removing indirection
(relation ExtractedExprCache (Term Expr Assumption))

(rule ((= (TCPair t1 c1) (ExtractedExpr e1))
       (ContextOf e1 ctx1))
      ((ExtractedExprCache t1 e1 ctx1))
      :ruleset cicm-index)

(rule (
        (ExtractedExprCache t1 e1 (InIf true pred1 orig_ins3))
        (ExtractedExprCache t1 e2 (InIf false pred2 orig_ins4))
        (!= e1 e2)
     )
     ((InvCodeMotionCandidate e1 e2))
     :ruleset cicm-index)


(rule (
        (= if_e (If pred orig_ins thn els))
        (HasArgType thn (TupleT tylist))
        (HasArgType els (TupleT tylist))
        (ContextOf if_e outer_ctx)

        (= e1 (Uop o x))
        (HasType e1 (Base ty))
        (= (TCPair t1 c1) (ExtractedExpr e1))
        (> 10 (expr_size e1))
        (ExprIsPure e1)
        (ContextOf e1 (InIf true pred orig_ins))

        (= e2 (Uop o y))
        (HasType e2 (Base ty))
        (= (TCPair t2 c2) (ExtractedExpr e2))
        (> 10 (expr_size e2))
        (ExprIsPure e2)
        (ContextOf e2 (InIf false pred orig_ins))

        (= t1 t2)
        (= orig_ins_len (TypeList-length tylist))
      )
      (
        ; pull the term out to the outer context
        (let new_term (TermSubst outer_ctx orig_ins t1)) 
        
        ; Add it as an input to the new if
        (let new_ins (Concat orig_ins (Single new_term)))
        (let new_ins_ty (TupleT (TLConcat tylist (TCons ty (TNil)))))

        ; New contexts
        (let if_tr (InIf true  pred new_ins))
        (let if_fa (InIf false pred new_ins))

        ; SubTuple- this is the sublist of the new inputs that corresponds
        ; to the original inputs (without the pulled-out input)
        (let st_tr (SubTuple (Arg new_ins_ty if_tr) 0 orig_ins_len))
        (let st_fa (SubTuple (Arg new_ins_ty if_fa) 0 orig_ins_len))

        ; New regions
        (let new_thn (Subst if_tr st_tr thn))
        (let new_els (Subst if_fa st_fa els))
        
        ; Union the new arg with the original expr in each branch
        (union (Get (Arg new_ins_ty if_tr) orig_ins_len) (Subst if_tr st_tr e1))
        (union (Get (Arg new_ins_ty if_fa) orig_ins_len) (Subst if_fa st_fa e2))
        
        ; Subsume the original exprs now that the new arg is there
        ; Doing this prevents us from pulling the same exprs out of the new if
        ; Can only subsume an e-node (not an e-class), and we don't want to
        ; subsume the Subst node directly, since it won't have a chance to do
        ; the actual substitution, so manually compute the first round of
        ; substitution so that we can subsume the Uop e-nodes.
        ; First construct the Uop, so that it exists in the e-graph, because
        ; you can't subsume things that don't exist in the e-graph already.
        (Uop o (Subst if_tr st_tr x))
        (Uop o (Subst if_fa st_fa y))
        ; Now subsume:
        (subsume (Uop o (Subst if_tr st_tr x)))
        (subsume (Uop o (Subst if_fa st_fa y)))

        ; Create new if and union it with the original
        (union if_e (If pred new_ins new_thn new_els))
      )
    :ruleset cicm)

       

(rule (
        (InvCodeMotionCandidate e1 e2)
        (= if_e (If pred orig_ins thn els))
        (HasArgType thn (TupleT tylist))
        (HasArgType els (TupleT tylist))
        (ContextOf if_e outer_ctx)

        (ContextOf e1 (InIf true pred orig_ins))
        (ContextOf e2 (InIf false pred orig_ins))
        (= e1 (Bop o x1 y1))
        
        (= e2 (Bop o x2 y2))
        
        (= (TCPair t1 c1) (ExtractedExpr e1))
        (> 10 (expr_size e1))
        (ExprIsPure e1)
        (HasType e1 (Base ty))
        

        
        (HasType e2 (Base ty))
        (= (TCPair t2 c2) (ExtractedExpr e2))
        (> 10 (expr_size e2))
        (ExprIsPure e2)

        (= t1 t2)
        (= orig_ins_len (TypeList-length tylist))
      )
      (
        ; pull the term out to the outer context
        (let new_term (TermSubst outer_ctx orig_ins t1)) 
        
        ; Add it as an input to the new if
        (let new_ins (Concat orig_ins (Single new_term)))
        (let new_ins_ty (TupleT (TLConcat tylist (TCons ty (TNil)))))

        ; New contexts
        (let if_tr (InIf true  pred new_ins))
        (let if_fa (InIf false pred new_ins))

        ; SubTuple- this is the sublist of the new inputs that corresponds
        ; to the original inputs (without the pulled-out input)
        (let st_tr (SubTuple (Arg new_ins_ty if_tr) 0 orig_ins_len))
        (let st_fa (SubTuple (Arg new_ins_ty if_fa) 0 orig_ins_len))

        ; New regions
        (let new_thn (Subst if_tr st_tr thn))
        (let new_els (Subst if_fa st_fa els))
        
        ; Union the new arg with the original expr in each branch
        (union (Get (Arg new_ins_ty if_tr) orig_ins_len) (Subst if_tr st_tr e1))
        (union (Get (Arg new_ins_ty if_fa) orig_ins_len) (Subst if_fa st_fa e2))
        
        ; Subsume the original exprs now that the new arg is there
        ; Doing this prevents us from pulling the same exprs out of the new if
        ; Can only subsume an e-node (not an e-class), and we don't want to
        ; subsume the Subst node directly, since it won't have a chance to do
        ; the actual substitution, so manually compute the first round of
        ; substitution so that we can subsume the Uop e-nodes.
        ; First construct the Uop, so that it exists in the e-graph, because
        ; you can't subsume things that don't exist in the e-graph already.
        (Bop o (Subst if_tr st_tr x1) (Subst if_tr st_tr y1))
        (Bop o (Subst if_fa st_fa x2) (Subst if_fa st_fa y2))
        ; Now subsume:
        (subsume (Bop o (Subst if_tr st_tr x1) (Subst if_tr st_tr y1)))
        (subsume (Bop o (Subst if_fa st_fa x2) (Subst if_fa st_fa y2)))

        ; Create new if and union it with the original
        (union if_e (If pred new_ins new_thn new_els))
      )
    :ruleset cicm)"#;

#[cfg(feature = "eggplant")]
pub(crate) mod native {
    use super::super::schema_dsl;
    use super::super::schema_dsl::{ExtractedExprCachePRRuleCtx, InvCodeMotionCandidatePRRuleCtx};
    use crate::eggplant_backend::peepholes::native::PeepholeTx;
    use eggplant::prelude::{
        prim_call, AsHandle, BaseVar, Compare, Insertable, IntoHandleTy, PEq, PatRecSgl,
        RuleRunnerSgl, RuleSetId,
    };
    use eggplant::wrap::EgglogTy;

    #[derive(Clone, Copy, Debug)]
    struct TermAndCostTy;

    impl EgglogTy for TermAndCostTy {
        const TY_NAME: &'static str = "TermAndCost";
        const TY_NAME_LOWER: &'static str = "term_and_cost";
        type Valued = eggplant::wrap::Value<Self>;
        type EnumVariantMarker = ();
    }

    #[eggplant::pat_vars]
    struct CicmIndexExtractedExprCachePat<PR: PatRecSgl> {
        t1: schema_dsl::Term,
        e1: schema_dsl::Expr,
        ctx1: schema_dsl::Assumption,
        context_of: schema_dsl::ContextOf,
    }

    fn cicm_index_extracted_expr_cache_pat<PR: PatRecSgl>() -> CicmIndexExtractedExprCachePat<PR> {
        let t1 = schema_dsl::Term::query_leaf();
        let e1 = schema_dsl::Expr::query_leaf();
        let ctx1 = schema_dsl::Assumption::query_leaf();
        let c1 = BaseVar::<i64, PR>::query_named("c1");

        let extracted_expr = prim_call::<TermAndCostTy>(
            "TCPair",
            vec![t1.handle().into_handle_ty(), c1.handle().into_handle_ty()],
        )
        .eq(&prim_call::<TermAndCostTy>(
            "ExtractedExpr",
            vec![e1.handle().into_handle_ty()],
        ));
        let context_of = schema_dsl::ContextOf::query_fields(&e1, &ctx1);

        CicmIndexExtractedExprCachePat::new(t1, e1, ctx1, context_of).assert(extracted_expr)
    }

    #[eggplant::pat_vars]
    struct CicmIndexCandidatePat<PR: PatRecSgl> {
        t1: schema_dsl::Term,
        e1: schema_dsl::Expr,
        e2: schema_dsl::Expr,
        true_branch_cache: schema_dsl::ExtractedExprCache,
        false_branch_cache: schema_dsl::ExtractedExprCache,
    }

    fn cicm_index_candidate_pat<PR: PatRecSgl>() -> CicmIndexCandidatePat<PR> {
        let t1 = schema_dsl::Term::query_leaf();
        let e1 = schema_dsl::Expr::query_leaf();
        let e2 = schema_dsl::Expr::query_leaf();
        let pred1 = schema_dsl::Expr::query_leaf();
        let pred2 = schema_dsl::Expr::query_leaf();
        let orig_ins3 = schema_dsl::Expr::query_leaf();
        let orig_ins4 = schema_dsl::Expr::query_leaf();
        let true_if_ctx = schema_dsl::InIf::query(&pred1, &orig_ins3);
        let false_if_ctx = schema_dsl::InIf::query(&pred2, &orig_ins4);
        let true_branch_cache =
            schema_dsl::ExtractedExprCache::query_fields(&t1, &e1, &true_if_ctx);
        let false_branch_cache =
            schema_dsl::ExtractedExprCache::query_fields(&t1, &e2, &false_if_ctx);
        let distinct_exprs = e1.handle().ne(&e2.handle());
        let true_branch_flag = true_if_ctx.handle_pred_is_true().eq(&(&true).as_handle());
        let false_branch_flag = false_if_ctx.handle_pred_is_true().eq(&(&false).as_handle());

        CicmIndexCandidatePat::new(t1, e1, e2, true_branch_cache, false_branch_cache)
            .assert(true_branch_flag)
            .assert(false_branch_flag)
            .assert(distinct_exprs)
    }

    #[eggplant::pat_vars]
    struct CicmUopPat<PR: PatRecSgl> {
        if_e: schema_dsl::Expr,
        pred: schema_dsl::Expr,
        orig_ins: schema_dsl::Expr,
        thn: schema_dsl::Expr,
        els: schema_dsl::Expr,
        then_has_arg_type: schema_dsl::HasArgType,
        else_has_arg_type: schema_dsl::HasArgType,
        outer_ctx: schema_dsl::Assumption,
        if_context: schema_dsl::ContextOf,
        tylist: schema_dsl::TypeList,
        ty: schema_dsl::BaseType,
        e1: schema_dsl::Expr,
        e2: schema_dsl::Expr,
        op: schema_dsl::UnaryOp,
        x: schema_dsl::Expr,
        y: schema_dsl::Expr,
        e1_has_type: schema_dsl::HasType,
        e1_pure: schema_dsl::ExprIsPure,
        e1_context: schema_dsl::ContextOf,
        e2_has_type: schema_dsl::HasType,
        e2_pure: schema_dsl::ExprIsPure,
        e2_context: schema_dsl::ContextOf,
        t1: schema_dsl::Term,
    }

    fn cicm_uop_pat<PR: PatRecSgl>() -> CicmUopPat<PR> {
        let if_e = schema_dsl::Expr::query_leaf();
        let pred = schema_dsl::Expr::query_leaf();
        let orig_ins = schema_dsl::Expr::query_leaf();
        let thn = schema_dsl::Expr::query_leaf();
        let els = schema_dsl::Expr::query_leaf();
        let outer_ctx = schema_dsl::Assumption::query_leaf();
        let tylist = schema_dsl::TypeList::query_leaf();
        let ty = schema_dsl::BaseType::query_leaf();
        let e1 = schema_dsl::Expr::query_leaf();
        let e2 = schema_dsl::Expr::query_leaf();
        let op = schema_dsl::UnaryOp::query_leaf();
        let x = schema_dsl::Expr::query_leaf();
        let y = schema_dsl::Expr::query_leaf();
        let t1 = schema_dsl::Term::query_leaf();
        let size1 = BaseVar::<i64, PR>::query_named("size1");
        let size2 = BaseVar::<i64, PR>::query_named("size2");
        let c1 = BaseVar::<i64, PR>::query_named("c1");
        let c2 = BaseVar::<i64, PR>::query_named("c2");

        let tuple_ty = schema_dsl::TupleT::query(&tylist);
        let base_ty = schema_dsl::Base::query(&ty);
        let true_if_ctx = schema_dsl::InIf::query(&pred, &orig_ins);
        let false_if_ctx = schema_dsl::InIf::query(&pred, &orig_ins);

        let if_match = if_e
            .handle()
            .eq(&schema_dsl::If::query(&pred, &orig_ins, &thn, &els).handle());
        let then_has_arg_type = schema_dsl::HasArgType::query_fields(&thn, &tuple_ty);
        let else_has_arg_type = schema_dsl::HasArgType::query_fields(&els, &tuple_ty);
        let if_context = schema_dsl::ContextOf::query_fields(&if_e, &outer_ctx);
        let e1_match = e1.handle().eq(&schema_dsl::Uop::query(&op, &x).handle());
        let e1_has_type = schema_dsl::HasType::query_fields(&e1, &base_ty);
        let e1_size = size1
            .handle()
            .eq(&crate::eggplant_backend::expr_size::native::expr_size::query(&e1).handle());
        let e1_small = size1.handle().lt(&10_i64);
        let e1_pure = schema_dsl::ExprIsPure::query_fields(&e1);
        let e1_context = schema_dsl::ContextOf::query_fields(&e1, &true_if_ctx);
        let e1_extracted = prim_call::<TermAndCostTy>(
            "TCPair",
            vec![t1.handle().into_handle_ty(), c1.handle().into_handle_ty()],
        )
        .eq(&prim_call::<TermAndCostTy>(
            "ExtractedExpr",
            vec![e1.handle().into_handle_ty()],
        ));
        let e2_match = e2.handle().eq(&schema_dsl::Uop::query(&op, &y).handle());
        let e2_has_type = schema_dsl::HasType::query_fields(&e2, &base_ty);
        let e2_size = size2
            .handle()
            .eq(&crate::eggplant_backend::expr_size::native::expr_size::query(&e2).handle());
        let e2_small = size2.handle().lt(&10_i64);
        let e2_pure = schema_dsl::ExprIsPure::query_fields(&e2);
        let e2_context = schema_dsl::ContextOf::query_fields(&e2, &false_if_ctx);
        let e2_extracted = prim_call::<TermAndCostTy>(
            "TCPair",
            vec![t1.handle().into_handle_ty(), c2.handle().into_handle_ty()],
        )
        .eq(&prim_call::<TermAndCostTy>(
            "ExtractedExpr",
            vec![e2.handle().into_handle_ty()],
        ));
        let true_branch_flag = true_if_ctx.handle_pred_is_true().eq(&(&true).as_handle());
        let false_branch_flag = false_if_ctx.handle_pred_is_true().eq(&(&false).as_handle());

        CicmUopPat::new(
            if_e,
            pred,
            orig_ins,
            thn,
            els,
            then_has_arg_type,
            else_has_arg_type,
            outer_ctx,
            if_context,
            tylist,
            ty,
            e1,
            e2,
            op,
            x,
            y,
            e1_has_type,
            e1_pure,
            e1_context,
            e2_has_type,
            e2_pure,
            e2_context,
            t1,
        )
        .assert(if_match)
        .assert(e1_match)
        .assert(e1_size)
        .assert(e1_small)
        .assert(e1_extracted)
        .assert(e2_match)
        .assert(e2_size)
        .assert(e2_small)
        .assert(e2_extracted)
        .assert(true_branch_flag)
        .assert(false_branch_flag)
    }

    #[eggplant::pat_vars]
    struct CicmBopPat<PR: PatRecSgl> {
        if_e: schema_dsl::Expr,
        pred: schema_dsl::Expr,
        orig_ins: schema_dsl::Expr,
        thn: schema_dsl::Expr,
        els: schema_dsl::Expr,
        then_has_arg_type: schema_dsl::HasArgType,
        else_has_arg_type: schema_dsl::HasArgType,
        outer_ctx: schema_dsl::Assumption,
        if_context: schema_dsl::ContextOf,
        tylist: schema_dsl::TypeList,
        ty: schema_dsl::BaseType,
        e1: schema_dsl::Expr,
        e2: schema_dsl::Expr,
        inv_candidate: schema_dsl::InvCodeMotionCandidate,
        op: schema_dsl::BinaryOp,
        x1: schema_dsl::Expr,
        y1: schema_dsl::Expr,
        x2: schema_dsl::Expr,
        y2: schema_dsl::Expr,
        e1_context: schema_dsl::ContextOf,
        e2_context: schema_dsl::ContextOf,
        e1_has_type: schema_dsl::HasType,
        e2_has_type: schema_dsl::HasType,
        e1_pure: schema_dsl::ExprIsPure,
        e2_pure: schema_dsl::ExprIsPure,
        t1: schema_dsl::Term,
    }

    fn cicm_bop_pat<PR: PatRecSgl>() -> CicmBopPat<PR> {
        let if_e = schema_dsl::Expr::query_leaf();
        let pred = schema_dsl::Expr::query_leaf();
        let orig_ins = schema_dsl::Expr::query_leaf();
        let thn = schema_dsl::Expr::query_leaf();
        let els = schema_dsl::Expr::query_leaf();
        let outer_ctx = schema_dsl::Assumption::query_leaf();
        let tylist = schema_dsl::TypeList::query_leaf();
        let ty = schema_dsl::BaseType::query_leaf();
        let e1 = schema_dsl::Expr::query_leaf();
        let e2 = schema_dsl::Expr::query_leaf();
        let op = schema_dsl::BinaryOp::query_leaf();
        let x1 = schema_dsl::Expr::query_leaf();
        let y1 = schema_dsl::Expr::query_leaf();
        let x2 = schema_dsl::Expr::query_leaf();
        let y2 = schema_dsl::Expr::query_leaf();
        let t1 = schema_dsl::Term::query_leaf();
        let size1 = BaseVar::<i64, PR>::query_named("size1");
        let size2 = BaseVar::<i64, PR>::query_named("size2");
        let c1 = BaseVar::<i64, PR>::query_named("c1");
        let c2 = BaseVar::<i64, PR>::query_named("c2");

        let tuple_ty = schema_dsl::TupleT::query(&tylist);
        let base_ty = schema_dsl::Base::query(&ty);
        let true_if_ctx = schema_dsl::InIf::query(&pred, &orig_ins);
        let false_if_ctx = schema_dsl::InIf::query(&pred, &orig_ins);

        let inv_candidate = schema_dsl::InvCodeMotionCandidate::query_fields(&e1, &e2);
        let if_match = if_e
            .handle()
            .eq(&schema_dsl::If::query(&pred, &orig_ins, &thn, &els).handle());
        let then_has_arg_type = schema_dsl::HasArgType::query_fields(&thn, &tuple_ty);
        let else_has_arg_type = schema_dsl::HasArgType::query_fields(&els, &tuple_ty);
        let if_context = schema_dsl::ContextOf::query_fields(&if_e, &outer_ctx);
        let e1_context = schema_dsl::ContextOf::query_fields(&e1, &true_if_ctx);
        let e2_context = schema_dsl::ContextOf::query_fields(&e2, &false_if_ctx);
        let e1_match = e1
            .handle()
            .eq(&schema_dsl::Bop::query(&op, &x1, &y1).handle());
        let e2_match = e2
            .handle()
            .eq(&schema_dsl::Bop::query(&op, &x2, &y2).handle());
        let e1_has_type = schema_dsl::HasType::query_fields(&e1, &base_ty);
        let e2_has_type = schema_dsl::HasType::query_fields(&e2, &base_ty);
        let e1_extracted = prim_call::<TermAndCostTy>(
            "TCPair",
            vec![t1.handle().into_handle_ty(), c1.handle().into_handle_ty()],
        )
        .eq(&prim_call::<TermAndCostTy>(
            "ExtractedExpr",
            vec![e1.handle().into_handle_ty()],
        ));
        let e2_extracted = prim_call::<TermAndCostTy>(
            "TCPair",
            vec![t1.handle().into_handle_ty(), c2.handle().into_handle_ty()],
        )
        .eq(&prim_call::<TermAndCostTy>(
            "ExtractedExpr",
            vec![e2.handle().into_handle_ty()],
        ));
        let e1_size = size1
            .handle()
            .eq(&crate::eggplant_backend::expr_size::native::expr_size::query(&e1).handle());
        let e2_size = size2
            .handle()
            .eq(&crate::eggplant_backend::expr_size::native::expr_size::query(&e2).handle());
        let e1_small = size1.handle().lt(&10_i64);
        let e2_small = size2.handle().lt(&10_i64);
        let e1_pure = schema_dsl::ExprIsPure::query_fields(&e1);
        let e2_pure = schema_dsl::ExprIsPure::query_fields(&e2);
        let true_branch_flag = true_if_ctx.handle_pred_is_true().eq(&(&true).as_handle());
        let false_branch_flag = false_if_ctx.handle_pred_is_true().eq(&(&false).as_handle());

        CicmBopPat::new(
            if_e,
            pred,
            orig_ins,
            thn,
            els,
            then_has_arg_type,
            else_has_arg_type,
            outer_ctx,
            if_context,
            tylist,
            ty,
            e1,
            e2,
            inv_candidate,
            op,
            x1,
            y1,
            x2,
            y2,
            e1_context,
            e2_context,
            e1_has_type,
            e2_has_type,
            e1_pure,
            e2_pure,
            t1,
        )
        .assert(if_match)
        .assert(e1_match)
        .assert(e2_match)
        .assert(e1_extracted)
        .assert(e2_extracted)
        .assert(e1_size)
        .assert(e2_size)
        .assert(e1_small)
        .assert(e2_small)
        .assert(true_branch_flag)
        .assert(false_branch_flag)
    }

    pub(crate) fn register_native_rules() -> RuleSetId {
        let ruleset = RuleSetId("cicm");
        let index_ruleset = RuleSetId("cicm-index");

        PeepholeTx::add_rule(
            "conditional_invariant_code_motion_index_extracted_expr_cache",
            index_ruleset,
            cicm_index_extracted_expr_cache_pat,
            |ctx, pat| {
                ctx.insert_extracted_expr_cache(pat.t1, pat.e1, pat.ctx1);
            },
        );

        PeepholeTx::add_rule(
            "conditional_invariant_code_motion_index_candidates",
            index_ruleset,
            cicm_index_candidate_pat,
            |ctx, pat| {
                ctx.insert_inv_code_motion_candidate(pat.e1, pat.e2);
            },
        );

        PeepholeTx::add_rule(
            "conditional_invariant_code_motion_uop",
            ruleset,
            cicm_uop_pat,
            |ctx, pat| {
                let zero = 0_i64.to_value(&ctx).val;
                let orig_ins_len =
                    ctx.lookup_expect("TypeList-length", &[pat.tylist.to_value(&ctx).val]);

                let new_term = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "TermSubst",
                    &[
                        pat.outer_ctx.to_value(&ctx).val,
                        pat.orig_ins.to_value(&ctx).val,
                        pat.t1.to_value(&ctx).val,
                    ],
                ));
                let new_ins = eggplant::wrap::Value::<schema_dsl::Expr>::new(
                    (&ctx).insert(
                        "Concat",
                        &[
                            pat.orig_ins.to_value(&ctx).val,
                            eggplant::wrap::Value::<schema_dsl::Expr>::new(
                                (&ctx).insert("Single", &[new_term.to_value(&ctx).val]),
                            )
                            .to_value(&ctx)
                            .val,
                        ],
                    ),
                );

                let tnil =
                    eggplant::wrap::Value::<schema_dsl::TypeList>::new((ctx).insert("TNil", &[]));
                let appended_tail = eggplant::wrap::Value::<schema_dsl::TypeList>::new(
                    (&ctx).insert("TCons", &[pat.ty.to_value(&ctx).val, tnil.val]),
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
                    (&ctx).insert("SubTuple", &[arg_tr.to_value(&ctx).val, zero, orig_ins_len]),
                );
                let st_fa = eggplant::wrap::Value::<schema_dsl::Expr>::new(
                    (&ctx).insert("SubTuple", &[arg_fa.to_value(&ctx).val, zero, orig_ins_len]),
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

                let pulled_in_true = eggplant::wrap::Value::<schema_dsl::Expr>::new(
                    (&ctx).insert("Get", &[arg_tr.to_value(&ctx).val, orig_ins_len]),
                );
                let pulled_in_false = eggplant::wrap::Value::<schema_dsl::Expr>::new(
                    (&ctx).insert("Get", &[arg_fa.to_value(&ctx).val, orig_ins_len]),
                );
                let subst_e1 = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "Subst",
                    &[
                        if_tr.to_value(&ctx).val,
                        st_tr.to_value(&ctx).val,
                        pat.e1.to_value(&ctx).val,
                    ],
                ));
                let subst_e2 = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "Subst",
                    &[
                        if_fa.to_value(&ctx).val,
                        st_fa.to_value(&ctx).val,
                        pat.e2.to_value(&ctx).val,
                    ],
                ));
                ctx.union(pulled_in_true, subst_e1);
                ctx.union(pulled_in_false, subst_e2);

                let subst_x = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "Subst",
                    &[
                        if_tr.to_value(&ctx).val,
                        st_tr.to_value(&ctx).val,
                        pat.x.to_value(&ctx).val,
                    ],
                ));
                let subst_y = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "Subst",
                    &[
                        if_fa.to_value(&ctx).val,
                        st_fa.to_value(&ctx).val,
                        pat.y.to_value(&ctx).val,
                    ],
                ));
                let _ = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "Uop",
                    &[pat.op.to_value(&ctx).val, subst_x.to_value(&ctx).val],
                ));
                let _ = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "Uop",
                    &[pat.op.to_value(&ctx).val, subst_y.to_value(&ctx).val],
                ));
                ctx.subsume(
                    "Uop",
                    &[pat.op.to_value(&ctx).val, subst_x.to_value(&ctx).val],
                );
                ctx.subsume(
                    "Uop",
                    &[pat.op.to_value(&ctx).val, subst_y.to_value(&ctx).val],
                );

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

        PeepholeTx::add_rule(
            "conditional_invariant_code_motion_bop",
            ruleset,
            cicm_bop_pat,
            |ctx, pat| {
                let zero = 0_i64.to_value(&ctx).val;
                let orig_ins_len =
                    ctx.lookup_expect("TypeList-length", &[pat.tylist.to_value(&ctx).val]);

                let new_term = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "TermSubst",
                    &[
                        pat.outer_ctx.to_value(&ctx).val,
                        pat.orig_ins.to_value(&ctx).val,
                        pat.t1.to_value(&ctx).val,
                    ],
                ));
                let new_ins = eggplant::wrap::Value::<schema_dsl::Expr>::new(
                    (&ctx).insert(
                        "Concat",
                        &[
                            pat.orig_ins.to_value(&ctx).val,
                            eggplant::wrap::Value::<schema_dsl::Expr>::new(
                                (&ctx).insert("Single", &[new_term.to_value(&ctx).val]),
                            )
                            .to_value(&ctx)
                            .val,
                        ],
                    ),
                );

                let tnil =
                    eggplant::wrap::Value::<schema_dsl::TypeList>::new((ctx).insert("TNil", &[]));
                let appended_tail = eggplant::wrap::Value::<schema_dsl::TypeList>::new(
                    (&ctx).insert("TCons", &[pat.ty.to_value(&ctx).val, tnil.val]),
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
                    (&ctx).insert("SubTuple", &[arg_tr.to_value(&ctx).val, zero, orig_ins_len]),
                );
                let st_fa = eggplant::wrap::Value::<schema_dsl::Expr>::new(
                    (&ctx).insert("SubTuple", &[arg_fa.to_value(&ctx).val, zero, orig_ins_len]),
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

                let pulled_in_true = eggplant::wrap::Value::<schema_dsl::Expr>::new(
                    (&ctx).insert("Get", &[arg_tr.to_value(&ctx).val, orig_ins_len]),
                );
                let pulled_in_false = eggplant::wrap::Value::<schema_dsl::Expr>::new(
                    (&ctx).insert("Get", &[arg_fa.to_value(&ctx).val, orig_ins_len]),
                );
                let subst_e1 = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "Subst",
                    &[
                        if_tr.to_value(&ctx).val,
                        st_tr.to_value(&ctx).val,
                        pat.e1.to_value(&ctx).val,
                    ],
                ));
                let subst_e2 = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "Subst",
                    &[
                        if_fa.to_value(&ctx).val,
                        st_fa.to_value(&ctx).val,
                        pat.e2.to_value(&ctx).val,
                    ],
                ));
                ctx.union(pulled_in_true, subst_e1);
                ctx.union(pulled_in_false, subst_e2);

                let subst_x1 = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "Subst",
                    &[
                        if_tr.to_value(&ctx).val,
                        st_tr.to_value(&ctx).val,
                        pat.x1.to_value(&ctx).val,
                    ],
                ));
                let subst_y1 = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "Subst",
                    &[
                        if_tr.to_value(&ctx).val,
                        st_tr.to_value(&ctx).val,
                        pat.y1.to_value(&ctx).val,
                    ],
                ));
                let subst_x2 = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "Subst",
                    &[
                        if_fa.to_value(&ctx).val,
                        st_fa.to_value(&ctx).val,
                        pat.x2.to_value(&ctx).val,
                    ],
                ));
                let subst_y2 = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "Subst",
                    &[
                        if_fa.to_value(&ctx).val,
                        st_fa.to_value(&ctx).val,
                        pat.y2.to_value(&ctx).val,
                    ],
                ));
                let _ = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "Bop",
                    &[
                        pat.op.to_value(&ctx).val,
                        subst_x1.to_value(&ctx).val,
                        subst_y1.to_value(&ctx).val,
                    ],
                ));
                let _ = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "Bop",
                    &[
                        pat.op.to_value(&ctx).val,
                        subst_x2.to_value(&ctx).val,
                        subst_y2.to_value(&ctx).val,
                    ],
                ));
                ctx.subsume(
                    "Bop",
                    &[
                        pat.op.to_value(&ctx).val,
                        subst_x1.to_value(&ctx).val,
                        subst_y1.to_value(&ctx).val,
                    ],
                );
                ctx.subsume(
                    "Bop",
                    &[
                        pat.op.to_value(&ctx).val,
                        subst_x2.to_value(&ctx).val,
                        subst_y2.to_value(&ctx).val,
                    ],
                );

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

#[cfg(all(test, feature = "eggplant"))]
mod native_tests {
    use crate::ast::*;
    use crate::eggplant_backend::test_lock;
    use egglog::{ast::Expr as EgglogExpr, EGraph, TermDag};

    fn candidate_expr() -> String {
        tif(
            getat(2),
            parallel!(getat(0), getat(1), getat(3)),
            less_than(getat(0), int(7)),
            less_than(getat(1), int(7)),
        )
        .with_arg_types(tuplet!(intt(), intt(), boolt(), statet()), base(boolt()))
        .to_string()
    }

    fn cicm_schedule() -> String {
        let helpers = crate::schedule::helpers();
        format!("(run-schedule {helpers})\n(run-schedule cicm)\n(run-schedule {helpers})\n")
    }

    fn eval_text(prologue: &str, expr: &str, schedule: &str) -> (String, Vec<String>) {
        let program =
            format!("{prologue}\n(let __rlcr_expr {expr})\n(ExprIsValid __rlcr_expr)\n{schedule}");
        let mut egraph = EGraph::default();
        egraph.parse_and_run_program(None, &program).unwrap();

        let (serialized, _) = crate::greedy_dag_extractor::serialized_egraph(egraph.clone());
        let mut if_nodes = serialized
            .nodes
            .values()
            .filter(|node| node.op == "If")
            .map(|node| format!("{node:?}"))
            .collect::<Vec<_>>();
        if_nodes.sort();

        let mut termdag = TermDag::default();
        let (sort, value) = egraph
            .eval_expr(&EgglogExpr::Var(
                egglog::ast::Span::Panic,
                "__rlcr_expr".into(),
            ))
            .unwrap();
        let (_, extracted) = egraph.extract(value, &mut termdag, &sort).unwrap();
        (termdag.to_string(&extracted), if_nodes)
    }

    fn eval_native(
        prologue: &str,
        expr: &str,
        schedule: &str,
        ablate: Option<&str>,
    ) -> (String, Vec<String>) {
        let initialization = format!("(let __rlcr_expr {expr})\n(ExprIsValid __rlcr_expr)");
        use eggplant::egglog::ast::Expr as NativeEgglogExpr;

        crate::with_native_rules_egraph(prologue, &initialization, schedule, ablate, |egraph| {
            let (serialized, _) = crate::greedy_dag_extractor::serialized_egraph_native(egraph)
                .map_err(|err| {
                    eggplant::egglog::Error::ParseError(eggplant::egglog::ast::ParseError(
                        eggplant::egglog::ast::Span::Panic,
                        err,
                    ))
                })?;
            let mut if_nodes = serialized
                .nodes
                .values()
                .filter(|node| node.op == "If")
                .map(|node| format!("{node:?}"))
                .collect::<Vec<_>>();
            if_nodes.sort();

            let (sort, value) = egraph.eval_expr(&NativeEgglogExpr::Var(
                eggplant::egglog::ast::Span::Panic,
                "__rlcr_expr".into(),
            ))?;
            let (termdag, extracted, _) = egraph.extract_value(&sort, value)?;
            Ok((termdag.to_string(extracted), if_nodes))
        })
        .unwrap()
    }

    #[test]
    fn native_feature_path_matches_text_backend_for_cicm_case() {
        let _guard = test_lock::lock();
        let expr = candidate_expr();
        let schedule = cicm_schedule();

        let (text_extracted, text_if_nodes) =
            eval_text(&crate::prologue_egglog_text(), &expr, &schedule);
        let (native_extracted, native_if_nodes) = eval_native(
            &crate::feature_execution_prologue(true, None),
            &expr,
            &schedule,
            None,
        );

        assert_eq!(native_extracted, text_extracted);
        assert_eq!(native_if_nodes.len(), text_if_nodes.len());
        assert!(
            native_if_nodes.iter().any(|node| node.contains("SubTuple")),
            "native cicm feature path should preserve the normalized If input shape"
        );
        assert!(
            text_if_nodes.iter().any(|node| node.contains("SubTuple")),
            "text cicm backend should preserve the normalized If input shape"
        );
    }
}
