pub(crate) fn fragment() -> String {
    let mut out = String::new();
    out.push_str(GENERATED_MARKER);
    out.push_str(IVT);
    out.push('\n');
    out
}

#[cfg(feature = "eggplant")]
pub(crate) fn native_fragment() -> String {
    let mut out = String::new();
    out.push_str(GENERATED_MARKER);
    out.push_str(IVT_SUPPORT);
    out.push('\n');
    out
}

const GENERATED_MARKER: &str = "; (Generated from eggplant Rust: src/eggplant_backend/ivt.rs)\n";
const IVT_SUPPORT: &str = r#"(relation IVTNewInputsAnalysisDemand (Expr))

(ruleset ivt-analysis)

(sort IVTRes)
;;                              perm passthrough-perm passthrough-type passthrough-type-len
(constructor IVTAnalysisRes (Expr Expr             TypeList         i64) IVTRes)
(constructor IVTMin (IVTRes IVTRes) IVTRes)

(rule ((= lhs (IVTMin (IVTAnalysisRes _a _b _c len1) (IVTAnalysisRes _d _e _f len2)))
       (<= len1 len2))
      ((union lhs (IVTAnalysisRes _a _b _c len1)))
        :ruleset ivt-analysis)
(rule ((= lhs (IVTMin (IVTAnalysisRes _a _b _c len1) (IVTAnalysisRes _d _e _f len2)))
       (> len1 len2))
      ((union lhs (IVTAnalysisRes _d _e _f len2)))
        :ruleset ivt-analysis)


;; use an analysis to avoid exploring all combinations of passthrough vs not passed through values. Always prefer not passed through
;;                                  expr1 curr  if  result
(function ivt_new_inputs_analysis_impl (Expr  Expr  Node) IVTRes :merge (IVTMin old new))

;; ivt_new_inputs_analysis computes a permutation perm which corresponds to accessing elements of an if region.
;; It also makes accesses of passthrough arguments access new indices after the length of the if region.
;; For example, if expr1 is: [get(if, 1), get(arg, 1), get(if, 0), get(arg, 3)]
;; It produces a new permutation: [get(arg, 1), get(arg, 2), get(arg, 0), get(arg, 3)]
;; The accesses of the if statement remain unchanged, and the accesses of the passthrough arguments are moved to the end.
;; This new permutation is intended to be used with a substitution argument (Concat if-statement passthrough-args)
;; Also produced is a passthrough-perm, which selects all of the passthrough arguments and puts them in a single tuple
;;                              expr1 if result
(function ivt_new_inputs_analysis (Expr  Node) IVTRes :merge (IVTMin old new))

(ruleset loop-inversion)"#;
const IVT: &str = r#"(relation IVTNewInputsAnalysisDemand (Expr))

(ruleset ivt-analysis)

(sort IVTRes)
;;                              perm passthrough-perm passthrough-type passthrough-type-len
(constructor IVTAnalysisRes (Expr Expr             TypeList         i64) IVTRes)
(constructor IVTMin (IVTRes IVTRes) IVTRes)

(rule ((= lhs (IVTMin (IVTAnalysisRes _a _b _c len1) (IVTAnalysisRes _d _e _f len2)))
       (<= len1 len2))
      ((union lhs (IVTAnalysisRes _a _b _c len1)))
        :ruleset ivt-analysis)
(rule ((= lhs (IVTMin (IVTAnalysisRes _a _b _c len1) (IVTAnalysisRes _d _e _f len2)))
       (> len1 len2))
      ((union lhs (IVTAnalysisRes _d _e _f len2)))
        :ruleset ivt-analysis)


;; use an analysis to avoid exploring all combinations of passthrough vs not passed through values. Always prefer not passed through
;;                                  expr1 curr  if  result
(function ivt_new_inputs_analysis_impl (Expr  Expr  Node) IVTRes :merge (IVTMin old new))

;; ivt_new_inputs_analysis computes a permutation perm which corresponds to accessing elements of an if region.
;; It also makes accesses of passthrough arguments access new indices after the length of the if region.
;; For example, if expr1 is: [get(if, 1), get(arg, 1), get(if, 0), get(arg, 3)]
;; It produces a new permutation: [get(arg, 1), get(arg, 2), get(arg, 0), get(arg, 3)]
;; The accesses of the if statement remain unchanged, and the accesses of the passthrough arguments are moved to the end.
;; This new permutation is intended to be used with a substitution argument (Concat if-statement passthrough-args)
;; Also produced is a passthrough-perm, which selects all of the passthrough arguments and puts them in a single tuple
;;                              expr1 if result
(function ivt_new_inputs_analysis (Expr  Node) IVTRes :merge (IVTMin old new))


(rule (
    (DoWhile inpW outW)
) (
    (IVTNewInputsAnalysisDemand outW)
) :ruleset ivt-analysis)

(rule (
    (IVTNewInputsAnalysisDemand loop-body)
    ;; first input is a predicate
    (= loop-body (Concat (Single pred) rest))
    ;; another input is an if statement with shared predicate
    (= if-eclass (If pred inputs thn else))
    (= (Get loop-body i) (Get if-eclass j))
    (!= i 0)
) (
    (let perm (Empty (TmpType) (InFunc "no-ctx")))
    (set
     (ivt_new_inputs_analysis_impl loop-body rest (IfNode if-eclass pred inputs thn else))
     (IVTAnalysisRes perm perm (TNil) 0))
) :ruleset ivt-analysis)

;; recursive case for accessing the if statement
(rule (
    (= (ivt_new_inputs_analysis_impl loop-body curr ifnode) (IVTAnalysisRes perm pperm passthrough-tys len))
    (= ifnode (IfNode if-eclass pred inputs then else))
    (= curr (Concat (Single (Get if-eclass ith)) rest))
) (
    (let new-perm (Concat perm (Single (Get (Arg (TmpType) (InFunc "no-ctx")) ith))))
    (set (ivt_new_inputs_analysis_impl loop-body rest ifnode)
         (IVTAnalysisRes new-perm  pperm passthrough-tys len))
) :ruleset ivt-analysis)

;; recursive case for accessing a passed-through argument
(rule (
    (= (ivt_new_inputs_analysis_impl loop-body curr ifnode) (IVTAnalysisRes perm pperm passthrough-tys len))
    (= ifnode (IfNode if-eclass pred inputs then else))
    (= curr (Concat (Single (Get (Arg ty ctx) ith)) rest))
    (= (Get loop-body (+ ith 1)) (Get curr 0))
    (HasType (Get (Arg ty ctx) ith) (Base new-ty))
    (= (tuple-length if-eclass) if-len)
) (
    (let get-passed-through (Single (Get (Arg (TmpType) (InFunc "no-ctx")) (+ if-len len))))
    (let new-perm (Concat perm get-passed-through))
    (let original-get-index (Single (Get (Arg (TmpType) (InFunc "no-ctx")) ith)))
    (let new-pperm (Concat pperm original-get-index))
    (let new-passthrough-tys (TLConcat passthrough-tys (TCons new-ty (TNil))))
    (set (ivt_new_inputs_analysis_impl loop-body rest ifnode)
         (IVTAnalysisRes new-perm new-pperm new-passthrough-tys (+ len 1)))
) :ruleset ivt-analysis)

; base case for accessing if statement
(rule (
    (= (ivt_new_inputs_analysis_impl loop-body (Single last) ifnode) (IVTAnalysisRes perm pperm passthrough-tys len))
    (= ifnode (IfNode if-eclass pred inputs then else))
    (= last (Get if-eclass ith))
) (
    (let new-perm (Concat perm (Single (Get (Arg (TmpType) (InFunc "no-ctx")) ith))))
    (set (ivt_new_inputs_analysis loop-body ifnode) (IVTAnalysisRes new-perm pperm passthrough-tys len))
) :ruleset ivt-analysis)

; base case for accessing a passed-through argument
(rule (
    (= (ivt_new_inputs_analysis_impl loop-body curr ifnode) (IVTAnalysisRes perm pperm passthrough-tys len))
    (= ifnode (IfNode if-eclass pred inputs then else))
    (= curr (Single (Get (Arg ty ctx) ith)))
    (= (Get loop-body (+ ith 1)) (Get curr 0))
    (HasType (Get (Arg ty ctx) ith) (Base new-ty))
    (= (tuple-length if-eclass) if-len)
) (
    (let get-passed-through (Single (Get (Arg (TmpType) (InFunc "no-ctx")) (+ if-len len))))
    (let new-perm (Concat perm get-passed-through))
    (let original-get-index (Single (Get (Arg (TmpType) (InFunc "no-ctx")) ith)))
    (let new-pperm (Concat pperm original-get-index))
    (let new-passthrough-tys (TLConcat passthrough-tys (TCons new-ty (TNil))))
    (set (ivt_new_inputs_analysis loop-body ifnode) (IVTAnalysisRes new-perm new-pperm new-passthrough-tys (+ len 1)))
) :ruleset ivt-analysis)


(ruleset loop-inversion)

(rule (
    (= loop (DoWhile inpW outW))
    (= (ivt_new_inputs_analysis outW ifnode) (IVTAnalysisRes perm pperm passthrough-tys _len))
    (= ifnode (IfNode if if-cond if-inputs then else))
    (= if-inputs-len (tuple-length if-inputs))
    (= passthrough-len (TypeList-length passthrough-tys))

    (ContextOf inpW outer-ctx)
    (ContextOf if-inputs if-ctx)
    (HasType if-inputs inputs-ty)
    (= inputs-ty (TupleT inputs-ty-list))
) (
    ;; new peeled condition, checks the if's condition before the first iteration
    (let new-if-cond (Subst outer-ctx inpW if-cond))

    ;; new inputs to the if are 1) the inputs run once unconditionally concatted with
    ;; 2) the passthrough values
    (let new-if-inp
        (Concat (Subst outer-ctx inpW if-inputs)
                (Subst outer-ctx inpW pperm)))
    ;; if contexts
    (let new-if-true-ctx (InIf true new-if-cond new-if-inp))
    (let new-if-false-ctx (InIf false new-if-cond new-if-inp))

    (let new-loop-arg-ty (TupleT (TLConcat inputs-ty-list passthrough-tys)))
    (let new-loop-arg (Arg new-loop-arg-ty (TmpCtx)))
    (let new-loop-context (TmpCtx))

    ;; body
    ;; loop begins by running the then branch of the if statement, which uses the first if-inputs-length elements of arg
    (let then-arg (SubTuple new-loop-arg 0 if-inputs-len))
    (let new-then-branch
        (Subst new-loop-context then-arg then))
    ;; the inputs are then run on the combination of
    ;; the then branch and the passthrough values
    (let then-branch-and-passthrough
      (Concat new-then-branch (SubTuple new-loop-arg if-inputs-len passthrough-len)))
    ;; permute them to move passthrough and if outputs back
    ;; to where if-inputs and if-cond expect them to be
    (let permuted-then-branch-and-passthrough
      (Subst new-loop-context then-branch-and-passthrough perm))
    ;; substitute into inputs and condi
    (let new-inputs-after-then-branch 
        (Subst new-loop-context permuted-then-branch-and-passthrough
            (Concat (Single if-cond) if-inputs)))
    (let new-loop-outputs
        (Concat new-inputs-after-then-branch
           (SubTuple new-loop-arg if-inputs-len passthrough-len)))

    (let new-loop (DoWhile (Arg new-loop-arg-ty new-if-true-ctx) new-loop-outputs))
    (let new-if
        (If new-if-cond new-if-inp
            new-loop
            (Arg new-loop-arg-ty new-if-false-ctx)))

    ;; Apply the body of the false branch as an afterprocessing wrapper
    (let final-if-inputs
       (SubTuple new-if 0 if-inputs-len))
    (let else-branch-end
        (Subst outer-ctx final-if-inputs else))
    (let else-branch-end-and-passthrough
        (Concat else-branch-end
               (SubTuple new-if if-inputs-len passthrough-len)))
    (let final-permuted
        (Subst outer-ctx else-branch-end-and-passthrough perm))

    (union final-permuted loop)
    (union new-loop-context (InLoop (Arg new-loop-arg-ty new-if-true-ctx) new-loop-outputs))

    (subsume (DoWhile inpW outW))
    (delete (TmpCtx))
) :ruleset loop-inversion)"#;

#[cfg(feature = "eggplant")]
pub(crate) mod native {
    #![allow(non_camel_case_types)]

    use super::super::schema_dsl;
    use super::super::schema_dsl::{
        Expr, ExprRuleCtx, IVTNewInputsAnalysisDemandPRRuleCtx, IVTRes, IVTResPRRuleCtx, Node,
        NodePRRuleCtx,
    };
    use crate::eggplant_backend::peepholes::native::PeepholeTx;
    use eggplant::prelude::{
        prim_call, AsHandle, BaseVar, Insertable, IntoHandleTy, PEq, PatRecSgl, RuleRunnerSgl,
        RuleSetId,
    };

    #[eggplant::func(output = IVTRes, merge = "(IVTMin old new)")]
    struct ivt_new_inputs_analysis_impl {
        loop_body: Expr,
        curr: Expr,
        ifnode: Node,
    }

    #[eggplant::func(output = IVTRes, merge = "(IVTMin old new)")]
    struct ivt_new_inputs_analysis {
        loop_body: Expr,
        ifnode: Node,
    }

    #[eggplant::pat_vars]
    struct IvtDemandPat<PR: PatRecSgl> {
        out_w: schema_dsl::Expr,
    }

    fn ivt_demand_pat<PR: PatRecSgl>() -> IvtDemandPat<PR> {
        let inp_w = schema_dsl::Expr::query_leaf();
        let out_w = schema_dsl::Expr::query_leaf();
        let _loop_expr = schema_dsl::DoWhile::query(&inp_w, &out_w);

        IvtDemandPat::new(out_w)
    }

    #[eggplant::pat_vars]
    struct IvtSeedPat<PR: PatRecSgl> {
        loop_body: schema_dsl::Expr,
        demand: schema_dsl::IVTNewInputsAnalysisDemand,
        rest: schema_dsl::Expr,
        if_eclass: schema_dsl::Expr,
        pred: schema_dsl::Expr,
        inputs: schema_dsl::Expr,
        then_branch: schema_dsl::Expr,
        else_branch: schema_dsl::Expr,
    }

    fn ivt_seed_pat<PR: PatRecSgl>() -> IvtSeedPat<PR> {
        let pred = schema_dsl::Expr::query_leaf();
        let rest = schema_dsl::Expr::query_leaf();
        let loop_body = schema_dsl::Expr::query_leaf();
        let inputs = schema_dsl::Expr::query_leaf();
        let then_branch = schema_dsl::Expr::query_leaf();
        let else_branch = schema_dsl::Expr::query_leaf();
        let if_eclass = schema_dsl::Expr::query_leaf();
        let single_pred = schema_dsl::Single::query(&pred);
        let concat_body = schema_dsl::Concat::query(&single_pred, &rest);
        let if_expr = schema_dsl::If::query(&pred, &inputs, &then_branch, &else_branch);
        let loop_get = schema_dsl::Get::query(&loop_body);
        let if_get = schema_dsl::Get::query(&if_eclass);
        let demand = schema_dsl::IVTNewInputsAnalysisDemand::query();

        let demand_matches_loop_body = demand.expr.handle().eq(&loop_body.handle());
        let loop_matches_concat = loop_body.handle().eq(&concat_body.handle());
        let if_matches = if_eclass.handle().eq(&if_expr.handle());
        let same_get = loop_get.handle().eq(&if_get.handle());
        let not_pred_slot = loop_get.handle_index().ne(&0_i64);

        IvtSeedPat::new(
            loop_body,
            demand,
            rest,
            if_eclass,
            pred,
            inputs,
            then_branch,
            else_branch,
        )
        .assert(demand_matches_loop_body)
        .assert(loop_matches_concat)
        .assert(if_matches)
        .assert(same_get)
        .assert(not_pred_slot)
    }

    #[eggplant::pat_vars]
    struct IvtRecurseIfAccessPat<PR: PatRecSgl> {
        loop_body: schema_dsl::Expr,
        rest: schema_dsl::Expr,
        if_eclass: schema_dsl::Expr,
        pred: schema_dsl::Expr,
        inputs: schema_dsl::Expr,
        then_branch: schema_dsl::Expr,
        else_branch: schema_dsl::Expr,
        perm: schema_dsl::Expr,
        pperm: schema_dsl::Expr,
        passthrough_tys: schema_dsl::TypeList,
        len: i64,
        if_get: schema_dsl::Get,
    }

    fn ivt_recurse_if_access_pat<PR: PatRecSgl>() -> IvtRecurseIfAccessPat<PR> {
        let loop_body = schema_dsl::Expr::query_leaf();
        let rest = schema_dsl::Expr::query_leaf();
        let if_eclass = schema_dsl::Expr::query_leaf();
        let pred = schema_dsl::Expr::query_leaf();
        let inputs = schema_dsl::Expr::query_leaf();
        let then_branch = schema_dsl::Expr::query_leaf();
        let else_branch = schema_dsl::Expr::query_leaf();
        let perm = schema_dsl::Expr::query_leaf();
        let pperm = schema_dsl::Expr::query_leaf();
        let passthrough_tys = schema_dsl::TypeList::query_leaf();
        let len = BaseVar::<i64, PR>::query_named("ivt_len");
        let if_get = schema_dsl::Get::query(&if_eclass);
        let single_if_get = schema_dsl::Single::query(&if_get);
        let curr = schema_dsl::Concat::query(&single_if_get, &rest);
        let ifnode =
            schema_dsl::IfNode::query(&if_eclass, &pred, &inputs, &then_branch, &else_branch);
        let analysis = schema_dsl::IVTAnalysisRes::query(&perm, &pperm, &passthrough_tys);
        let analysis_matches = analysis
            .handle()
            .eq(&ivt_new_inputs_analysis_impl::query(&loop_body, &curr, &ifnode).handle());
        let len_matches = analysis.handle_len().eq(&len.handle());

        IvtRecurseIfAccessPat::new(
            loop_body,
            rest,
            if_eclass,
            pred,
            inputs,
            then_branch,
            else_branch,
            perm,
            pperm,
            passthrough_tys,
            len,
            if_get,
        )
        .assert(analysis_matches)
        .assert(len_matches)
    }

    #[eggplant::pat_vars]
    struct IvtRecursePassthroughPat<PR: PatRecSgl> {
        loop_body: schema_dsl::Expr,
        rest: schema_dsl::Expr,
        if_eclass: schema_dsl::Expr,
        pred: schema_dsl::Expr,
        inputs: schema_dsl::Expr,
        then_branch: schema_dsl::Expr,
        else_branch: schema_dsl::Expr,
        perm: schema_dsl::Expr,
        pperm: schema_dsl::Expr,
        passthrough_tys: schema_dsl::TypeList,
        len: i64,
        if_len: i64,
        arg_get: schema_dsl::Get,
        arg_has_base_type: schema_dsl::HasType,
        new_ty: schema_dsl::BaseType,
    }

    fn ivt_recurse_passthrough_pat<PR: PatRecSgl>() -> IvtRecursePassthroughPat<PR> {
        let loop_body = schema_dsl::Expr::query_leaf();
        let rest = schema_dsl::Expr::query_leaf();
        let if_eclass = schema_dsl::Expr::query_leaf();
        let pred = schema_dsl::Expr::query_leaf();
        let inputs = schema_dsl::Expr::query_leaf();
        let then_branch = schema_dsl::Expr::query_leaf();
        let else_branch = schema_dsl::Expr::query_leaf();
        let perm = schema_dsl::Expr::query_leaf();
        let pperm = schema_dsl::Expr::query_leaf();
        let passthrough_tys = schema_dsl::TypeList::query_leaf();
        let len = BaseVar::<i64, PR>::query_named("ivt_len");
        let if_len = BaseVar::<i64, PR>::query_named("ivt_if_len");
        let arg_ty = schema_dsl::Type::query_leaf();
        let arg_ctx = schema_dsl::Assumption::query_leaf();
        let arg_get = schema_dsl::Get::query(&schema_dsl::Arg::query(&arg_ty, &arg_ctx));
        let loop_get = schema_dsl::Get::query(&loop_body);
        let new_ty = schema_dsl::BaseType::query_leaf();
        let new_base_ty = schema_dsl::Base::query(&new_ty);
        let arg_has_base_type = schema_dsl::HasType::query();
        let single_arg_get = schema_dsl::Single::query(&arg_get);
        let curr = schema_dsl::Concat::query(&single_arg_get, &rest);
        let arg_has_base_type_matches_arg = arg_has_base_type.expr.handle().eq(&arg_get.handle());
        let arg_has_base_type_matches_ty = arg_has_base_type.ty.handle().eq(&new_base_ty.handle());
        let loop_matches_arg = loop_get.handle().eq(&arg_get.handle());
        let shifted_index = loop_get
            .handle_index()
            .eq(&(arg_get.handle_index() + (&1_i64).as_handle()));
        let ifnode =
            schema_dsl::IfNode::query(&if_eclass, &pred, &inputs, &then_branch, &else_branch);
        let analysis = schema_dsl::IVTAnalysisRes::query(&perm, &pperm, &passthrough_tys);
        let analysis_matches = analysis
            .handle()
            .eq(&ivt_new_inputs_analysis_impl::query(&loop_body, &curr, &ifnode).handle());
        let len_matches = analysis.handle_len().eq(&len.handle());
        let if_len_known = if_len.handle().eq(&prim_call::<i64>(
            "tuple-length",
            vec![if_eclass.handle().into_handle_ty()],
        ));

        IvtRecursePassthroughPat::new(
            loop_body,
            rest,
            if_eclass,
            pred,
            inputs,
            then_branch,
            else_branch,
            perm,
            pperm,
            passthrough_tys,
            len,
            if_len,
            arg_get,
            arg_has_base_type,
            new_ty,
        )
        .assert(arg_has_base_type_matches_arg)
        .assert(arg_has_base_type_matches_ty)
        .assert(analysis_matches)
        .assert(len_matches)
        .assert(if_len_known)
        .assert(loop_matches_arg)
        .assert(shifted_index)
    }

    #[eggplant::pat_vars]
    struct IvtFinishIfAccessPat<PR: PatRecSgl> {
        loop_body: schema_dsl::Expr,
        if_eclass: schema_dsl::Expr,
        pred: schema_dsl::Expr,
        inputs: schema_dsl::Expr,
        then_branch: schema_dsl::Expr,
        else_branch: schema_dsl::Expr,
        perm: schema_dsl::Expr,
        pperm: schema_dsl::Expr,
        passthrough_tys: schema_dsl::TypeList,
        len: i64,
        last: schema_dsl::Get,
    }

    fn ivt_finish_if_access_pat<PR: PatRecSgl>() -> IvtFinishIfAccessPat<PR> {
        let loop_body = schema_dsl::Expr::query_leaf();
        let if_eclass = schema_dsl::Expr::query_leaf();
        let pred = schema_dsl::Expr::query_leaf();
        let inputs = schema_dsl::Expr::query_leaf();
        let then_branch = schema_dsl::Expr::query_leaf();
        let else_branch = schema_dsl::Expr::query_leaf();
        let perm = schema_dsl::Expr::query_leaf();
        let pperm = schema_dsl::Expr::query_leaf();
        let passthrough_tys = schema_dsl::TypeList::query_leaf();
        let len = BaseVar::<i64, PR>::query_named("ivt_len");
        let last = schema_dsl::Get::query(&if_eclass);
        let curr = schema_dsl::Single::query(&last);
        let ifnode =
            schema_dsl::IfNode::query(&if_eclass, &pred, &inputs, &then_branch, &else_branch);
        let analysis = schema_dsl::IVTAnalysisRes::query(&perm, &pperm, &passthrough_tys);
        let analysis_matches = analysis
            .handle()
            .eq(&ivt_new_inputs_analysis_impl::query(&loop_body, &curr, &ifnode).handle());
        let len_matches = analysis.handle_len().eq(&len.handle());

        IvtFinishIfAccessPat::new(
            loop_body,
            if_eclass,
            pred,
            inputs,
            then_branch,
            else_branch,
            perm,
            pperm,
            passthrough_tys,
            len,
            last,
        )
        .assert(analysis_matches)
        .assert(len_matches)
    }

    #[eggplant::pat_vars]
    struct IvtFinishPassthroughPat<PR: PatRecSgl> {
        loop_body: schema_dsl::Expr,
        if_eclass: schema_dsl::Expr,
        pred: schema_dsl::Expr,
        inputs: schema_dsl::Expr,
        then_branch: schema_dsl::Expr,
        else_branch: schema_dsl::Expr,
        perm: schema_dsl::Expr,
        pperm: schema_dsl::Expr,
        passthrough_tys: schema_dsl::TypeList,
        len: i64,
        if_len: i64,
        arg_get: schema_dsl::Get,
        arg_has_base_type: schema_dsl::HasType,
        new_ty: schema_dsl::BaseType,
    }

    fn ivt_finish_passthrough_pat<PR: PatRecSgl>() -> IvtFinishPassthroughPat<PR> {
        let loop_body = schema_dsl::Expr::query_leaf();
        let if_eclass = schema_dsl::Expr::query_leaf();
        let pred = schema_dsl::Expr::query_leaf();
        let inputs = schema_dsl::Expr::query_leaf();
        let then_branch = schema_dsl::Expr::query_leaf();
        let else_branch = schema_dsl::Expr::query_leaf();
        let perm = schema_dsl::Expr::query_leaf();
        let pperm = schema_dsl::Expr::query_leaf();
        let passthrough_tys = schema_dsl::TypeList::query_leaf();
        let len = BaseVar::<i64, PR>::query_named("ivt_len");
        let if_len = BaseVar::<i64, PR>::query_named("ivt_if_len");
        let arg_ty = schema_dsl::Type::query_leaf();
        let arg_ctx = schema_dsl::Assumption::query_leaf();
        let arg_get = schema_dsl::Get::query(&schema_dsl::Arg::query(&arg_ty, &arg_ctx));
        let loop_get = schema_dsl::Get::query(&loop_body);
        let new_ty = schema_dsl::BaseType::query_leaf();
        let new_base_ty = schema_dsl::Base::query(&new_ty);
        let arg_has_base_type = schema_dsl::HasType::query();
        let curr = schema_dsl::Single::query(&arg_get);
        let arg_has_base_type_matches_arg = arg_has_base_type.expr.handle().eq(&arg_get.handle());
        let arg_has_base_type_matches_ty = arg_has_base_type.ty.handle().eq(&new_base_ty.handle());
        let loop_matches_arg = loop_get.handle().eq(&arg_get.handle());
        let shifted_index = loop_get
            .handle_index()
            .eq(&(arg_get.handle_index() + (&1_i64).as_handle()));
        let ifnode =
            schema_dsl::IfNode::query(&if_eclass, &pred, &inputs, &then_branch, &else_branch);
        let analysis = schema_dsl::IVTAnalysisRes::query(&perm, &pperm, &passthrough_tys);
        let analysis_matches = analysis
            .handle()
            .eq(&ivt_new_inputs_analysis_impl::query(&loop_body, &curr, &ifnode).handle());
        let len_matches = analysis.handle_len().eq(&len.handle());
        let if_len_known = if_len.handle().eq(&prim_call::<i64>(
            "tuple-length",
            vec![if_eclass.handle().into_handle_ty()],
        ));

        IvtFinishPassthroughPat::new(
            loop_body,
            if_eclass,
            pred,
            inputs,
            then_branch,
            else_branch,
            perm,
            pperm,
            passthrough_tys,
            len,
            if_len,
            arg_get,
            arg_has_base_type,
            new_ty,
        )
        .assert(arg_has_base_type_matches_arg)
        .assert(arg_has_base_type_matches_ty)
        .assert(analysis_matches)
        .assert(len_matches)
        .assert(if_len_known)
        .assert(loop_matches_arg)
        .assert(shifted_index)
    }

    #[eggplant::pat_vars]
    struct LoopInversionPat<PR: PatRecSgl> {
        loop_expr: schema_dsl::DoWhile,
        inp_w: schema_dsl::Expr,
        out_w: schema_dsl::Expr,
        if_cond: schema_dsl::Expr,
        if_inputs: schema_dsl::Expr,
        then_branch: schema_dsl::Expr,
        else_branch: schema_dsl::Expr,
        perm: schema_dsl::Expr,
        pperm: schema_dsl::Expr,
        passthrough_tys: schema_dsl::TypeList,
        outer_ctx: schema_dsl::Assumption,
        if_ctx: schema_dsl::Assumption,
        inputs_ty_list: schema_dsl::TypeList,
        outer_context: schema_dsl::ContextOf,
        if_context: schema_dsl::ContextOf,
        if_inputs_have_type: schema_dsl::HasType,
    }

    fn loop_inversion_pat<PR: PatRecSgl>() -> LoopInversionPat<PR> {
        let inp_w = schema_dsl::Expr::query_leaf();
        let out_w = schema_dsl::Expr::query_leaf();
        let if_cond = schema_dsl::Expr::query_leaf();
        let if_inputs = schema_dsl::Expr::query_leaf();
        let then_branch = schema_dsl::Expr::query_leaf();
        let else_branch = schema_dsl::Expr::query_leaf();
        let loop_expr = schema_dsl::DoWhile::query(&inp_w, &out_w);
        let if_expr = schema_dsl::If::query(&if_cond, &if_inputs, &then_branch, &else_branch);
        let perm = schema_dsl::Expr::query_leaf();
        let pperm = schema_dsl::Expr::query_leaf();
        let passthrough_tys = schema_dsl::TypeList::query_leaf();
        let outer_ctx = schema_dsl::Assumption::query_leaf();
        let if_ctx = schema_dsl::Assumption::query_leaf();
        let inputs_ty_list = schema_dsl::TypeList::query_leaf();
        let inputs_ty = schema_dsl::TupleT::query(&inputs_ty_list);
        let outer_context = schema_dsl::ContextOf::query();
        let if_context = schema_dsl::ContextOf::query();
        let if_inputs_have_type = schema_dsl::HasType::query();
        let outer_context_matches_expr = outer_context.expr.handle().eq(&inp_w.handle());
        let outer_context_matches_ctx = outer_context.ctx.handle().eq(&outer_ctx.handle());
        let if_context_matches_expr = if_context.expr.handle().eq(&if_inputs.handle());
        let if_context_matches_ctx = if_context.ctx.handle().eq(&if_ctx.handle());
        let if_inputs_have_type_matches_expr =
            if_inputs_have_type.expr.handle().eq(&if_inputs.handle());
        let if_inputs_have_type_matches_ty = if_inputs_have_type.ty.handle().eq(&inputs_ty.handle());
        let len = BaseVar::<i64, PR>::query_named("_len");
        let ifnode =
            schema_dsl::IfNode::query(&if_expr, &if_cond, &if_inputs, &then_branch, &else_branch);
        let analysis = schema_dsl::IVTAnalysisRes::query(&perm, &pperm, &passthrough_tys);
        let analysis_matches = analysis
            .handle()
            .eq(&ivt_new_inputs_analysis::query(&out_w, &ifnode).handle());
        let len_matches = analysis.handle_len().eq(&len.handle());

        LoopInversionPat::new(
            loop_expr,
            inp_w,
            out_w,
            if_cond,
            if_inputs,
            then_branch,
            else_branch,
            perm,
            pperm,
            passthrough_tys,
            outer_ctx,
            if_ctx,
            inputs_ty_list,
            outer_context,
            if_context,
            if_inputs_have_type,
        )
        .assert(outer_context_matches_expr)
        .assert(outer_context_matches_ctx)
        .assert(if_context_matches_expr)
        .assert(if_context_matches_ctx)
        .assert(if_inputs_have_type_matches_expr)
        .assert(if_inputs_have_type_matches_ty)
        .assert(analysis_matches)
        .assert(len_matches)
    }

    pub(crate) fn register_native_support_rules() -> RuleSetId {
        let ruleset = RuleSetId("ivt-analysis");

        PeepholeTx::add_rule(
            "ivt_analysis_demand_loop_outputs",
            ruleset,
            ivt_demand_pat,
            |ctx, pat| {
                ctx.insert_ivt_new_inputs_analysis_demand(pat.out_w);
            },
        );

        PeepholeTx::add_rule(
            "ivt_analysis_seed_from_if_output",
            ruleset,
            ivt_seed_pat,
            |ctx, pat| {
                let no_ctx_name = ctx.intern_base::<String, _>("no-ctx".to_owned());
                let tmp_type =
                    eggplant::wrap::Value::<schema_dsl::Type>::new((ctx).insert("TmpType", &[]));
                let no_ctx = eggplant::wrap::Value::<schema_dsl::Assumption>::new(
                    (ctx).insert("InFunc", &[no_ctx_name.val]),
                );
                let perm = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "Empty",
                    &[tmp_type.to_value(&ctx).val, no_ctx.to_value(&ctx).val],
                ));
                let tnil =
                    eggplant::wrap::Value::<schema_dsl::TypeList>::new((ctx).insert("TNil", &[]));
                let ifnode = ctx.insert_if_node(
                    pat.if_eclass,
                    pat.pred,
                    pat.inputs,
                    pat.then_branch,
                    pat.else_branch,
                );
                let res = ctx.insert_ivt_analysis_res(perm, perm, tnil, 0_i64);
                ctx.set_ivt_new_inputs_analysis_impl(pat.loop_body, pat.rest, ifnode, res);
            },
        );

        PeepholeTx::add_rule(
            "ivt_analysis_recurse_if_access",
            ruleset,
            ivt_recurse_if_access_pat,
            |ctx, pat| {
                let no_ctx_name = ctx.intern_base::<String, _>("no-ctx".to_owned());
                let tmp_type =
                    eggplant::wrap::Value::<schema_dsl::Type>::new((ctx).insert("TmpType", &[]));
                let no_ctx = eggplant::wrap::Value::<schema_dsl::Assumption>::new(
                    (ctx).insert("InFunc", &[no_ctx_name.val]),
                );
                let tmp_arg = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "Arg",
                    &[tmp_type.to_value(&ctx).val, no_ctx.to_value(&ctx).val],
                ));
                let if_access = ctx.insert_single(ctx.insert_get(tmp_arg, ctx.devalue(pat.if_get.index)));
                let new_perm = eggplant::wrap::Value::<schema_dsl::Expr>::new(
                    (&ctx).insert(
                        "Concat",
                        &[pat.perm.to_value(&ctx).val, if_access.val],
                    ),
                );
                let ifnode = ctx.insert_if_node(
                    pat.if_eclass,
                    pat.pred,
                    pat.inputs,
                    pat.then_branch,
                    pat.else_branch,
                );
                let res = ctx.insert_ivt_analysis_res(
                    new_perm,
                    pat.pperm,
                    pat.passthrough_tys,
                    ctx.devalue(pat.len),
                );
                ctx.set_ivt_new_inputs_analysis_impl(pat.loop_body, pat.rest, ifnode, res);
            },
        );

        PeepholeTx::add_rule(
            "ivt_analysis_recurse_passthrough_access",
            ruleset,
            ivt_recurse_passthrough_pat,
            |ctx, pat| {
                let no_ctx_name = ctx.intern_base::<String, _>("no-ctx".to_owned());
                let tmp_type =
                    eggplant::wrap::Value::<schema_dsl::Type>::new((ctx).insert("TmpType", &[]));
                let no_ctx = eggplant::wrap::Value::<schema_dsl::Assumption>::new(
                    (ctx).insert("InFunc", &[no_ctx_name.val]),
                );
                let tmp_arg = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "Arg",
                    &[tmp_type.to_value(&ctx).val, no_ctx.to_value(&ctx).val],
                ));
                let len = ctx.devalue(pat.len);
                let if_len = ctx.devalue(pat.if_len);
                let get_passed_through =
                    ctx.insert_single(ctx.insert_get(tmp_arg, if_len + len));
                let new_perm = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "Concat",
                    &[pat.perm.to_value(&ctx).val, get_passed_through.val],
                ));
                let original_get_index =
                    ctx.insert_single(ctx.insert_get(tmp_arg, ctx.devalue(pat.arg_get.index)));
                let new_pperm = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "Concat",
                    &[pat.pperm.to_value(&ctx).val, original_get_index.val],
                ));
                let tnil =
                    eggplant::wrap::Value::<schema_dsl::TypeList>::new((ctx).insert("TNil", &[]));
                let new_passthrough_tys = eggplant::wrap::Value::<schema_dsl::TypeList>::new(
                    (&ctx).insert(
                        "TLConcat",
                        &[
                            pat.passthrough_tys.to_value(&ctx).val,
                            eggplant::wrap::Value::<schema_dsl::TypeList>::new((&ctx).insert(
                                "TCons",
                                &[pat.new_ty.to_value(&ctx).val, tnil.to_value(&ctx).val],
                            ))
                            .to_value(&ctx)
                            .val,
                        ],
                    ),
                );
                let ifnode = ctx.insert_if_node(
                    pat.if_eclass,
                    pat.pred,
                    pat.inputs,
                    pat.then_branch,
                    pat.else_branch,
                );
                let res =
                    ctx.insert_ivt_analysis_res(new_perm, new_pperm, new_passthrough_tys, len + 1);
                ctx.set_ivt_new_inputs_analysis_impl(pat.loop_body, pat.rest, ifnode, res);
            },
        );

        PeepholeTx::add_rule(
            "ivt_analysis_finish_if_access",
            ruleset,
            ivt_finish_if_access_pat,
            |ctx, pat| {
                let no_ctx_name = ctx.intern_base::<String, _>("no-ctx".to_owned());
                let tmp_type =
                    eggplant::wrap::Value::<schema_dsl::Type>::new((ctx).insert("TmpType", &[]));
                let no_ctx = eggplant::wrap::Value::<schema_dsl::Assumption>::new(
                    (ctx).insert("InFunc", &[no_ctx_name.val]),
                );
                let tmp_arg = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "Arg",
                    &[tmp_type.to_value(&ctx).val, no_ctx.to_value(&ctx).val],
                ));
                let last_access = ctx.insert_single(ctx.insert_get(tmp_arg, ctx.devalue(pat.last.index)));
                let new_perm = eggplant::wrap::Value::<schema_dsl::Expr>::new(
                    (&ctx).insert(
                        "Concat",
                        &[pat.perm.to_value(&ctx).val, last_access.val],
                    ),
                );
                let ifnode = ctx.insert_if_node(
                    pat.if_eclass,
                    pat.pred,
                    pat.inputs,
                    pat.then_branch,
                    pat.else_branch,
                );
                let res = ctx.insert_ivt_analysis_res(
                    new_perm,
                    pat.pperm,
                    pat.passthrough_tys,
                    ctx.devalue(pat.len),
                );
                ctx.set_ivt_new_inputs_analysis(pat.loop_body, ifnode, res);
            },
        );

        PeepholeTx::add_rule(
            "ivt_analysis_finish_passthrough_access",
            ruleset,
            ivt_finish_passthrough_pat,
            |ctx, pat| {
                let no_ctx_name = ctx.intern_base::<String, _>("no-ctx".to_owned());
                let tmp_type =
                    eggplant::wrap::Value::<schema_dsl::Type>::new((ctx).insert("TmpType", &[]));
                let no_ctx = eggplant::wrap::Value::<schema_dsl::Assumption>::new(
                    (ctx).insert("InFunc", &[no_ctx_name.val]),
                );
                let tmp_arg = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "Arg",
                    &[tmp_type.to_value(&ctx).val, no_ctx.to_value(&ctx).val],
                ));
                let len = ctx.devalue(pat.len);
                let if_len = ctx.devalue(pat.if_len);
                let get_passed_through =
                    ctx.insert_single(ctx.insert_get(tmp_arg, if_len + len));
                let new_perm = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "Concat",
                    &[pat.perm.to_value(&ctx).val, get_passed_through.val],
                ));
                let original_get_index =
                    ctx.insert_single(ctx.insert_get(tmp_arg, ctx.devalue(pat.arg_get.index)));
                let new_pperm = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "Concat",
                    &[pat.pperm.to_value(&ctx).val, original_get_index.val],
                ));
                let tnil =
                    eggplant::wrap::Value::<schema_dsl::TypeList>::new((ctx).insert("TNil", &[]));
                let new_passthrough_tys = eggplant::wrap::Value::<schema_dsl::TypeList>::new(
                    (&ctx).insert(
                        "TLConcat",
                        &[
                            pat.passthrough_tys.to_value(&ctx).val,
                            eggplant::wrap::Value::<schema_dsl::TypeList>::new((&ctx).insert(
                                "TCons",
                                &[pat.new_ty.to_value(&ctx).val, tnil.to_value(&ctx).val],
                            ))
                            .to_value(&ctx)
                            .val,
                        ],
                    ),
                );
                let ifnode = ctx.insert_if_node(
                    pat.if_eclass,
                    pat.pred,
                    pat.inputs,
                    pat.then_branch,
                    pat.else_branch,
                );
                let res =
                    ctx.insert_ivt_analysis_res(new_perm, new_pperm, new_passthrough_tys, len + 1);
                ctx.set_ivt_new_inputs_analysis(pat.loop_body, ifnode, res);
            },
        );

        ruleset
    }

    pub(crate) fn register_native_rules() -> RuleSetId {
        let ruleset = RuleSetId("loop-inversion");

        PeepholeTx::add_rule("loop_inversion", ruleset, loop_inversion_pat, |ctx, pat| {
            let zero = ctx._intern_base::<i64, i64>(0);
            let if_inputs_len =
                ctx.lookup_expect("tuple-length", &[pat.if_inputs.to_value(&ctx).val]);
            let passthrough_len =
                ctx.lookup_expect("TypeList-length", &[pat.passthrough_tys.to_value(&ctx).val]);

            let new_if_cond = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                "Subst",
                &[
                    pat.outer_ctx.to_value(&ctx).val,
                    pat.inp_w.to_value(&ctx).val,
                    pat.if_cond.to_value(&ctx).val,
                ],
            ));
            let subst_if_inputs = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                "Subst",
                &[
                    pat.outer_ctx.to_value(&ctx).val,
                    pat.inp_w.to_value(&ctx).val,
                    pat.if_inputs.to_value(&ctx).val,
                ],
            ));
            let subst_pperm = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                "Subst",
                &[
                    pat.outer_ctx.to_value(&ctx).val,
                    pat.inp_w.to_value(&ctx).val,
                    pat.pperm.to_value(&ctx).val,
                ],
            ));
            let new_if_inp = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                "Concat",
                &[
                    subst_if_inputs.to_value(&ctx).val,
                    subst_pperm.to_value(&ctx).val,
                ],
            ));
            let new_if_true_ctx =
                eggplant::wrap::Value::<schema_dsl::Assumption>::new((&ctx).insert(
                    "InIf",
                    &[
                        true.to_value(&ctx).val,
                        new_if_cond.to_value(&ctx).val,
                        new_if_inp.to_value(&ctx).val,
                    ],
                ));
            let new_if_false_ctx =
                eggplant::wrap::Value::<schema_dsl::Assumption>::new((&ctx).insert(
                    "InIf",
                    &[
                        false.to_value(&ctx).val,
                        new_if_cond.to_value(&ctx).val,
                        new_if_inp.to_value(&ctx).val,
                    ],
                ));

            let new_ty_list = eggplant::wrap::Value::<schema_dsl::TypeList>::new((&ctx).insert(
                "TLConcat",
                &[
                    pat.inputs_ty_list.to_value(&ctx).val,
                    pat.passthrough_tys.to_value(&ctx).val,
                ],
            ));
            let new_loop_arg_ty = eggplant::wrap::Value::<schema_dsl::Type>::new(
                (ctx).insert("TupleT", &[new_ty_list.val]),
            );
            let tmp_ctx =
                eggplant::wrap::Value::<schema_dsl::Assumption>::new((ctx).insert("TmpCtx", &[]));
            let new_loop_arg = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                "Arg",
                &[
                    new_loop_arg_ty.to_value(&ctx).val,
                    tmp_ctx.to_value(&ctx).val,
                ],
            ));
            let then_arg = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                "SubTuple",
                &[new_loop_arg.to_value(&ctx).val, zero, if_inputs_len],
            ));
            let new_then_branch = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                "Subst",
                &[
                    tmp_ctx.to_value(&ctx).val,
                    then_arg.to_value(&ctx).val,
                    pat.then_branch.to_value(&ctx).val,
                ],
            ));
            let passthrough_suffix = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                "SubTuple",
                &[
                    new_loop_arg.to_value(&ctx).val,
                    if_inputs_len,
                    passthrough_len,
                ],
            ));
            let then_and_passthrough =
                eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "Concat",
                    &[
                        new_then_branch.to_value(&ctx).val,
                        passthrough_suffix.to_value(&ctx).val,
                    ],
                ));
            let permuted_then_and_passthrough =
                eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "Subst",
                    &[
                        tmp_ctx.to_value(&ctx).val,
                        then_and_passthrough.to_value(&ctx).val,
                        pat.perm.to_value(&ctx).val,
                    ],
                ));
            let if_cond_and_inputs = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                "Concat",
                &[
                    ctx.insert_single(pat.if_cond).val,
                    pat.if_inputs.to_value(&ctx).val,
                ],
            ));
            let new_inputs_after_then =
                eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "Subst",
                    &[
                        tmp_ctx.to_value(&ctx).val,
                        permuted_then_and_passthrough.to_value(&ctx).val,
                        if_cond_and_inputs.to_value(&ctx).val,
                    ],
                ));
            let new_loop_outputs = eggplant::wrap::Value::<schema_dsl::Expr>::new(
                (&ctx).insert(
                    "Concat",
                    &[
                        new_inputs_after_then.to_value(&ctx).val,
                        eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                            "SubTuple",
                            &[
                                new_loop_arg.to_value(&ctx).val,
                                if_inputs_len,
                                passthrough_len,
                            ],
                        ))
                        .to_value(&ctx)
                        .val,
                    ],
                ),
            );

            let new_loop_input = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                "Arg",
                &[
                    new_loop_arg_ty.to_value(&ctx).val,
                    new_if_true_ctx.to_value(&ctx).val,
                ],
            ));
            let new_loop = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                "DoWhile",
                &[
                    new_loop_input.to_value(&ctx).val,
                    new_loop_outputs.to_value(&ctx).val,
                ],
            ));
            let new_if = eggplant::wrap::Value::<schema_dsl::Expr>::new(
                (&ctx).insert(
                    "If",
                    &[
                        new_if_cond.to_value(&ctx).val,
                        new_if_inp.to_value(&ctx).val,
                        new_loop.to_value(&ctx).val,
                        eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                            "Arg",
                            &[
                                new_loop_arg_ty.to_value(&ctx).val,
                                new_if_false_ctx.to_value(&ctx).val,
                            ],
                        ))
                        .to_value(&ctx)
                        .val,
                    ],
                ),
            );

            let final_if_inputs = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                "SubTuple",
                &[new_if.to_value(&ctx).val, zero, if_inputs_len],
            ));
            let else_branch_end = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                "Subst",
                &[
                    pat.outer_ctx.to_value(&ctx).val,
                    final_if_inputs.to_value(&ctx).val,
                    pat.else_branch.to_value(&ctx).val,
                ],
            ));
            let else_branch_end_and_passthrough = eggplant::wrap::Value::<schema_dsl::Expr>::new(
                (&ctx).insert(
                    "Concat",
                    &[
                        else_branch_end.to_value(&ctx).val,
                        eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                            "SubTuple",
                            &[new_if.to_value(&ctx).val, if_inputs_len, passthrough_len],
                        ))
                        .to_value(&ctx)
                        .val,
                    ],
                ),
            );
            let final_permuted = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                "Subst",
                &[
                    pat.outer_ctx.to_value(&ctx).val,
                    else_branch_end_and_passthrough.to_value(&ctx).val,
                    pat.perm.to_value(&ctx).val,
                ],
            ));

            ctx.union(final_permuted, pat.loop_expr);
            let loop_ctx = eggplant::wrap::Value::<schema_dsl::Assumption>::new((&ctx).insert(
                "InLoop",
                &[
                    new_loop_input.to_value(&ctx).val,
                    new_loop_outputs.to_value(&ctx).val,
                ],
            ));
            ctx.union(tmp_ctx, loop_ctx);
            ctx.subsume(
                "DoWhile",
                &[pat.inp_w.to_value(&ctx).val, pat.out_w.to_value(&ctx).val],
            );
            ctx.remove("TmpCtx", &[]);
        });

        ruleset
    }
}

#[cfg(all(test, feature = "eggplant"))]
mod native_tests {
    use crate::ast::*;
    use crate::eggplant_backend::test_lock;
    use crate::schedule;

    fn candidate_expr() -> String {
        let cond = less_than(getat(0), int(10));
        let if_in_loop = tif(
            cond.clone(),
            parallel!(add(getat(0), getat(1))),
            parallel!(add(getat(0), int(1)), int(2)),
            parallel!(getat(0), int(3)),
        );

        dowhile(
            parallel!(int(0), int(0), int(1)),
            parallel!(
                cond,
                get(if_in_loop.clone(), 0),
                get(if_in_loop, 1),
                getat(2)
            ),
        )
        .add_arg_type(tuplet!())
        .add_ctx(infunc("main"))
        .0
        .to_string()
    }

    fn focused_schedule() -> String {
        let helpers = schedule::helpers();
        format!(
            "(run-schedule\n  (repeat 3\n    {helpers}\n    (saturate ivt-analysis))\n  {helpers}\n)"
        )
    }

    fn parse_print_size_output<I, S>(messages: I, function_name: &str) -> usize
    where
        I: IntoIterator<Item = S>,
        S: ToString,
    {
        messages
            .into_iter()
            .find_map(|message| message.to_string().trim().parse::<usize>().ok())
            .unwrap_or_else(|| panic!("missing print-size output for {function_name}"))
    }

    fn text_function_size(
        prologue: &str,
        expr: &str,
        schedule: &str,
        function_name: &str,
    ) -> usize {
        let program = format!(
            "{prologue}\n(let __rlcr_text_expr {expr})\n{schedule}\n(print-size {function_name})\n"
        );
        let mut egraph = egglog::EGraph::default();
        let messages = egraph.parse_and_run_program(None, &program).unwrap();
        parse_print_size_output(messages, function_name)
    }

    fn native_function_size(
        prologue: &str,
        expr: &str,
        schedule: &str,
        ablate: Option<&str>,
        function_name: &str,
    ) -> usize {
        let initialization = format!("(let __rlcr_native_expr {expr})");
        crate::with_native_rules_egraph(prologue, &initialization, schedule, ablate, |egraph| {
            let messages =
                egraph.parse_and_run_program(None, &format!("(print-size {function_name})"))?;
            Ok(parse_print_size_output(messages, function_name))
        })
        .unwrap()
    }

    #[test]
    fn native_feature_path_matches_text_backend_for_loop_inversion_case() {
        let _guard = test_lock::lock();
        let expr = candidate_expr();
        let schedule = focused_schedule();
        let ablated_schedule = schedule.clone();
        let text_analysis_rows = text_function_size(
            &crate::prologue_egglog_text(),
            &expr,
            &schedule,
            "ivt_new_inputs_analysis",
        );
        let native_analysis_rows = native_function_size(
            &crate::feature_execution_prologue(true, None),
            &expr,
            &schedule,
            None,
            "ivt_new_inputs_analysis",
        );
        let ablated_analysis_rows = native_function_size(
            &crate::feature_execution_prologue(true, Some("loop-inversion")),
            &expr,
            &ablated_schedule,
            Some("loop-inversion"),
            "ivt_new_inputs_analysis",
        );

        assert_eq!(native_analysis_rows, text_analysis_rows);
        assert!(
            native_analysis_rows > 0,
            "native ivt-analysis support should materialize ivt_new_inputs_analysis rows",
        );
        assert_eq!(
            ablated_analysis_rows, native_analysis_rows,
            "ablating loop-inversion should still leave ivt-analysis support active",
        );
    }
}
