pub(crate) fn fragment() -> String {
    let mut out = String::new();
    out.push_str(GENERATED_MARKER);
    out.push_str(LOOP_STRENGTH_REDUCTION);
    out.push('\n');
    out
}

#[cfg(feature = "eggplant")]
pub(crate) fn native_fragment() -> String {
    let mut out = String::new();
    out.push_str(GENERATED_MARKER);
    out.push_str(LOOP_STRENGTH_REDUCTION_SUPPORT);
    out.push('\n');
    out
}

const GENERATED_MARKER: &str =
    "; (Generated from eggplant Rust: src/eggplant_backend/loop_strength_reduction.rs)\n";
const LOOP_STRENGTH_REDUCTION_SUPPORT: &str = r#";; ORIGINAL
;; a = 0
;; c = 3
;; for  i = 0 to n:
;;     a = i * c
;;
;; OPTIMIZED
;; a = 0
;; c = 3
;; d = 0
;; for i = 0 to n:
;;     a += d
;;     d += c

(ruleset loop-strength-reduction)

; Finds invariants/constants within a body.
; Columns: body; value of invariant in inputs; value of invariant in outputs
;; Get the input and output value of an invariant, or constant int, within the loop
;;             loop in   out
(relation LsrInv (Expr Expr Expr))

; Private temporary context for native feature-path execution.
(constructor LsrTmpCtx (Expr Expr Expr) Assumption)"#;
const LOOP_STRENGTH_REDUCTION: &str = r#";; ORIGINAL
;; a = 0
;; c = 3
;; for  i = 0 to n:
;;     a = i * c
;;
;; OPTIMIZED
;; a = 0
;; c = 3
;; d = 0
;; for i = 0 to n:
;;     a += d
;;     d += c
(ruleset loop-strength-reduction)

; Finds invariants/constants within a body.
; Columns: body; value of invariant in inputs; value of invariant in outputs
;; Get the input and output value of an invariant, or constant int, within the loop
;;             loop in   out
(relation LsrInv (Expr Expr Expr))

; TODO: there may be a bug with finding the invariant, or it just may not be extracted.
; Can make this work on loop_with_mul_by_inv and a rust test later.
; (rule (
;     (= loop (DoWhile inputs pred-and-body))
;     (= (Get outputs (+ i 1)) (Get (Arg arg-type assm) i)))
;     ((inv loop (Get inputs i) (Get (Arg arg-type assm) i))) :ruleset always-run)
(rule (
    (= loop (DoWhile inputs pred-and-body))
    (ContextOf inputs loop-input-ctx)
    (ContextOf pred-and-body loop-output-ctx)
    (= constant (Const c out-type loop-output-ctx))
    (HasArgType inputs in-type)
    )
    ((LsrInv loop (Const c in-type loop-input-ctx) constant)) :ruleset always-run)

(rule 
    (
        ;; Find loop
        (= old-loop (DoWhile inputs pred-and-outputs))
        (ContextOf pred-and-outputs loop-ctx)

        ; Find loop variable (argument that gets incremented with an invariant)
        (LsrInv old-loop loop-incr-in loop-incr-out)
        ; Since the first el of pred-and-outputs is the pred, we need to offset i
        (= (Get pred-and-outputs (+ i 1)) (Bop (Add) (Get (Arg arg-type assm) i) loop-incr-out))

        ; Find invariant where input is same as output, or constant
        (LsrInv old-loop c-in c-out)

        ; Find multiplication of loop variable and invariant
        (= old-mul (Bop (Mul) c-out (Get (Arg arg-type assm) i)))
        (ContextOf old-mul loop-ctx)

        (= arg-type (TupleT ty-list))
        ; n is index of our new, temporary variable d
        (= n (tuple-length inputs))
    )
    (
        ; Each time we need to update d by the product of the multiplied constant and the loop increment
        (let addend (Bop (Mul) c-out loop-incr-out))

        ; Initial value of d is i * c
        (let d-init (Bop (Mul) c-in (Get inputs i)))

        ; Construct optimized theta
        ; new-inputs already has the correct context
        (let new-inputs (Concat inputs (Single d-init)))

        ; We need to create a new type, with one more input
        (let new-arg-ty (TupleT (TLConcat ty-list (TCons (IntT) (TNil)))))
        (let replace-arg (SubTuple (Arg new-arg-ty (TmpCtx)) 0 n))

        ; Value of d in loop. Add context to addend
        (let d-out (Bop (Add) (Get (Arg new-arg-ty (TmpCtx)) n)
                   (Subst (TmpCtx) replace-arg addend)))

        ; build the old body, making sure to set the correct arg type and context
        (let new-body
          (Concat
            (Subst (TmpCtx) replace-arg pred-and-outputs)
            (Single d-out)))

        (let new-loop (DoWhile new-inputs new-body))

        (let new-c (Subst (TmpCtx) replace-arg c-out))

        ; Now that we have the new loop, union the temporary context with the actual ctx
        (union (TmpCtx) (InLoop new-inputs new-body))

        ; Substitute d for the *i expression
        (let new-mul
            (Bop (Mul) new-c (Get replace-arg i)))
        (union (Get (Arg new-arg-ty (TmpCtx)) n) new-mul)

        ; Subsume the multiplication in the new loop to prevent
        ; from firing loop strength reduction again on the new loop
        ; Workaround of egglog issue: https://github.com/egraphs-good/egglog/issues/462
        ; add the expression we are about to subsume
        (let before
          (Bop (Mul) new-c (Get replace-arg i)))
        ; now subsume it
        (subsume
          (Bop (Mul) new-c (Get replace-arg i)))

        ; Project all but last
        (union old-loop (SubTuple new-loop 0 n))
        (delete (TmpCtx))
    )
    :ruleset loop-strength-reduction
)"#;

#[cfg(feature = "eggplant")]
pub(crate) mod native {
    use super::super::schema_dsl;
    use crate::eggplant_backend::peepholes::native::PeepholeTx;
    use crate::eggplant_backend::schema_dsl::LsrInvPRRuleCtx;
    use eggplant::prelude::{
        AsHandle, Insertable, PEq, PatRecSgl, RuleRunnerSgl, RuleSetId,
    };

    #[eggplant::pat_vars]
    struct LoopStrengthReductionConstInvariantPat<PR: PatRecSgl> {
        old_loop: schema_dsl::DoWhile,
        loop_input_ctx: schema_dsl::Assumption,
        in_type: schema_dsl::Type,
        constant: schema_dsl::Const,
        c: schema_dsl::Constant,
        inputs_context: schema_dsl::ContextOf,
        body_context: schema_dsl::ContextOf,
        inputs_have_type: schema_dsl::HasArgType,
    }

    fn loop_strength_reduction_const_invariant_pat<PR: PatRecSgl>(
    ) -> LoopStrengthReductionConstInvariantPat<PR> {
        let inputs = schema_dsl::Expr::query_leaf();
        let pred_and_body = schema_dsl::Expr::query_leaf();
        let old_loop = schema_dsl::DoWhile::query(&inputs, &pred_and_body);
        let loop_input_ctx = schema_dsl::Assumption::query_leaf();
        let loop_output_ctx = schema_dsl::Assumption::query_leaf();
        let c = schema_dsl::Constant::query_leaf();
        let out_type = schema_dsl::Type::query_leaf();
        let constant = schema_dsl::Const::query(&c, &out_type, &loop_output_ctx);
        let in_type = schema_dsl::Type::query_leaf();

        let inputs_context = schema_dsl::ContextOf::query_fields(&inputs, &loop_input_ctx);
        let body_context = schema_dsl::ContextOf::query_fields(&pred_and_body, &loop_output_ctx);
        let inputs_have_type = schema_dsl::HasArgType::query_fields(&inputs, &in_type);

        LoopStrengthReductionConstInvariantPat::new(
            old_loop,
            loop_input_ctx,
            in_type,
            constant,
            c,
            inputs_context,
            body_context,
            inputs_have_type,
        )
    }

    #[eggplant::pat_vars]
    struct LoopStrengthReductionPat<PR: PatRecSgl> {
        inputs: schema_dsl::Expr,
        pred_and_outputs: schema_dsl::Expr,
        loop_ctx: schema_dsl::Assumption,
        loop_incr_in: schema_dsl::Expr,
        loop_incr_out: schema_dsl::Expr,
        c_in: schema_dsl::Expr,
        c_out: schema_dsl::Expr,
        ty_list: schema_dsl::TypeList,
        arg_i: schema_dsl::Get,
        old_loop: schema_dsl::DoWhile,
        pred_and_outputs_in_loop: schema_dsl::ContextOf,
        loop_increment: schema_dsl::LsrInv,
        invariant: schema_dsl::LsrInv,
        mul_in_loop: schema_dsl::ContextOf,
    }

    fn loop_strength_reduction_pat<PR: PatRecSgl>() -> LoopStrengthReductionPat<PR> {
        let inputs = schema_dsl::Expr::query_leaf();
        let pred_and_outputs = schema_dsl::Expr::query_leaf();
        let old_loop = schema_dsl::DoWhile::query(&inputs, &pred_and_outputs);
        let loop_ctx = schema_dsl::Assumption::query_leaf();
        let loop_incr_in = schema_dsl::Expr::query_leaf();
        let loop_incr_out = schema_dsl::Expr::query_leaf();
        let c_in = schema_dsl::Expr::query_leaf();
        let c_out = schema_dsl::Expr::query_leaf();
        let ty_list = schema_dsl::TypeList::query_leaf();
        let arg_type = schema_dsl::TupleT::query(&ty_list);
        let assm = schema_dsl::Assumption::query_leaf();
        let arg = schema_dsl::Arg::query(&arg_type, &assm);
        let arg_i = schema_dsl::Get::query(&arg);
        let body_out = schema_dsl::Get::query(&pred_and_outputs);
        let add = schema_dsl::Bop::query(&schema_dsl::Add::query(), &arg_i, &loop_incr_out);
        let old_mul = schema_dsl::Bop::query(&schema_dsl::Mul::query(), &c_out, &arg_i);

        let pred_and_outputs_in_loop =
            schema_dsl::ContextOf::query_fields(&pred_and_outputs, &loop_ctx);
        let loop_increment =
            schema_dsl::LsrInv::query_fields(&old_loop, &loop_incr_in, &loop_incr_out);
        let body_out_matches = body_out.handle().eq(&add.handle());
        let body_index_matches = body_out
            .handle_index()
            .eq(&(arg_i.handle_index() + (&1_i64).as_handle()));
        let invariant = schema_dsl::LsrInv::query_fields(&old_loop, &c_in, &c_out);
        let mul_in_loop = schema_dsl::ContextOf::query_fields(&old_mul, &loop_ctx);

        LoopStrengthReductionPat::new(
            inputs,
            pred_and_outputs,
            loop_ctx,
            loop_incr_in,
            loop_incr_out,
            c_in,
            c_out,
            ty_list,
            arg_i,
            old_loop,
            pred_and_outputs_in_loop,
            loop_increment,
            invariant,
            mul_in_loop,
        )
        .assert(body_out_matches)
        .assert(body_index_matches)
    }

    pub(crate) fn register_native_rules() -> RuleSetId {
        let ruleset = RuleSetId("loop-strength-reduction");
        let always_run = RuleSetId("always-run");

        PeepholeTx::add_rule(
            "loop_strength_reduction_const_invariant",
            always_run,
            loop_strength_reduction_const_invariant_pat,
            |ctx, pat| {
                let input_constant = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert("Const", &[pat.c.val, pat.in_type.val, pat.loop_input_ctx.val]));
                ctx.insert_lsr_inv(pat.old_loop, input_constant, pat.constant);
            },
        );

        PeepholeTx::add_rule(
            "loop_strength_reduction",
            ruleset,
            loop_strength_reduction_pat,
            |ctx, pat| {
                let zero = 0_i64.to_value(&ctx).val;
                let n = ctx.lookup_expect("tuple-length", &[pat.inputs.to_value(&ctx).val]);

                let mul_op = eggplant::wrap::Value::<schema_dsl::BinaryOp>::new((ctx).insert("Mul", &[]));
                let add_op = eggplant::wrap::Value::<schema_dsl::BinaryOp>::new((ctx).insert("Add", &[]));

                let addend = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert("Bop", &[
                        mul_op.val,
                        pat.c_out.to_value(&ctx).val,
                        pat.loop_incr_out.to_value(&ctx).val,
                    ]));
                let input_i = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert("Get", &[pat.inputs.to_value(&ctx).val, pat.arg_i.index.val]));
                let d_init = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert("Bop", &[
                        mul_op.val,
                        pat.c_in.to_value(&ctx).val,
                        input_i.to_value(&ctx).val,
                    ]));
                let new_inputs = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert("Concat", &[
                        pat.inputs.to_value(&ctx).val,
                        eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert("Single", &[d_init.to_value(&ctx).val]))
                        .to_value(&ctx)
                        .val,
                    ]));

                let int_ty = eggplant::wrap::Value::<schema_dsl::BaseType>::new((ctx).insert("IntT", &[]));
                let tnil = eggplant::wrap::Value::<schema_dsl::TypeList>::new((ctx).insert("TNil", &[]));
                let appended_tail =
                    eggplant::wrap::Value::<schema_dsl::TypeList>::new((ctx).insert("TCons", &[int_ty.val, tnil.val]));
                let new_ty_list = eggplant::wrap::Value::<schema_dsl::TypeList>::new((&ctx).insert("TLConcat", &[
                        pat.ty_list.to_value(&ctx).val,
                        appended_tail.to_value(&ctx).val,
                    ]));
                let new_arg_ty =
                    eggplant::wrap::Value::<schema_dsl::Type>::new((ctx).insert("TupleT", &[new_ty_list.val]));
                let tmp_ctx = eggplant::wrap::Value::<schema_dsl::Assumption>::new((&ctx).insert("LsrTmpCtx", &[
                        new_inputs.to_value(&ctx).val,
                        pat.pred_and_outputs.to_value(&ctx).val,
                        pat.c_out.to_value(&ctx).val,
                    ]));
                let tmp_arg = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert("Arg", &[new_arg_ty.to_value(&ctx).val, tmp_ctx.to_value(&ctx).val]));
                let replace_arg = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert("SubTuple", &[tmp_arg.to_value(&ctx).val, zero, n]));

                let d_out = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert("Bop", &[
                        add_op.val,
                        eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert("Get", &[tmp_arg.to_value(&ctx).val, n]))
                        .to_value(&ctx)
                        .val,
                        eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert("Subst", &[
                                tmp_ctx.to_value(&ctx).val,
                                replace_arg.to_value(&ctx).val,
                                addend.to_value(&ctx).val,
                            ]))
                        .to_value(&ctx)
                        .val,
                    ]));
                let new_body = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert("Concat", &[
                        eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert("Subst", &[
                                tmp_ctx.to_value(&ctx).val,
                                replace_arg.to_value(&ctx).val,
                                pat.pred_and_outputs.to_value(&ctx).val,
                            ]))
                        .to_value(&ctx)
                        .val,
                        eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert("Single", &[d_out.to_value(&ctx).val]))
                        .to_value(&ctx)
                        .val,
                    ]));
                let new_loop = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert("DoWhile", &[new_inputs.to_value(&ctx).val, new_body.to_value(&ctx).val]));
                let new_c = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert("Subst", &[
                        tmp_ctx.to_value(&ctx).val,
                        replace_arg.to_value(&ctx).val,
                        pat.c_out.to_value(&ctx).val,
                    ]));
                let loop_ctx = eggplant::wrap::Value::<schema_dsl::Assumption>::new((&ctx).insert("InLoop", &[new_inputs.to_value(&ctx).val, new_body.to_value(&ctx).val]));
                ctx.union(tmp_ctx, loop_ctx);

                let new_mul_input = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert("Get", &[replace_arg.to_value(&ctx).val, pat.arg_i.index.val]));
                let new_mul = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert("Bop", &[
                        mul_op.val,
                        new_c.to_value(&ctx).val,
                        new_mul_input.to_value(&ctx).val,
                    ]));
                let tmp_arg_n =
                    eggplant::wrap::Value::<schema_dsl::Expr>::new((ctx).insert("Get", &[tmp_arg.to_value(&ctx).val, n]));
                ctx.union(tmp_arg_n, new_mul);
                ctx.subsume(
                    "Bop",
                    &[
                        mul_op.val,
                        new_c.to_value(&ctx).val,
                        new_mul_input.to_value(&ctx).val,
                    ],
                );

                let projected_loop = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert("SubTuple", &[new_loop.to_value(&ctx).val, zero, n]));
                ctx.union(pat.old_loop, projected_loop);
                ctx.remove(
                    "LsrTmpCtx",
                    &[
                        new_inputs.to_value(&ctx).val,
                        pat.pred_and_outputs.to_value(&ctx).val,
                        pat.c_out.to_value(&ctx).val,
                    ],
                );
            },
        );

        ruleset
    }
}

#[cfg(all(test, feature = "eggplant"))]
mod native_tests {
    use crate::ast::*;
    use crate::eggplant_backend::test_lock;
    use crate::schedule;
    use egglog::{ast::Expr as EgglogExpr, EGraph, TermDag};

    fn candidate_and_expected_exprs() -> (String, String) {
        let candidate = dowhile(
            parallel!(int(0), int(0)),
            parallel!(
                less_than(add(getat(0), int(1)), int(8)),
                add(getat(0), int(1)),
                mul(int(3), getat(0))
            ),
        )
        .add_arg_type(emptyt())
        .to_string();

        let optimized_inputs = concat(parallel!(int(0), int(0)), single(int(0)));
        let optimized_body_prefix = parallel!(
            less_than(add(getat(0), int(1)), int(8)),
            add(getat(0), int(1)),
            getat(2)
        );
        let expected = dowhile(
            optimized_inputs,
            concat(optimized_body_prefix, single(add(getat(2), int(3)))),
        )
        .add_arg_type(emptyt())
        .to_string();

        (candidate, expected)
    }

    fn focused_schedule() -> String {
        let helpers = schedule::helpers();
        format!("(run-schedule\n{helpers}\nloop-strength-reduction\n{helpers}\n)")
    }

    fn eval_and_extract_text_expr(prologue: &str, expr: &str, schedule: &str) -> String {
        let binding = "__rlcr_text_expr";
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
        let binding = "__rlcr_native_expr";
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

    fn native_feature_op_count(
        prologue: &str,
        expr: &str,
        schedule: &str,
        ablate: Option<&str>,
        op: &str,
    ) -> usize {
        let initialization = format!("(let __rlcr_native_expr {expr})");
        crate::with_native_rules_egraph(prologue, &initialization, schedule, ablate, |egraph| {
            Ok(egraph
                .get_function(op)
                .map(|_| egraph.get_size(op))
                .unwrap_or_default())
        })
        .unwrap()
    }

    #[test]
    fn native_feature_path_matches_text_backend_for_loop_strength_reduction_case() {
        let _guard = test_lock::lock();
        let (expr, _expected) = candidate_and_expected_exprs();
        let schedule = focused_schedule();
        let text_extracted =
            eval_and_extract_text_expr(&crate::prologue_egglog_text(), &expr, &schedule);
        let native_extracted = eval_and_extract_native_expr(
            &crate::feature_execution_prologue(true, None),
            &expr,
            &schedule,
            None,
        );
        let native_lsr_tmp_ctx_count = native_feature_op_count(
            &crate::feature_execution_prologue(true, None),
            &expr,
            &schedule,
            None,
            "LsrTmpCtx",
        );
        let ablated_native_lsr_tmp_ctx_count = native_feature_op_count(
            &crate::feature_execution_prologue(true, Some("loop-strength-reduction")),
            &expr,
            &schedule,
            Some("loop-strength-reduction"),
            "LsrTmpCtx",
        );

        assert!(
            native_extracted == text_extracted,
            "native feature path should preserve the extracted representative seen in the text backend",
        );
        assert!(
            native_lsr_tmp_ctx_count > 0,
            "native feature path should materialize the private LsrTmpCtx artifact when loop-strength-reduction fires",
        );
        assert_eq!(
            ablated_native_lsr_tmp_ctx_count, 0,
            "ablating the native loop-strength-reduction ruleset should remove the native-only LsrTmpCtx artifact",
        );
    }
}
