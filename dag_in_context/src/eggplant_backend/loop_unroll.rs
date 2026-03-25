pub(crate) fn fragment() -> String {
    let mut out = String::new();
    out.push_str(GENERATED_MARKER);
    out.push_str(LOOP_UNROLL);
    out.push('\n');
    out
}

#[cfg(feature = "eggplant")]
pub(crate) fn native_fragment() -> String {
    let (support_only, _) = LOOP_UNROLL
        .split_once("\n;; unroll a loop with constant bounds and initial value\n")
        .expect("loop_unroll::native_fragment() expects the generated unroll rule marker");
    let mut out = String::new();
    out.push_str(GENERATED_MARKER);
    out.push_str("(constructor LoopUnrollTmpCtx () Assumption)\n");
    out.push_str(&support_only.replace("TmpCtx", "LoopUnrollTmpCtx"));
    out.push('\n');
    out
}

const GENERATED_MARKER: &str =
    "; (Generated from eggplant Rust: src/eggplant_backend/loop_unroll.rs)\n";
const LOOP_UNROLL: &str = r#";; Some simple simplifications of loops
(ruleset loop-unroll)
(ruleset loop-iters-analysis)


;; by default, guess that all loops run 1000 times
(rule ((DoWhile inputs outputs))
      ((set (LoopNumItersGuess inputs outputs) 1000))
      :ruleset loop-iters-analysis)

;; For a loop that is false, its num iters is 1
(rule 
  ((= loop (DoWhile inputs outputs))
   (= (Const (Bool false) ty ctx) (Get outputs 0)))
  ((set (LoopNumItersGuess inputs outputs) 1))
:ruleset loop-iters-analysis)

;; Figure out number of iterations for a loop with constant bounds and initial value
;; and i is updated before checking pred
;; TODO: we could make it work for decrementing loops
(rule
  ((= lhs (DoWhile inputs outputs))
   (= pred (Get outputs 0))
   ;; iteration counter starts at start_const
   (= (Const (Int start_const) _ty1 _ctx1) (Get inputs counter_i))
   ;; updated counter at counter_i
   (= next_counter (Get outputs (+ counter_i 1)))
   ;; increments by some constant each loop
   (= next_counter (Bop (Add) (Get (Arg _ty _ctx) counter_i)
                              (Const (Int increment) _ty2 _ctx2)))
   (> increment 0)
   ;; while next_counter less than end_constant
   (= pred (Bop (LessThan) next_counter
                           (Const (Int end_constant) _ty3 _ctx3)))
   ;; end constant is at least start constant
   (>= end_constant start_const)
  )
  (
    (set (LoopNumItersGuess inputs outputs) (/ (- end_constant start_const) increment))
  )
  :ruleset loop-iters-analysis)

;; Figure out number of iterations for a loop with constant bounds and initial value
;; and i is updated after checking pred
(rule
  ((= lhs (DoWhile inputs outputs))
   (= pred (Get outputs 0))
   ;; iteration counter starts at start_const
   (= (Const (Int start_const) _ty1 _ctx1) (Get inputs counter_i))
   (= body-arg (Get (Arg _ty _ctx) counter_i))
   ;; updated counter at counter_i
   (= next_counter (Get outputs (+ counter_i 1)))
   ;; increments by a constant each loop
   (= next_counter (Bop (Add) body-arg
                              (Const (Int increment) _ty2 _ctx2)))
   (> increment 0)
   ;; while this counter less than end_constant
   (= pred (Bop (LessThan) body-arg
                           (Const (Int end_constant) _ty3 _ctx3)))
   ;; end constant is at least start constant
   (>= end_constant start_const)
  )
  (
    (set (LoopNumItersGuess inputs outputs) (+ (/ (- end_constant start_const) increment) 1))
  )
  :ruleset loop-iters-analysis)

;; loop peeling rule
;; Only peel loops that we know iterate < 3 times
;; (constructor LoopPeeledPlaceholder (Expr) Assumption :unextractable)

;; WARNING: THIS RULE DOES NOT MAINTAIN WEAK LINEARITY
;; If another already peeled loop exists in the egraph, it can violate the invariant.
;; See the eggcc paper for what rules are safe.
;; (ruleset loop-peel)
;;(rule
;; ((= lhs (DoWhile inputs outputs))
;;  (ContextOf lhs ctx)
;;  (HasType inputs inputs-ty)
;;  (= outputs-len (tuple-length outputs))
;;  (= old_cost (LoopNumItersGuess inputs outputs))
;;  (< old_cost 3)
;;  )
;; (
;;  (let executed-once
;;    (Subst ctx inputs outputs))
;;  (let executed-once-body
;;     (SubTuple executed-once 1 (- outputs-len 1)))
;;  (let then-ctx
;;    (InIf true (Get executed-once 0) executed-once-body))
;;  (let else-ctx
;;    (InIf false (Get executed-once 0) executed-once-body))
;;
;;  (let new-loop-arg
;;    (Arg inputs-ty then-ctx))
;;  (let new-loop-body
;;    (Subst (LoopPeeledPlaceholder lhs) new-loop-arg outputs))
;;  (union (InLoop new-loop-arg new-loop-body) (LoopPeeledPlaceholder lhs))
;;
;;  (union lhs
;;    ;; check if we need to continue executing the loop
;;    (If (Get executed-once 0)
;;      executed-once-body ;; inputs are the body executed once
;;      (DoWhile new-loop-arg new-loop-body)
;;      (Arg inputs-ty else-ctx)))
;;
;;  (set (LoopNumItersGuess new-loop-arg new-loop-body) (- old_cost 1))
;;  )
;; :ruleset loop-peel)
;;
;; unroll a loop with constant bounds and initial value
(rule
  ((= lhs (DoWhile inputs outputs))
   (= num-inputs (tuple-length inputs))
   (= pred (Get outputs 0))
   ;; iteration counter starts at start_const
   (= (Const (Int start_const) _ty1 _ctx1) (Get inputs counter_i))
   ;; updated counter at counter_i
   (= next_counter (Get outputs (+ counter_i 1)))
   ;; increments by one each loop
   (= next_counter (Bop (Add) (Get (Arg _ty _ctx) counter_i)
                              (Const (Int 1) _ty2 _ctx2)))
   ;; while less than end_constant
   (= pred (Bop (LessThan) next_counter
                           (Const (Int end_constant) _ty3 _ctx3)))
   ;; start and end constant is a multiple of 4 and greater than start_const
   (> end_constant start_const)
   (= (% start_const 4) 0)
   (= (% end_constant 4) 0)
   (= old_cost (LoopNumItersGuess inputs outputs))
  )
  (
    (let one-iter (SubTuple outputs 1 num-inputs))
    (let unrolled
        (Subst (TmpCtx) one-iter
          (Subst (TmpCtx) one-iter
            (Subst (TmpCtx) one-iter
               outputs))))
    (union lhs
      (DoWhile inputs
        unrolled))
    (let actual-ctx (InLoop inputs unrolled))
    (union (TmpCtx) actual-ctx)

    (set (LoopNumItersGuess inputs unrolled) (/ old_cost 4))
    (delete (TmpCtx))
  )
  :ruleset loop-unroll)"#;

#[cfg(feature = "eggplant")]
pub(crate) mod native {
        use super::super::schema_dsl;
    use crate::eggplant_backend::peepholes::native::PeepholeTx;
    use eggplant::prelude::{
        AsHandle, BaseVar, Insertable, PEq, PatRecSgl, RuleRunnerSgl, RuleSetId,
    };

    #[eggplant::pat_vars]
    struct LoopUnrollPat<PR: PatRecSgl> {
        lhs: schema_dsl::DoWhile,
        inputs: schema_dsl::Expr,
        outputs: schema_dsl::Expr,
        start_const: schema_dsl::Int,
        end_const: schema_dsl::Int,
    }

    fn loop_unroll_pat<PR: PatRecSgl>() -> LoopUnrollPat<PR> {
        let inputs = schema_dsl::Expr::query_leaf();
        let outputs = schema_dsl::Expr::query_leaf();
        let lhs = schema_dsl::DoWhile::query(&inputs, &outputs);

        let pred = schema_dsl::Get::query(&outputs);
        let pred_is_first = pred.handle_index().eq(&(&0_i64).as_handle());

        let counter_i = BaseVar::<i64, PR>::query_named("loop_unroll_counter_i");
        let input_counter = schema_dsl::Get::query(&inputs);
        let input_index_matches = input_counter.handle_index().eq(&counter_i.handle());

        let start_ty = schema_dsl::Type::query_leaf();
        let start_ctx = schema_dsl::Assumption::query_leaf();
        let start_const = schema_dsl::Int::query();
        let start_expr = schema_dsl::Const::query(&start_const, &start_ty, &start_ctx);
        let input_matches_start = input_counter.handle().eq(&start_expr.handle());

        let next_counter = schema_dsl::Get::query(&outputs);
        let next_index_matches = next_counter
            .handle_index()
            .eq(&(counter_i.handle() + (&1_i64).as_handle()));

        let arg = schema_dsl::Arg::query(
            &schema_dsl::TupleT::query(&schema_dsl::TypeList::<PR, _>::query_leaf()),
            &schema_dsl::Assumption::<PR, _>::query_leaf(),
        );
        let arg_counter = schema_dsl::Get::query(&arg);
        let arg_index_matches = arg_counter.handle_index().eq(&counter_i.handle());

        let one_ty = schema_dsl::Type::query_leaf();
        let one_ctx = schema_dsl::Assumption::query_leaf();
        let one_int = schema_dsl::Int::query();
        let one_matches_value = one_int.handle_value().eq(&1_i64);
        let one = schema_dsl::Const::query(&one_int, &one_ty, &one_ctx);
        let increment = schema_dsl::Bop::query(&schema_dsl::Add::query(), &arg_counter, &one);
        let next_matches_increment = next_counter.handle().eq(&increment.handle());

        let end_ty = schema_dsl::Type::query_leaf();
        let end_ctx = schema_dsl::Assumption::query_leaf();
        let end_const = schema_dsl::Int::query();
        let end_expr = schema_dsl::Const::query(&end_const, &end_ty, &end_ctx);
        let pred_matches_bound = pred.handle().eq(&schema_dsl::Bop::query(
            &schema_dsl::LessThan::query(),
            &next_counter,
            &end_expr,
        )
        .handle());

        LoopUnrollPat::new(lhs, inputs, outputs, start_const, end_const)
            .assert(pred_is_first)
            .assert(input_index_matches)
            .assert(input_matches_start)
            .assert(next_index_matches)
            .assert(arg_index_matches)
            .assert(next_matches_increment)
            .assert(one_matches_value)
            .assert(pred_matches_bound)
    }

    pub(crate) fn register_native_rules() -> RuleSetId {
        let ruleset = RuleSetId("loop-unroll");

        PeepholeTx::add_rule("loop_unroll", ruleset, loop_unroll_pat, |ctx, pat| {
            let start_const = ctx.devalue(pat.start_const.value);
            let end_const = ctx.devalue(pat.end_const.value);
            if end_const <= start_const || start_const % 4 != 0 || end_const % 4 != 0 {
                return;
            }

            let inputs = pat.inputs.to_value(&ctx).val;
            let outputs = pat.outputs.to_value(&ctx).val;
            let num_inputs = ctx.lookup_expect("tuple-length", &[inputs]);
            let Some(old_cost_value) = ctx.lookup("LoopNumItersGuess", &[inputs, outputs]) else {
                return;
            };
            let old_cost = ctx._devalue_base::<i64>(old_cost_value);

            let one_iter = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert("SubTuple", &[outputs, ctx._intern_base::<i64, i64>(1), num_inputs]));
            let tmp_ctx = eggplant::wrap::Value::<schema_dsl::Assumption>::new((ctx).insert("LoopUnrollTmpCtx", &[]));

            let subst_once = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert("Subst", &[
                    tmp_ctx.to_value(&ctx).val,
                    one_iter.to_value(&ctx).val,
                    outputs,
                ]));
            let subst_twice = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert("Subst", &[
                    tmp_ctx.to_value(&ctx).val,
                    one_iter.to_value(&ctx).val,
                    subst_once.to_value(&ctx).val,
                ]));
            let unrolled = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert("Subst", &[
                    tmp_ctx.to_value(&ctx).val,
                    one_iter.to_value(&ctx).val,
                    subst_twice.to_value(&ctx).val,
                ]));
            let new_loop = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert("DoWhile", &[inputs, unrolled.to_value(&ctx).val]));
            let actual_ctx = eggplant::wrap::Value::<schema_dsl::Assumption>::new((&ctx).insert("InLoop", &[inputs, unrolled.to_value(&ctx).val]));

            ctx.union(pat.lhs, new_loop);
            ctx.union(tmp_ctx, actual_ctx);
            ctx.insert_func_tbl(
                "LoopNumItersGuess",
                &[
                    inputs,
                    unrolled.to_value(&ctx).val,
                    ctx._intern_base::<i64, i64>(old_cost / 4),
                ],
            );
            ctx.remove("LoopUnrollTmpCtx", &[]);
        });

        ruleset
    }
}

#[cfg(all(test, feature = "eggplant"))]
mod native_tests {
    use crate::ast::*;
    use crate::eggplant_backend::test_lock;
    use crate::schedule;
    use egglog::{ast::Expr as EgglogExpr, EGraph, TermDag};

    fn candidate_expr() -> String {
        dowhile(
            parallel!(int(0)),
            parallel!(
                less_than(add(getat(0), int(1)), int(8)),
                add(getat(0), int(1))
            ),
        )
        .add_arg_type(base(intt()))
        .to_string()
    }

    fn focused_schedule() -> String {
        let helpers = schedule::helpers();
        format!("(run-schedule\n{helpers}\nloop-unroll\n{helpers}\n)")
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
    fn native_feature_path_matches_text_backend_for_loop_unroll_case() {
        let _guard = test_lock::lock();
        let expr = candidate_expr();
        let schedule = focused_schedule();
        let text_extracted =
            eval_and_extract_text_expr(&crate::prologue_egglog_text(), &expr, &schedule);
        let native_extracted = eval_and_extract_native_expr(
            &crate::feature_execution_prologue(true, None),
            &expr,
            &schedule,
            None,
        );
        let native_tmp_ctx_count = native_feature_op_count(
            &crate::feature_execution_prologue(true, None),
            &expr,
            &schedule,
            None,
            "LoopUnrollTmpCtx",
        );
        let ablated_native_tmp_ctx_count = native_feature_op_count(
            &crate::feature_execution_prologue(true, Some("loop-unroll")),
            &expr,
            &schedule,
            Some("loop-unroll"),
            "LoopUnrollTmpCtx",
        );

        assert_eq!(
            native_extracted, text_extracted,
            "native feature path should preserve the extracted representative seen in the text backend",
        );
        assert!(
            native_tmp_ctx_count > 0,
            "native feature path should materialize the private LoopUnrollTmpCtx artifact when loop-unroll fires",
        );
        assert_eq!(
            ablated_native_tmp_ctx_count, 0,
            "ablating the native loop-unroll ruleset should remove the native-only LoopUnrollTmpCtx artifact",
        );
    }
}
