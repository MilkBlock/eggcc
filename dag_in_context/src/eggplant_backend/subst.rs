pub(crate) fn fragment() -> String {
    let mut out = String::new();
    out.push_str(GENERATED_MARKER);
    out.push_str(SUBST);
    out.push('\n');
    out
}

#[cfg(feature = "eggplant")]
pub(crate) fn native_fragment() -> String {
    // Keep the HasArgType propagation rule text-backed for now.
    // The earlier native HasArgType port changed loop_unroll feature-path extraction.
    let without_if_subst = SUBST.replacen(SUBST_IF_SUBST_RULE, "", 1);

    assert_ne!(
        without_if_subst, SUBST,
        "subst::native_fragment() expects the IfSubst rule to be present"
    );
    let without_arg = without_if_subst.replacen(SUBST_ARG_LEAF_RULE, "", 1);
    assert_ne!(
        without_arg, without_if_subst,
        "subst::native_fragment() expects the Arg leaf rule to be present"
    );
    let without_const = without_arg.replacen(SUBST_CONST_LEAF_RULE, "", 1);
    assert_ne!(
        without_const, without_arg,
        "subst::native_fragment() expects the Const leaf rule to be present"
    );
    let support_only = without_const.replacen(SUBST_EMPTY_LEAF_RULE, "", 1);
    assert_ne!(
        support_only, without_const,
        "subst::native_fragment() expects the Empty leaf rule to be present"
    );

    let mut out = String::new();
    out.push_str(GENERATED_MARKER);
    out.push_str(&support_only);
    out.push('\n');
    out
}

const GENERATED_MARKER: &str = "; (Generated from eggplant Rust: src/eggplant_backend/subst.rs)\n";
const SUBST: &str = r#";; Substitution rules allow for substituting some new expression for the argument
;; in some new context.
;; It performs the substitution, copying over the equalities from the original eclass.
;; It only places context on the leaf nodes.

(ruleset subst)
(ruleset apply-subst-unions)

;; (Subst assumption to in) substitutes `to` for `(Arg ty)` in `in`.
;; It also replaces the leaf context in `to` with `assumption` using `AddContext`.
;; `assumption` *justifies* this substitution, as the context that the result is used in.
;; In other words, it must refine the equivalence relation of `in` with `to` as the argument.
(constructor Subst (Assumption Expr Expr) Expr :unextractable)


;; Use an if statement with a true boolean to substitute `to` for `in`.
;; This is used when substitution can't be used due to
;; the weak linearity constraint.
(constructor IfSubst (Expr Expr) Expr)

(rule ((= lhs (IfSubst to in))
       (HasArgType to ty)
       (ContextOf to ctx))
      ((union lhs
         (If (Const (Bool true) ty ctx)
           to
           in
           in)))
       :ruleset subst)


;; Used to delay unions for the subst ruleset.
;; This is necessary because substitution may not terminate if it can
;; observe its own results- it may create infinitly large terms.
;; Instead, we phase substitution by delaying resulting unions in this table.
;; After applying this table, substitutions and this table are cleared.
(constructor DelayedSubstUnion (Expr Expr) Expr :unextractable)

;; add a type rule to get the arg type of a substitution
;; this enables nested substitutions
(rule ((= lhs (Subst assum to in))
       (HasArgType to ty))
      ((HasArgType lhs ty))
      :ruleset subst)

;; Substitution typechecks only when the type of the
;; argument matches the type of the substitution.
(rule ((Subst assum to in)
       (HasArgType in ty)
       (HasType to ty2)
       (!= ty ty2)
       ;; tmptype disables typechecking
       (!= ty (TmpType))
       (!= ty2 (TmpType)))
      ((extract "Extracting type mismatch")
       (extract ty)
       (extract ty2)
       (panic "Substitution type mismatch! Argument type must match type of substituted term"))
       :ruleset subst)


;; leaf node with context
;; replace this context- subst assumes the context is more specific
(rule ((= lhs (Subst assum to e))
       (= e (Arg _ty _oldctx))
       )
      ;; add the assumption `to`
      ((DelayedSubstUnion lhs (AddContext assum to))
       (subsume (Subst assum to e)))
      :ruleset subst)
(rule ((= lhs (Subst assum to e))
       (= e (Const c _ty _oldctx))
       (HasArgType to newty))
      ((DelayedSubstUnion lhs (Const c newty assum))
      (subsume (Subst assum to e)))
      :ruleset subst)
(rule ((= lhs (Subst assum to e))
       (= e (Empty _ty _oldctx))
       (HasArgType to newty))
      ((DelayedSubstUnion lhs (Empty newty assum))
      (subsume (Subst assum to e)))
      :ruleset subst)

;; Operators
(rule ((= e (Top op c1 c2 c3))
       (= lhs (Subst assum to e))
       (ExprIsResolved e)
       (ExprIsResolved to))
      ((DelayedSubstUnion lhs
         (Top op (Subst assum to c1)
                 (Subst assum to c2)
                 (Subst assum to c3)))
       (subsume (Subst assum to e)))
         :ruleset subst)

(rule ((= e (Bop op c1 c2))
       (= lhs (Subst assum to e))
       (ExprIsResolved e)
       (ExprIsResolved to))
      ((DelayedSubstUnion lhs
         (Bop op (Subst assum to c1)
                 (Subst assum to c2)))
       (subsume (Subst assum to e)))
         :ruleset subst)
(rule ((= e (Uop op c1))
       (= lhs (Subst assum to e))
       (ExprIsResolved e)
       (ExprIsResolved to))
      ((DelayedSubstUnion lhs
         (Uop op (Subst assum to c1)))
       (subsume (Subst assum to e)))
         :ruleset subst)    

(rule ((= e (Get c1 index))
       (= lhs (Subst assum to e))
       (ExprIsResolved e)
       (ExprIsResolved to))
      ((DelayedSubstUnion lhs
         (Get (Subst assum to c1) index))
       (subsume (Subst assum to e)))
         :ruleset subst)
(rule ((= e (Alloc id c1 c2 ty))
       (= lhs (Subst assum to e))
       (ExprIsResolved e)
       (ExprIsResolved to))
      ((DelayedSubstUnion lhs
         (Alloc id (Subst assum to c1)
                   (Subst assum to c2)
                   ty))
       (subsume (Subst assum to e)))
         :ruleset subst)
(rule ((= e (Call name c1))
       (= lhs (Subst assum to e))
       (ExprIsResolved e)
       (ExprIsResolved to))
      ((DelayedSubstUnion lhs
         (Call name (Subst assum to c1)))
       (subsume (Subst assum to e)))
         :ruleset subst)


;; Tuple operators
(rule ((= e (Single c1))
       (= lhs (Subst assum to e))
       (ExprIsResolved e)
       (ExprIsResolved to))
      ((DelayedSubstUnion lhs
         (Single (Subst assum to c1)))
       (subsume (Subst assum to e)))
         :ruleset subst)
(rule ((= e (Concat c1 c2))
       (= lhs (Subst assum to e))
       (ExprIsResolved e)
       (ExprIsResolved to))
      ((DelayedSubstUnion lhs
         (Concat (Subst assum to c1)
                 (Subst assum to c2)))
       (subsume (Subst assum to e)))
         :ruleset subst)

;; Control flow
(rule ((= lhs (Subst assum to inner))
       (= inner (Switch pred inputs c1))
       (ExprIsResolved inner))
      ((DelayedSubstUnion lhs
         (Switch (Subst assum to pred)
                 (Subst assum to inputs)
                 c1))
       (subsume (Subst assum to inner)))
         :ruleset subst)
(rule ((= lhs (Subst assum to inner))
       (= inner (If pred inputs c1 c2))
       (ExprIsResolved inner))
      ((DelayedSubstUnion lhs
         (If (Subst assum to pred)
             (Subst assum to inputs)
             c1
             c2))
       (subsume (Subst assum to inner)))
         :ruleset subst)
(rule ((= e (DoWhile in out))
       (= lhs (Subst assum to e))
       (ExprIsResolved e)
       (ExprIsResolved to))
      ((DelayedSubstUnion lhs
         (DoWhile (Subst assum to in)
                  out))
       (subsume (Subst assum to e)))
      :ruleset subst)

;; substitute into function (convenience for testing)
(rewrite (Subst assum to (Function name inty outty body))
         (Function name inty outty (Subst assum to body))
         :when ((ExprIsResolved body))
         :ruleset subst)



;; ########################### Apply subst unions

(rule ((DelayedSubstUnion lhs rhs))
      ((union lhs rhs))
      :ruleset apply-subst-unions)"#;

#[cfg(feature = "eggplant")]
const SUBST_IF_SUBST_RULE: &str = r#"(rule ((= lhs (IfSubst to in))
       (HasArgType to ty)
       (ContextOf to ctx))
      ((union lhs
         (If (Const (Bool true) ty ctx)
           to
           in
           in)))
       :ruleset subst)
"#;

#[cfg(feature = "eggplant")]
const SUBST_ARG_LEAF_RULE: &str = r#"(rule ((= lhs (Subst assum to e))
       (= e (Arg _ty _oldctx))
       )
      ;; add the assumption `to`
      ((DelayedSubstUnion lhs (AddContext assum to))
       (subsume (Subst assum to e)))
      :ruleset subst)
"#;

#[cfg(feature = "eggplant")]
const SUBST_CONST_LEAF_RULE: &str = r#"(rule ((= lhs (Subst assum to e))
       (= e (Const c _ty _oldctx))
       (HasArgType to newty))
      ((DelayedSubstUnion lhs (Const c newty assum))
      (subsume (Subst assum to e)))
      :ruleset subst)
"#;

#[cfg(feature = "eggplant")]
const SUBST_EMPTY_LEAF_RULE: &str = r#"(rule ((= lhs (Subst assum to e))
       (= e (Empty _ty _oldctx))
       (HasArgType to newty))
      ((DelayedSubstUnion lhs (Empty newty assum))
      (subsume (Subst assum to e)))
      :ruleset subst)
"#;

#[cfg(feature = "eggplant")]
pub(crate) mod native {
    use super::super::schema_dsl::{self, ExprRuleCtx};
    use crate::eggplant_backend::peepholes::native::PeepholeTx;
    use eggplant::prelude::{prim_call, IntoHandleTy, PEq, PatRecSgl, RuleRunnerSgl, RuleSetId};

    fn expr_leaf<PR: PatRecSgl>() -> schema_dsl::Expr<PR> {
        schema_dsl::Expr::query_leaf()
    }

    fn type_leaf<PR: PatRecSgl>() -> schema_dsl::Type<PR> {
        schema_dsl::Type::query_leaf()
    }

    fn assumption_leaf<PR: PatRecSgl>() -> schema_dsl::Assumption<PR> {
        schema_dsl::Assumption::query_leaf()
    }

    fn constant_leaf<PR: PatRecSgl>() -> schema_dsl::Constant<PR> {
        schema_dsl::Constant::query_leaf()
    }

    #[eggplant::pat_vars]
    struct IfSubstPat<PR: PatRecSgl> {
        lhs: schema_dsl::Expr,
        to: schema_dsl::Expr,
        input: schema_dsl::Expr,
        ty: schema_dsl::Type,
        ctx: schema_dsl::Assumption,
        has_arg_type: schema_dsl::HasArgType,
        context_of: schema_dsl::ContextOf,
    }

    #[eggplant::pat_vars]
    struct SubstConstPat<PR: PatRecSgl> {
        lhs: schema_dsl::Expr,
        assum: schema_dsl::Assumption,
        to: schema_dsl::Expr,
        input: schema_dsl::Expr,
        constant: schema_dsl::Constant,
        new_ty: schema_dsl::Type,
        has_arg_type: schema_dsl::HasArgType,
    }

    #[eggplant::pat_vars]
    struct SubstArgPat<PR: PatRecSgl> {
        lhs: schema_dsl::Expr,
        assum: schema_dsl::Assumption,
        to: schema_dsl::Expr,
        input: schema_dsl::Expr,
    }

    #[eggplant::pat_vars]
    struct SubstEmptyPat<PR: PatRecSgl> {
        lhs: schema_dsl::Expr,
        assum: schema_dsl::Assumption,
        to: schema_dsl::Expr,
        input: schema_dsl::Expr,
        new_ty: schema_dsl::Type,
        has_arg_type: schema_dsl::HasArgType,
    }

    fn if_subst_pat<PR: PatRecSgl>() -> IfSubstPat<PR> {
        let lhs = expr_leaf::<PR>();
        let to = expr_leaf::<PR>();
        let input = expr_leaf::<PR>();
        let ty = type_leaf::<PR>();
        let ctx = assumption_leaf::<PR>();
        let lhs_is_if_subst = lhs.handle().eq(&prim_call::<schema_dsl::Expr>(
            "IfSubst",
            vec![
                to.handle().into_handle_ty(),
                input.handle().into_handle_ty(),
            ],
        ));
        let has_arg_type = schema_dsl::HasArgType::query_fields(&to, &ty);
        let context_of = schema_dsl::ContextOf::query_fields(&to, &ctx);

        IfSubstPat::new(lhs, to, input, ty, ctx, has_arg_type, context_of).assert(lhs_is_if_subst)
    }

    fn subst_const_pat<PR: PatRecSgl>() -> SubstConstPat<PR> {
        let lhs = expr_leaf::<PR>();
        let assum = assumption_leaf::<PR>();
        let to = expr_leaf::<PR>();
        let input = expr_leaf::<PR>();
        let constant = constant_leaf::<PR>();
        let old_ty = type_leaf::<PR>();
        let old_ctx = assumption_leaf::<PR>();
        let new_ty = type_leaf::<PR>();
        let input_is_const = input
            .handle()
            .eq(&schema_dsl::Const::query(&constant, &old_ty, &old_ctx).handle());
        let lhs_is_subst = lhs.handle().eq(&prim_call::<schema_dsl::Expr>(
            "Subst",
            vec![
                assum.handle().into_handle_ty(),
                to.handle().into_handle_ty(),
                input.handle().into_handle_ty(),
            ],
        ));
        let has_arg_type = schema_dsl::HasArgType::query_fields(&to, &new_ty);

        SubstConstPat::new(lhs, assum, to, input, constant, new_ty, has_arg_type)
            .assert(input_is_const)
            .assert(lhs_is_subst)
    }

    fn subst_arg_pat<PR: PatRecSgl>() -> SubstArgPat<PR> {
        let lhs = expr_leaf::<PR>();
        let assum = assumption_leaf::<PR>();
        let to = expr_leaf::<PR>();
        let input = expr_leaf::<PR>();
        let old_ty = type_leaf::<PR>();
        let old_ctx = assumption_leaf::<PR>();
        let input_is_arg = input
            .handle()
            .eq(&schema_dsl::Arg::query(&old_ty, &old_ctx).handle());
        let lhs_is_subst = lhs.handle().eq(&prim_call::<schema_dsl::Expr>(
            "Subst",
            vec![
                assum.handle().into_handle_ty(),
                to.handle().into_handle_ty(),
                input.handle().into_handle_ty(),
            ],
        ));

        SubstArgPat::new(lhs, assum, to, input)
            .assert(input_is_arg)
            .assert(lhs_is_subst)
    }

    fn subst_empty_pat<PR: PatRecSgl>() -> SubstEmptyPat<PR> {
        let lhs = expr_leaf::<PR>();
        let assum = assumption_leaf::<PR>();
        let to = expr_leaf::<PR>();
        let input = expr_leaf::<PR>();
        let old_ty = type_leaf::<PR>();
        let old_ctx = assumption_leaf::<PR>();
        let new_ty = type_leaf::<PR>();
        let input_is_empty = input
            .handle()
            .eq(&schema_dsl::Empty::query(&old_ty, &old_ctx).handle());
        let lhs_is_subst = lhs.handle().eq(&prim_call::<schema_dsl::Expr>(
            "Subst",
            vec![
                assum.handle().into_handle_ty(),
                to.handle().into_handle_ty(),
                input.handle().into_handle_ty(),
            ],
        ));
        let has_arg_type = schema_dsl::HasArgType::query_fields(&to, &new_ty);

        SubstEmptyPat::new(lhs, assum, to, input, new_ty, has_arg_type)
            .assert(input_is_empty)
            .assert(lhs_is_subst)
    }

    pub(crate) fn register_native_rules() -> RuleSetId {
        let ruleset = RuleSetId("subst");

        PeepholeTx::add_rule("subst_if_subst", ruleset, if_subst_pat, |ctx, pat| {
            let true_value = ctx._intern_base::<bool, bool>(true);
            let constant = eggplant::wrap::Value::<schema_dsl::Constant>::new(
                (ctx).insert("Bool", &[true_value]),
            );
            let condition = ctx.insert_const(constant, pat.ty, pat.ctx);
            let rewritten = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                "If",
                &[condition.val, pat.to.val, pat.input.val, pat.input.val],
            ));

            ctx.union(pat.lhs, rewritten);
        });
        PeepholeTx::add_rule("subst_arg_leaf", ruleset, subst_arg_pat, |ctx, pat| {
            let rewritten = eggplant::wrap::Value::<schema_dsl::Expr>::new(
                (ctx).insert("AddContext", &[pat.assum.val, pat.to.val]),
            );
            let _ = eggplant::wrap::Value::<schema_dsl::Expr>::new(
                (&ctx).insert("DelayedSubstUnion", &[pat.lhs.val, rewritten.val]),
            );
            ctx.subsume("Subst", &[pat.assum.val, pat.to.val, pat.input.val]);
        });
        PeepholeTx::add_rule("subst_const_leaf", ruleset, subst_const_pat, |ctx, pat| {
            let rewritten = eggplant::wrap::Value::<schema_dsl::Expr>::new(
                (&ctx).insert("Const", &[pat.constant.val, pat.new_ty.val, pat.assum.val]),
            );
            let _ = eggplant::wrap::Value::<schema_dsl::Expr>::new(
                (&ctx).insert("DelayedSubstUnion", &[pat.lhs.val, rewritten.val]),
            );
            ctx.subsume("Subst", &[pat.assum.val, pat.to.val, pat.input.val]);
        });
        PeepholeTx::add_rule("subst_empty_leaf", ruleset, subst_empty_pat, |ctx, pat| {
            let rewritten = eggplant::wrap::Value::<schema_dsl::Expr>::new(
                (ctx).insert("Empty", &[pat.new_ty.val, pat.assum.val]),
            );
            let _ = eggplant::wrap::Value::<schema_dsl::Expr>::new(
                (&ctx).insert("DelayedSubstUnion", &[pat.lhs.val, rewritten.val]),
            );
            ctx.subsume("Subst", &[pat.assum.val, pat.to.val, pat.input.val]);
        });

        ruleset
    }
}

#[cfg(all(test, feature = "eggplant"))]
mod native_tests {
    use crate::eggplant_backend::test_lock;
    use egglog::EGraph;

    fn if_subst_candidate_parts() -> (String, String, String) {
        let to = "(Const (Int 3) (Base (IntT)) (InFunc \"RLCR\"))".to_string();
        let input = "(Arg (Base (IntT)) (InFunc \"OLD\"))".to_string();
        let expected = "(If (Const (Bool true) (Base (IntT)) (InFunc \"RLCR\")) (Const (Int 3) (Base (IntT)) (InFunc \"RLCR\")) (Arg (Base (IntT)) (InFunc \"OLD\")) (Arg (Base (IntT)) (InFunc \"OLD\")))".to_string();
        (to, input, expected)
    }

    fn subst_schedule() -> String {
        format!(
            "(run-schedule {})\n(run-schedule (saturate subst))\n",
            crate::schedule::types_and_indexing()
        )
    }

    fn subst_leaf_schedule() -> String {
        format!("{}(run-schedule apply-subst-unions)\n", subst_schedule())
    }

    fn subst_arg_leaf_schedule() -> String {
        format!(
            "{}(run-schedule (saturate context))\n",
            subst_leaf_schedule()
        )
    }

    fn text_if_subst_holds(prologue: &str, to: &str, input: &str, expected: &str, schedule: &str) {
        let program = format!(
            "{prologue}\n(let __rlcr_to {to})\n(let __rlcr_input {input})\n(let __rlcr_subst (IfSubst __rlcr_to __rlcr_input))\n{schedule}(check (= __rlcr_subst {expected}))\n"
        );
        let mut egraph = EGraph::default();
        egraph.parse_and_run_program(None, &program).unwrap();
    }

    fn text_subst_leaf_holds(
        prologue: &str,
        assumption: &str,
        to: &str,
        input: &str,
        expected: &str,
        schedule: &str,
    ) {
        let program = format!(
            "{prologue}\n(let __rlcr_assumption {assumption})\n(let __rlcr_to {to})\n(let __rlcr_input {input})\n(let __rlcr_subst (Subst __rlcr_assumption __rlcr_to __rlcr_input))\n{schedule}(check (= __rlcr_subst {expected}))\n"
        );
        let mut egraph = EGraph::default();
        egraph.parse_and_run_program(None, &program).unwrap();
    }

    fn native_if_subst_holds(
        prologue: &str,
        to: &str,
        input: &str,
        expected: &str,
        schedule: &str,
        ablate: Option<&str>,
    ) -> std::result::Result<(), eggplant::egglog::Error> {
        let initialization = format!(
            "(let __rlcr_to {to})\n(let __rlcr_input {input})\n(let __rlcr_subst (IfSubst __rlcr_to __rlcr_input))"
        );
        crate::with_native_rules_egraph(prologue, &initialization, schedule, ablate, |egraph| {
            egraph.parse_and_run_program(None, &format!("(check (= __rlcr_subst {expected}))"))?;
            Ok(())
        })
    }

    fn native_subst_leaf_holds(
        prologue: &str,
        assumption: &str,
        to: &str,
        input: &str,
        expected: &str,
        schedule: &str,
        ablate: Option<&str>,
    ) -> std::result::Result<(), eggplant::egglog::Error> {
        let initialization = format!(
            "(let __rlcr_assumption {assumption})\n(let __rlcr_to {to})\n(let __rlcr_input {input})\n(let __rlcr_subst (Subst __rlcr_assumption __rlcr_to __rlcr_input))"
        );
        crate::with_native_rules_egraph(prologue, &initialization, schedule, ablate, |egraph| {
            egraph.parse_and_run_program(None, &format!("(check (= __rlcr_subst {expected}))"))?;
            Ok(())
        })
    }

    fn subst_const_candidate_parts() -> (String, String, String, String) {
        let assumption = "(InFunc \"RLCR\")".to_string();
        let to = "(Const (Int 3) (Base (IntT)) (InFunc \"TO\"))".to_string();
        let input = "(Const (Int 9) (Base (IntT)) (InFunc \"OLD\"))".to_string();
        let expected = "(Const (Int 9) (Base (IntT)) (InFunc \"RLCR\"))".to_string();
        (assumption, to, input, expected)
    }

    fn subst_arg_candidate_parts() -> (String, String, String, String) {
        let assumption = "(InFunc \"RLCR\")".to_string();
        let to = "(Const (Int 3) (Base (IntT)) (InFunc \"TO\"))".to_string();
        let input = "(Arg (Base (IntT)) (InFunc \"OLD\"))".to_string();
        let expected = "(Const (Int 3) (Base (IntT)) (InFunc \"RLCR\"))".to_string();
        (assumption, to, input, expected)
    }

    fn subst_empty_candidate_parts() -> (String, String, String, String) {
        let assumption = "(InFunc \"RLCR\")".to_string();
        let to = "(Const (Int 3) (Base (IntT)) (InFunc \"TO\"))".to_string();
        let input = "(Empty (Base (IntT)) (InFunc \"OLD\"))".to_string();
        let expected = "(Empty (Base (IntT)) (InFunc \"RLCR\"))".to_string();
        (assumption, to, input, expected)
    }

    #[test]
    fn native_feature_path_matches_text_backend_for_subst_if_subst_case() {
        let _guard = test_lock::lock();
        let (to, input, expected) = if_subst_candidate_parts();
        let schedule = subst_schedule();

        text_if_subst_holds(
            &crate::prologue_egglog_text(),
            &to,
            &input,
            &expected,
            &schedule,
        );
        native_if_subst_holds(
            &crate::feature_execution_prologue(true, None),
            &to,
            &input,
            &expected,
            &schedule,
            None,
        )
        .unwrap();

        let ablated = native_if_subst_holds(
            &crate::feature_execution_prologue(true, Some("subst")),
            &to,
            &input,
            &expected,
            &schedule,
            Some("subst"),
        );

        assert!(
            ablated.is_err(),
            "ablating subst should make the IfSubst witness fail",
        );
    }

    #[test]
    fn native_feature_path_matches_text_backend_for_subst_const_leaf_case() {
        let _guard = test_lock::lock();
        let (assumption, to, input, expected) = subst_const_candidate_parts();
        let schedule = subst_leaf_schedule();

        text_subst_leaf_holds(
            &crate::prologue_egglog_text(),
            &assumption,
            &to,
            &input,
            &expected,
            &schedule,
        );
        native_subst_leaf_holds(
            &crate::feature_execution_prologue(true, None),
            &assumption,
            &to,
            &input,
            &expected,
            &schedule,
            None,
        )
        .unwrap();

        let ablated = native_subst_leaf_holds(
            &crate::feature_execution_prologue(true, Some("subst")),
            &assumption,
            &to,
            &input,
            &expected,
            &schedule,
            Some("subst"),
        );

        assert!(
            ablated.is_err(),
            "ablating subst should make the Const leaf witness fail",
        );
    }

    #[test]
    fn native_feature_path_matches_text_backend_for_subst_arg_leaf_case() {
        let _guard = test_lock::lock();
        let (assumption, to, input, expected) = subst_arg_candidate_parts();
        let schedule = subst_arg_leaf_schedule();

        text_subst_leaf_holds(
            &crate::prologue_egglog_text(),
            &assumption,
            &to,
            &input,
            &expected,
            &schedule,
        );
        native_subst_leaf_holds(
            &crate::feature_execution_prologue(true, None),
            &assumption,
            &to,
            &input,
            &expected,
            &schedule,
            None,
        )
        .unwrap();

        let ablated = native_subst_leaf_holds(
            &crate::feature_execution_prologue(true, Some("subst")),
            &assumption,
            &to,
            &input,
            &expected,
            &schedule,
            Some("subst"),
        );

        assert!(
            ablated.is_err(),
            "ablating subst should make the Arg leaf witness fail",
        );
    }

    #[test]
    fn native_feature_path_matches_text_backend_for_subst_empty_leaf_case() {
        let _guard = test_lock::lock();
        let (assumption, to, input, expected) = subst_empty_candidate_parts();
        let schedule = subst_leaf_schedule();

        text_subst_leaf_holds(
            &crate::prologue_egglog_text(),
            &assumption,
            &to,
            &input,
            &expected,
            &schedule,
        );
        native_subst_leaf_holds(
            &crate::feature_execution_prologue(true, None),
            &assumption,
            &to,
            &input,
            &expected,
            &schedule,
            None,
        )
        .unwrap();

        let ablated = native_subst_leaf_holds(
            &crate::feature_execution_prologue(true, Some("subst")),
            &assumption,
            &to,
            &input,
            &expected,
            &schedule,
            Some("subst"),
        );

        assert!(
            ablated.is_err(),
            "ablating subst should make the Empty leaf witness fail",
        );
    }
}
