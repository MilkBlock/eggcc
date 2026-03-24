pub(crate) fn fragment() -> String {
    let mut out = String::new();
    out.push_str(GENERATED_MARKER);
    out.push_str(REC_TO_LOOP);
    out.push('\n');
    out
}

#[cfg(feature = "eggplant")]
pub(crate) fn native_fragment() -> String {
    let mut out = String::new();
    out.push_str(GENERATED_MARKER);
    out.push_str(REC_TO_LOOP_NATIVE_SUPPORT);
    out.push('\n');
    out
}

const GENERATED_MARKER: &str =
    "; (Generated from eggplant Rust: src/eggplant_backend/rec_to_loop.rs)\n";
const REC_TO_LOOP_NATIVE_SUPPORT: &str = r#";; this ruleset depends on swap_if running twice
;; swap_if un-permutes the outputs of the function and the if so this rule can match
(ruleset rec-to-loop)

;; Stores information about how to use a binary
;; operator to accumulate values
;; (bop start-val base-case-op)
(relation Accum-Bop (BinaryOp i64 BinaryOp))

;; addition is easy, it starts at 0 and adds the result of the recursive call
(Accum-Bop (Add) 0 (Add))

;; subtraction starts at zero, but adds the base case at the end
(Accum-Bop (Sub) 0 (Add))

;; multiplication starts at 1, and multiplies the result of the recursive call
(Accum-Bop (Mul) 1 (Mul))

;; It seems like integers have these properties based on: https://stackoverflow.com/questions/69480173/which-arithmetic-properties-do-twos-complement-integers-have"#;
const REC_TO_LOOP: &str = r#";; this ruleset depends on swap_if running twice
;; swap_if un-permutes the outputs of the function and the if so this rule can match
(ruleset rec-to-loop)



;; this rule finds a recursive functions
;; with a base case and a tail-recursive call
;; transforms them into a loop
;; transforming functions that look like this:
;; function name(inputs) {
;;    let start = always_runs(inputs);
;;    if (pred) {
;;       ret name(rec_case(start));
;;    } else {
;;       ret base_case(start);
;;    }
;; }
;; into:
;; function name(inputs) {
;;    let start = always_runs(inputs);
;;    if (start[0]) {
;;      do {
;;         start = always_runs(rec_case(start));
;;      } while (start[0]);
;;    }
;;    ret base_case(start);
;; }
;; for example, printBinary sums the results of recursive calls
(rule
  ((Function name in out body)
   (= body (If pred always-runs (Call name rec_case) base-case))
   (HasType always-runs start-ty)
   (HasType body func-ty))
  ((let loop-inputs (Arg start-ty (InIf true pred always-runs)))
   (let loop-outputs
     (Concat
         (Single (Subst (TmpCtx) rec_case pred))
         (Subst (TmpCtx) rec_case always-runs)))
   (union (TmpCtx) (InLoop loop-inputs loop-outputs))
   (delete (TmpCtx))

   (let loop
     (DoWhile loop-inputs loop-outputs))
    
    
  ;; initial start value
   (let outer-if
     (If pred always-runs
         loop
         (Arg start-ty (InIf false pred always-runs))))
   (union (Function name in out body)
     (Function name in out 
      (Subst (InFunc name) outer-if base-case))))
  :ruleset rec-to-loop)


;; Stores information about how to use a binary
;; operator to accumulate values
;; (bop start-val base-case-op)
(relation Accum-Bop (BinaryOp i64 BinaryOp))

;; addition is easy, it starts at 0 and adds the result of the recursive call
(Accum-Bop (Add) 0 (Add))

;; subtraction starts at zero, but adds the base case at the end
(Accum-Bop (Sub) 0 (Add))

;; multiplication starts at 1, and multiplies the result of the recursive call
(Accum-Bop (Mul) 1 (Mul))

;; It seems like integers have these properties based on: https://stackoverflow.com/questions/69480173/which-arithmetic-properties-do-twos-complement-integers-have


;; same as above rule, but with an accumulator
;; function name(inputs) {
;;    let start = always_runs(inputs);
;;    if (pred) {
;;       ret name(rec_case(start)) + f(start);
;;    } else {
;;       ret base_case(start);
;;    }
;; }
;; into:
;; function name(inputs) {
;;    let start = always_runs(inputs);
;;    let acc = 0;
;;    if (start[0]) {
;;      do {
;;         start = always_runs(rec_case(start));
;;         acc = acc + extra(start);
;;      } while (start[0]);
;;    }
;;    ret base_case(start) + acc;
;; }
(rule
  ((Function name in out body)
   (= body (If pred always-runs then-case base-case))
   (= call (Call name rec-case))
   (= then-case
      (Concat (Single (Bop acc-op (Get call 0) extra))
              (Single (Get call 1))))
   (Accum-Bop acc-op initial-int base-case-op)
   (HasType always-runs start-ty)
   (= always-runs-len (tuple-length always-runs))
   (= start-ty (TupleT start-ty-list))
   (HasType body func-ty)
   (ContextOf body body-ctx))
  ((let loop-ty
     (TupleT (TLConcat start-ty-list (TCons (IntT) (TNil)))))
   ;; recursive case in the loop
   (let new-rec-case
    (Subst (TmpCtx)
           (SubTuple (Arg loop-ty (TmpCtx)) 0 always-runs-len) rec-case))
   ;; extra computation in the loop
   (let new-extra
    (Subst (TmpCtx)
           (SubTuple (Arg loop-ty (TmpCtx)) 0 always-runs-len) extra))
   ;; acc starts at 0
   (let loop-inputs
     (Concat (Arg start-ty (InIf true pred always-runs)) (Single (Const (Int initial-int) start-ty (InIf true pred always-runs)))))
   (let loop-outputs
     (Concat
         (Single (Subst (TmpCtx) new-rec-case pred))
         (Concat
           (Subst (TmpCtx) new-rec-case always-runs)
           ;; add extra to acc
           (Single (Bop acc-op (Get (Arg loop-ty (TmpCtx)) always-runs-len) new-extra)))))
   ;; loop starts at zero, adds extra each iteration
   (let loop
     (DoWhile loop-inputs loop-outputs))
   ;; union tmpctx
   (union (TmpCtx) (InLoop loop-inputs loop-outputs))
   (delete (TmpCtx))
  
   (let outer-if
     (If pred always-runs
         loop
         (Concat
           (Arg start-ty (InIf false pred always-runs))
           ;; otherwise acc is 0
           (Single (Const (Int 0) start-ty (InIf false pred always-runs))))))
   ;; base case over latest start value
   (let new-base-case
     (Subst body-ctx (SubTuple outer-if 0 always-runs-len) base-case))
   ;; add base case to acc
   (let res
     (Concat
      (Single (Bop base-case-op (Get new-base-case 0) (Get outer-if always-runs-len)))
      (Single (Get new-base-case 1))))
   (union (Function name in out body)
          (Function name in out res)))
  :ruleset rec-to-loop)"#;

#[cfg(feature = "eggplant")]
pub(crate) mod native {
    use super::super::native_rule_helpers::insert_call;
    use super::super::schema_dsl;
    use crate::eggplant_backend::peepholes::native::PeepholeTx;
    use eggplant::prelude::{
        prim_fact, AsHandle, Insertable, IntoHandleTy, PEq, PatRecSgl, RuleRunnerSgl, RuleSetId,
    };
    use eggplant::wrap::{PRRuleCtx, PatVars};

    #[eggplant::pat_vars]
    struct RecToLoopPat<PR: PatRecSgl> {
        function: schema_dsl::Function,
        input_ty: schema_dsl::Type,
        output_ty: schema_dsl::Type,
        pred: schema_dsl::Expr,
        always_runs: schema_dsl::Expr,
        rec_case: schema_dsl::Expr,
        base_case: schema_dsl::Expr,
        start_ty: schema_dsl::Type,
        body_ty: schema_dsl::Type,
    }

    fn rec_to_loop_pat<PR: PatRecSgl>() -> RecToLoopPat<PR> {
        let input_ty = schema_dsl::Type::query_leaf();
        let output_ty = schema_dsl::Type::query_leaf();
        let pred = schema_dsl::Expr::query_leaf();
        let always_runs = schema_dsl::Expr::query_leaf();
        let rec_case = schema_dsl::Expr::query_leaf();
        let base_case = schema_dsl::Expr::query_leaf();
        let recursive_call = schema_dsl::Call::query(&rec_case);
        let body = schema_dsl::If::query(&pred, &always_runs, &recursive_call, &base_case);
        let function = schema_dsl::Function::query(&input_ty, &output_ty, &body);
        let start_ty = schema_dsl::Type::query_leaf();
        let body_ty = schema_dsl::Type::query_leaf();

        let same_name = function.handle_name().eq(&recursive_call.handle_name());
        let always_runs_has_type = prim_fact(
            "HasType",
            vec![
                always_runs.handle().into_handle_ty(),
                start_ty.handle().into_handle_ty(),
            ],
        );
        let body_has_type = prim_fact(
            "HasType",
            vec![
                body.handle().into_handle_ty(),
                body_ty.handle().into_handle_ty(),
            ],
        );

        RecToLoopPat::new(
            function,
            input_ty,
            output_ty,
            pred,
            always_runs,
            rec_case,
            base_case,
            start_ty,
            body_ty,
        )
        .assert(same_name)
        .assert(always_runs_has_type)
        .assert(body_has_type)
    }

    #[derive(Clone, Copy)]
    enum AccumRuleKind {
        Add,
        Sub,
        Mul,
    }

    impl AccumRuleKind {
        fn rule_name(self) -> &'static str {
            match self {
                Self::Add => "rec_to_loop_accum_add",
                Self::Sub => "rec_to_loop_accum_sub",
                Self::Mul => "rec_to_loop_accum_mul",
            }
        }

        fn acc_op_name(self) -> &'static str {
            match self {
                Self::Add => "Add",
                Self::Sub => "Sub",
                Self::Mul => "Mul",
            }
        }

        fn initial_int(self) -> i64 {
            match self {
                Self::Add | Self::Sub => 0,
                Self::Mul => 1,
            }
        }

        fn base_case_op_name(self) -> &'static str {
            match self {
                Self::Add | Self::Sub => "Add",
                Self::Mul => "Mul",
            }
        }
    }

    #[eggplant::pat_vars]
    struct RecToLoopAccumPat<PR: PatRecSgl> {
        function: schema_dsl::Function,
        input_ty: schema_dsl::Type,
        output_ty: schema_dsl::Type,
        pred: schema_dsl::Expr,
        always_runs: schema_dsl::Expr,
        rec_case: schema_dsl::Expr,
        extra: schema_dsl::Expr,
        base_case: schema_dsl::Expr,
        start_ty: schema_dsl::TupleT,
        start_ty_list: schema_dsl::TypeList,
        body_ty: schema_dsl::Type,
        body_ctx: schema_dsl::Assumption,
    }

    fn rec_to_loop_accum_pat<PR: PatRecSgl>(kind: AccumRuleKind) -> RecToLoopAccumPat<PR> {
        let input_ty = schema_dsl::Type::query_leaf();
        let output_ty = schema_dsl::Type::query_leaf();
        let pred = schema_dsl::Expr::query_leaf();
        let always_runs = schema_dsl::Expr::query_leaf();
        let rec_case = schema_dsl::Expr::query_leaf();
        let extra = schema_dsl::Expr::query_leaf();
        let base_case = schema_dsl::Expr::query_leaf();
        let recursive_call = schema_dsl::Call::query(&rec_case);
        let call0 = schema_dsl::Get::query(&recursive_call);
        let call1 = schema_dsl::Get::query(&recursive_call);
        let then_value = match kind {
            AccumRuleKind::Add => schema_dsl::Bop::query(&schema_dsl::Add::query(), &call0, &extra),
            AccumRuleKind::Sub => schema_dsl::Bop::query(&schema_dsl::Sub::query(), &call0, &extra),
            AccumRuleKind::Mul => schema_dsl::Bop::query(&schema_dsl::Mul::query(), &call0, &extra),
        };
        let then_case = schema_dsl::Concat::query(
            &schema_dsl::Single::query(&then_value),
            &schema_dsl::Single::query(&call1),
        );
        let body = schema_dsl::If::query(&pred, &always_runs, &then_case, &base_case);
        let function = schema_dsl::Function::query(&input_ty, &output_ty, &body);
        let start_ty_list = schema_dsl::TypeList::query_leaf();
        let start_ty = schema_dsl::TupleT::query(&start_ty_list);
        let body_ty = schema_dsl::Type::query_leaf();
        let body_ctx = schema_dsl::Assumption::query_leaf();

        let call0_is_first = call0.handle_index().eq(&(&0_i64).as_handle());
        let call1_is_second = call1.handle_index().eq(&(&1_i64).as_handle());
        let same_name = function.handle_name().eq(&recursive_call.handle_name());
        let always_runs_has_type = prim_fact(
            "HasType",
            vec![
                always_runs.handle().into_handle_ty(),
                start_ty.handle().into_handle_ty(),
            ],
        );
        let body_has_type = prim_fact(
            "HasType",
            vec![
                body.handle().into_handle_ty(),
                body_ty.handle().into_handle_ty(),
            ],
        );
        let body_context = prim_fact(
            "ContextOf",
            vec![
                body.handle().into_handle_ty(),
                body_ctx.handle().into_handle_ty(),
            ],
        );

        RecToLoopAccumPat::new(
            function,
            input_ty,
            output_ty,
            pred,
            always_runs,
            rec_case,
            extra,
            base_case,
            start_ty,
            start_ty_list,
            body_ty,
            body_ctx,
        )
        .assert(call0_is_first)
        .assert(call1_is_second)
        .assert(same_name)
        .assert(always_runs_has_type)
        .assert(body_has_type)
        .assert(body_context)
    }

    fn apply_rec_to_loop_accum<PR: PatRecSgl<MetaTy = ()>>(
        ctx: &PRRuleCtx<'_, '_, '_, '_, PR>,
        pat: &<RecToLoopAccumPat<PR> as PatVars<PR>>::Valued,
        kind: AccumRuleKind,
    ) {
        let zero = ctx._intern_base::<i64, i64>(0);
        let one = ctx._intern_base::<i64, i64>(1);
        let always_runs_len =
            ctx.lookup_expect("tuple-length", &[pat.always_runs.to_value(&ctx.ctx).val]);
        let acc_op = insert_call::<schema_dsl::BinaryOp>(&ctx.ctx, kind.acc_op_name(), &[]);
        let base_case_op =
            insert_call::<schema_dsl::BinaryOp>(&ctx.ctx, kind.base_case_op_name(), &[]);

        let int_ty = insert_call::<schema_dsl::BaseType>(&ctx.ctx, "IntT", &[]);
        let tnil = insert_call::<schema_dsl::TypeList>(&ctx.ctx, "TNil", &[]);
        let appended_tail =
            insert_call::<schema_dsl::TypeList>(&ctx.ctx, "TCons", &[int_ty.0.val, tnil.0.val]);
        let loop_ty_list = insert_call::<schema_dsl::TypeList>(
            &ctx.ctx,
            "TLConcat",
            &[
                pat.start_ty_list.to_value(&ctx.ctx).val,
                appended_tail.to_value(&ctx.ctx).val,
            ],
        );
        let loop_ty = insert_call::<schema_dsl::Type>(&ctx.ctx, "TupleT", &[loop_ty_list.0.val]);
        let tmp_ctx = insert_call::<schema_dsl::Assumption>(&ctx.ctx, "TmpCtx", &[]);
        let tmp_arg = insert_call::<schema_dsl::Expr>(
            &ctx.ctx,
            "Arg",
            &[
                loop_ty.to_value(&ctx.ctx).val,
                tmp_ctx.to_value(&ctx.ctx).val,
            ],
        );
        let loop_subtuple = insert_call::<schema_dsl::Expr>(
            &ctx.ctx,
            "SubTuple",
            &[tmp_arg.to_value(&ctx.ctx).val, zero, always_runs_len],
        );
        let new_rec_case = insert_call::<schema_dsl::Expr>(
            &ctx.ctx,
            "Subst",
            &[
                tmp_ctx.to_value(&ctx.ctx).val,
                loop_subtuple.to_value(&ctx.ctx).val,
                pat.rec_case.to_value(&ctx.ctx).val,
            ],
        );
        let new_extra = insert_call::<schema_dsl::Expr>(
            &ctx.ctx,
            "Subst",
            &[
                tmp_ctx.to_value(&ctx.ctx).val,
                loop_subtuple.to_value(&ctx.ctx).val,
                pat.extra.to_value(&ctx.ctx).val,
            ],
        );

        let then_ctx = insert_call::<schema_dsl::Assumption>(
            &ctx.ctx,
            "InIf",
            &[
                true.to_value(&ctx.ctx).val,
                pat.pred.to_value(&ctx.ctx).val,
                pat.always_runs.to_value(&ctx.ctx).val,
            ],
        );
        let loop_arg = insert_call::<schema_dsl::Expr>(
            &ctx.ctx,
            "Arg",
            &[
                pat.start_ty.to_value(&ctx.ctx).val,
                then_ctx.to_value(&ctx.ctx).val,
            ],
        );
        let initial_int = ctx._intern_base::<i64, i64>(kind.initial_int());
        let initial_const = insert_call::<schema_dsl::Expr>(
            &ctx.ctx,
            "Const",
            &[
                insert_call::<schema_dsl::Constant>(&ctx.ctx, "Int", &[initial_int])
                    .to_value(&ctx.ctx)
                    .val,
                pat.start_ty.to_value(&ctx.ctx).val,
                then_ctx.to_value(&ctx.ctx).val,
            ],
        );
        let loop_inputs = insert_call::<schema_dsl::Expr>(
            &ctx.ctx,
            "Concat",
            &[
                loop_arg.to_value(&ctx.ctx).val,
                insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "Single",
                    &[initial_const.to_value(&ctx.ctx).val],
                )
                .to_value(&ctx.ctx)
                .val,
            ],
        );

        let loop_pred = insert_call::<schema_dsl::Expr>(
            &ctx.ctx,
            "Subst",
            &[
                tmp_ctx.to_value(&ctx.ctx).val,
                new_rec_case.to_value(&ctx.ctx).val,
                pat.pred.to_value(&ctx.ctx).val,
            ],
        );
        let loop_body = insert_call::<schema_dsl::Expr>(
            &ctx.ctx,
            "Subst",
            &[
                tmp_ctx.to_value(&ctx.ctx).val,
                new_rec_case.to_value(&ctx.ctx).val,
                pat.always_runs.to_value(&ctx.ctx).val,
            ],
        );
        let new_acc = insert_call::<schema_dsl::Expr>(
            &ctx.ctx,
            "Bop",
            &[
                acc_op.to_value(&ctx.ctx).val,
                insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "Get",
                    &[tmp_arg.to_value(&ctx.ctx).val, always_runs_len],
                )
                .to_value(&ctx.ctx)
                .val,
                new_extra.to_value(&ctx.ctx).val,
            ],
        );
        let loop_outputs = insert_call::<schema_dsl::Expr>(
            &ctx.ctx,
            "Concat",
            &[
                insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "Single",
                    &[loop_pred.to_value(&ctx.ctx).val],
                )
                .to_value(&ctx.ctx)
                .val,
                insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "Concat",
                    &[
                        loop_body.to_value(&ctx.ctx).val,
                        insert_call::<schema_dsl::Expr>(
                            &ctx.ctx,
                            "Single",
                            &[new_acc.to_value(&ctx.ctx).val],
                        )
                        .to_value(&ctx.ctx)
                        .val,
                    ],
                )
                .to_value(&ctx.ctx)
                .val,
            ],
        );
        let loop_ctx = insert_call::<schema_dsl::Assumption>(
            &ctx.ctx,
            "InLoop",
            &[
                loop_inputs.to_value(&ctx.ctx).val,
                loop_outputs.to_value(&ctx.ctx).val,
            ],
        );
        ctx.union(tmp_ctx, loop_ctx);

        let loop_expr = insert_call::<schema_dsl::Expr>(
            &ctx.ctx,
            "DoWhile",
            &[
                loop_inputs.to_value(&ctx.ctx).val,
                loop_outputs.to_value(&ctx.ctx).val,
            ],
        );
        let else_ctx = insert_call::<schema_dsl::Assumption>(
            &ctx.ctx,
            "InIf",
            &[
                false.to_value(&ctx.ctx).val,
                pat.pred.to_value(&ctx.ctx).val,
                pat.always_runs.to_value(&ctx.ctx).val,
            ],
        );
        let else_arg = insert_call::<schema_dsl::Expr>(
            &ctx.ctx,
            "Arg",
            &[
                pat.start_ty.to_value(&ctx.ctx).val,
                else_ctx.to_value(&ctx.ctx).val,
            ],
        );
        let zero_const = insert_call::<schema_dsl::Expr>(
            &ctx.ctx,
            "Const",
            &[
                insert_call::<schema_dsl::Constant>(&ctx.ctx, "Int", &[zero])
                    .to_value(&ctx.ctx)
                    .val,
                pat.start_ty.to_value(&ctx.ctx).val,
                else_ctx.to_value(&ctx.ctx).val,
            ],
        );
        let outer_else = insert_call::<schema_dsl::Expr>(
            &ctx.ctx,
            "Concat",
            &[
                else_arg.to_value(&ctx.ctx).val,
                insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "Single",
                    &[zero_const.to_value(&ctx.ctx).val],
                )
                .to_value(&ctx.ctx)
                .val,
            ],
        );
        let outer_if = insert_call::<schema_dsl::Expr>(
            &ctx.ctx,
            "If",
            &[
                pat.pred.to_value(&ctx.ctx).val,
                pat.always_runs.to_value(&ctx.ctx).val,
                loop_expr.to_value(&ctx.ctx).val,
                outer_else.to_value(&ctx.ctx).val,
            ],
        );
        let new_base_case = insert_call::<schema_dsl::Expr>(
            &ctx.ctx,
            "Subst",
            &[
                pat.body_ctx.to_value(&ctx.ctx).val,
                insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "SubTuple",
                    &[outer_if.to_value(&ctx.ctx).val, zero, always_runs_len],
                )
                .to_value(&ctx.ctx)
                .val,
                pat.base_case.to_value(&ctx.ctx).val,
            ],
        );

        let result = insert_call::<schema_dsl::Expr>(
            &ctx.ctx,
            "Concat",
            &[
                insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "Single",
                    &[insert_call::<schema_dsl::Expr>(
                        &ctx.ctx,
                        "Bop",
                        &[
                            base_case_op.to_value(&ctx.ctx).val,
                            insert_call::<schema_dsl::Expr>(
                                &ctx.ctx,
                                "Get",
                                &[new_base_case.to_value(&ctx.ctx).val, zero],
                            )
                            .to_value(&ctx.ctx)
                            .val,
                            insert_call::<schema_dsl::Expr>(
                                &ctx.ctx,
                                "Get",
                                &[outer_if.to_value(&ctx.ctx).val, always_runs_len],
                            )
                            .to_value(&ctx.ctx)
                            .val,
                        ],
                    )
                    .to_value(&ctx.ctx)
                    .val],
                )
                .to_value(&ctx.ctx)
                .val,
                insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "Single",
                    &[insert_call::<schema_dsl::Expr>(
                        &ctx.ctx,
                        "Get",
                        &[new_base_case.to_value(&ctx.ctx).val, one],
                    )
                    .to_value(&ctx.ctx)
                    .val],
                )
                .to_value(&ctx.ctx)
                .val,
            ],
        );
        let new_function = insert_call::<schema_dsl::Expr>(
            &ctx.ctx,
            "Function",
            &[
                pat.function.name.val,
                pat.input_ty.to_value(&ctx.ctx).val,
                pat.output_ty.to_value(&ctx.ctx).val,
                result.to_value(&ctx.ctx).val,
            ],
        );

        ctx.union(pat.function, new_function);
        ctx.remove("TmpCtx", &[]);
    }

    pub(crate) fn register_native_rules() -> RuleSetId {
        let ruleset = RuleSetId("rec-to-loop");

        PeepholeTx::add_rule(
            "rec_to_loop_tail_recursive",
            ruleset,
            rec_to_loop_pat,
            |ctx, pat| {
                let tmp_ctx = insert_call::<schema_dsl::Assumption>(&ctx.ctx, "TmpCtx", &[]);
                let loop_input_ctx = insert_call::<schema_dsl::Assumption>(
                    &ctx.ctx,
                    "InIf",
                    &[
                        true.to_value(&ctx.ctx).val,
                        pat.pred.to_value(&ctx.ctx).val,
                        pat.always_runs.to_value(&ctx.ctx).val,
                    ],
                );
                let loop_inputs = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "Arg",
                    &[
                        pat.start_ty.to_value(&ctx.ctx).val,
                        loop_input_ctx.to_value(&ctx.ctx).val,
                    ],
                );
                let loop_pred = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "Subst",
                    &[
                        tmp_ctx.to_value(&ctx.ctx).val,
                        pat.rec_case.to_value(&ctx.ctx).val,
                        pat.pred.to_value(&ctx.ctx).val,
                    ],
                );
                let loop_body = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "Subst",
                    &[
                        tmp_ctx.to_value(&ctx.ctx).val,
                        pat.rec_case.to_value(&ctx.ctx).val,
                        pat.always_runs.to_value(&ctx.ctx).val,
                    ],
                );
                let loop_outputs = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "Concat",
                    &[
                        insert_call::<schema_dsl::Expr>(
                            &ctx.ctx,
                            "Single",
                            &[loop_pred.to_value(&ctx.ctx).val],
                        )
                        .to_value(&ctx.ctx)
                        .val,
                        loop_body.to_value(&ctx.ctx).val,
                    ],
                );
                let in_loop = insert_call::<schema_dsl::Assumption>(
                    &ctx.ctx,
                    "InLoop",
                    &[
                        loop_inputs.to_value(&ctx.ctx).val,
                        loop_outputs.to_value(&ctx.ctx).val,
                    ],
                );
                ctx.union(tmp_ctx, in_loop);

                let loop_expr = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "DoWhile",
                    &[
                        loop_inputs.to_value(&ctx.ctx).val,
                        loop_outputs.to_value(&ctx.ctx).val,
                    ],
                );
                let else_ctx = insert_call::<schema_dsl::Assumption>(
                    &ctx.ctx,
                    "InIf",
                    &[
                        false.to_value(&ctx.ctx).val,
                        pat.pred.to_value(&ctx.ctx).val,
                        pat.always_runs.to_value(&ctx.ctx).val,
                    ],
                );
                let else_arg = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "Arg",
                    &[
                        pat.start_ty.to_value(&ctx.ctx).val,
                        else_ctx.to_value(&ctx.ctx).val,
                    ],
                );
                let outer_if = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "If",
                    &[
                        pat.pred.to_value(&ctx.ctx).val,
                        pat.always_runs.to_value(&ctx.ctx).val,
                        loop_expr.to_value(&ctx.ctx).val,
                        else_arg.to_value(&ctx.ctx).val,
                    ],
                );
                let in_func = insert_call::<schema_dsl::Assumption>(
                    &ctx.ctx,
                    "InFunc",
                    &[pat.function.name.val],
                );
                let new_body = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "Subst",
                    &[
                        in_func.to_value(&ctx.ctx).val,
                        outer_if.to_value(&ctx.ctx).val,
                        pat.base_case.to_value(&ctx.ctx).val,
                    ],
                );
                let new_function = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "Function",
                    &[
                        pat.function.name.val,
                        pat.input_ty.to_value(&ctx.ctx).val,
                        pat.output_ty.to_value(&ctx.ctx).val,
                        new_body.to_value(&ctx.ctx).val,
                    ],
                );

                ctx.union(pat.function, new_function);
                ctx.remove("TmpCtx", &[]);
            },
        );

        for kind in [AccumRuleKind::Add, AccumRuleKind::Sub, AccumRuleKind::Mul] {
            PeepholeTx::add_rule(
                kind.rule_name(),
                ruleset,
                move || rec_to_loop_accum_pat(kind),
                move |ctx, pat| apply_rec_to_loop_accum(ctx, pat, kind),
            );
        }

        ruleset
    }
}
