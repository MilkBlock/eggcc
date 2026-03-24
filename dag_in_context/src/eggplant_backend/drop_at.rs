pub(crate) fn fragment() -> String {
    let mut out = String::new();
    out.push_str(GENERATED_MARKER);
    out.push_str(DROP_AT);
    out.push('\n');
    out
}

#[cfg(feature = "eggplant")]
pub(crate) fn native_fragment() -> String {
    let support_only = DROP_AT
        .replacen(DROP_AT_SEED_RULE, "", 1)
        .replacen(DROP_AT_CONST_RULE, "", 1)
        .replacen(DROP_AT_EMPTY_RULE, "", 1)
        .replacen(DROP_AT_ARG_GET_LT_RULE, "", 1)
        .replacen(DROP_AT_ARG_GET_GT_RULE, "", 1)
        .replacen(DROP_AT_TOP_RULE, "", 1)
        .replacen(DROP_AT_BOP_RULE, "", 1)
        .replacen(DROP_AT_UOP_RULE, "", 1)
        .replacen(DROP_AT_GENERIC_GET_RULE, "", 1)
        .replacen(DROP_AT_ALLOC_RULE, "", 1)
        .replacen(DROP_AT_CALL_RULE, "", 1)
        .replacen(DROP_AT_SWITCH_RULE, "", 1)
        .replacen(DROP_AT_IF_RULE, "", 1)
        .replacen(DROP_AT_DOWHILE_RULE, "", 1)
        .replacen(DROP_AT_FUNCTION_RULE, "", 1)
        .replacen(DROP_AT_SINGLE_RULE, "", 1)
        .replacen(DROP_AT_CONCAT_RULE, "", 1);

    assert_ne!(
        support_only, DROP_AT,
        "drop_at::native_fragment() expects the seed / arg-get rules to be present"
    );

    let mut out = String::new();
    out.push_str(GENERATED_MARKER);
    out.push_str(&support_only);
    out.push('\n');
    out
}

const GENERATED_MARKER: &str =
    "; (Generated from eggplant Rust: src/eggplant_backend/drop_at.rs)\n";
const DROP_AT: &str = r#";; Like Subst but for dropping inputs to a region
;; See subst.egg for more implementation documentation

(ruleset drop)
(ruleset apply-drop-unions)
(ruleset cleanup-drop)

;; (DropAt ctx idx in) removes all references to `(Get (Arg ...) idx)` in `in`.
;; It also replaces the leaf contexts with `ctx` and fixes up argument types,
;; as well as updating `(Get (Arg ...) j)` to `(Get (Arg ...) (- j 1))` for j > idx.
(constructor DropAt (Assumption i64 Expr) Expr :unextractable)
(constructor DelayedDropUnion (Expr Expr) Expr :unextractable)

;; Helper that precomputes the arg type that we need
(constructor DropAtInternal (Type Assumption i64 Expr) Expr :unextractable)
(rule ((= lhs (DropAt ctx idx in))
       (HasArgType in (TupleT oldty)))

      ((let newty (TupleT (TypeListRemoveAt oldty idx)))
       (union lhs (DropAtInternal newty ctx idx in)))
      :ruleset drop)

;; Leaves
(rule ((= lhs (DropAtInternal newty newctx idx (Const c oldty oldctx))))
      ((DelayedDropUnion lhs (Const c newty newctx)))
      :ruleset drop)
(rule ((= lhs (DropAtInternal newty newctx idx (Empty oldty oldctx))))
      ((DelayedDropUnion lhs (Empty newty newctx)))
      :ruleset drop)
; get stuck on purpose if `i = idx` or if we find a bare `Arg`
(rule ((= lhs (DropAtInternal newty newctx idx (Get (Arg oldty oldctx) i)))
       (< i idx))
      ((DelayedDropUnion lhs (Get (Arg newty newctx) i)))
      :ruleset drop)
(rule ((= lhs (DropAtInternal newty newctx idx (Get (Arg oldty oldctx) i)))
       (> i idx))
      ((DelayedDropUnion lhs (Get (Arg newty newctx) (- i 1))))
      :ruleset drop)

;; Operators
(rule ((= lhs (DropAtInternal newty newctx idx (Top op c1 c2 c3)))
       (ExprIsResolved (Top op c1 c2 c3)))
      ((DelayedDropUnion lhs (Top op
            (DropAtInternal newty newctx idx c1)
            (DropAtInternal newty newctx idx c2)
            (DropAtInternal newty newctx idx c3))))
      :ruleset drop)

(rule ((= lhs (DropAtInternal newty newctx idx (Bop op c1 c2)))
       (ExprIsResolved (Bop op c1 c2)))
      ((DelayedDropUnion lhs (Bop op
            (DropAtInternal newty newctx idx c1)
            (DropAtInternal newty newctx idx c2))))
      :ruleset drop)

(rule ((= lhs (DropAtInternal newty newctx idx (Uop op c1)))
       (ExprIsResolved (Uop op c1)))
      ((DelayedDropUnion lhs (Uop op
            (DropAtInternal newty newctx idx c1))))
      :ruleset drop)

;; this is okay because we get stuck at `Arg`s
(rule ((= lhs (DropAtInternal newty newctx idx (Get c1 index)))
       (ExprIsResolved (Get c1 index)))
      ((DelayedDropUnion lhs (Get
            (DropAtInternal newty newctx idx c1)
            index)))
      :ruleset drop)

(rule ((= lhs (DropAtInternal newty newctx idx (Alloc id c1 c2 ty)))
       (ExprIsResolved (Alloc id c1 c2 ty)))
      ((DelayedDropUnion lhs (Alloc id
            (DropAtInternal newty newctx idx c1)
            (DropAtInternal newty newctx idx c2)
            ty)))
      :ruleset drop)

(rule ((= lhs (DropAtInternal newty newctx idx (Call name c1)))
       (ExprIsResolved (Call name c1)))
      ((DelayedDropUnion lhs (Call name
            (DropAtInternal newty newctx idx c1))))
      :ruleset drop)

;; Tuple operators
(rule ((= lhs (DropAtInternal newty newctx idx (Single c1)))
       (ExprIsResolved (Single c1)))
      ((DelayedDropUnion lhs (Single
            (DropAtInternal newty newctx idx c1))))
      :ruleset drop)

(rule ((= lhs (DropAtInternal newty newctx idx (Concat c1 c2)))
       (ExprIsResolved (Concat c1 c2)))
      ((DelayedDropUnion lhs (Concat
            (DropAtInternal newty newctx idx c1)
            (DropAtInternal newty newctx idx c2))))
      :ruleset drop)

;; Control flow
(rule ((= lhs (DropAtInternal newty newctx idx (Switch pred inputs c1)))
       (ExprIsResolved (Switch pred inputs c1)))
      ((DelayedDropUnion lhs (Switch
            (DropAtInternal newty newctx idx pred)
            (DropAtInternal newty newctx idx inputs)
            c1)))
      :ruleset drop)

(rule ((= lhs (DropAtInternal newty newctx idx (If pred inputs c1 c2)))
       (ExprIsResolved (If pred inputs c1 c2)))
      ((DelayedDropUnion lhs (If
            (DropAtInternal newty newctx idx pred)
            (DropAtInternal newty newctx idx inputs)
            c1
            c2)))
      :ruleset drop)

(rule ((= lhs (DropAtInternal newty newctx idx (DoWhile in out)))
       (ExprIsResolved (DoWhile in out)))
      ((DelayedDropUnion lhs (DoWhile
            (DropAtInternal newty newctx idx in)
            out)))
      :ruleset drop)

(rewrite (DropAtInternal newty newctx idx (Function name inty outty body))
         (Function name inty outty (DropAtInternal newty newctx idx body))
         :when ((ExprIsResolved body))
         :ruleset drop)



;; ########################### Apply drop unions

(rule ((DelayedDropUnion lhs rhs))
      ((union lhs rhs))
      :ruleset apply-drop-unions)

;; ########################### Cleanup Dropat, DropAtInternal and DelayedDropUnion

(rule ((ExprIsResolved (DropAt newctx idx in)))
      ((subsume (DropAt newctx idx in)))
      :ruleset cleanup-drop)

(rule ((ExprIsResolved (DropAtInternal newty newctx idx in)))
      ((subsume (DropAtInternal newty newctx idx in)))
      :ruleset cleanup-drop)

(rule ((DelayedDropUnion lhs rhs))
      ((subsume (DelayedDropUnion lhs rhs)))
      :ruleset cleanup-drop)"#;

#[cfg(feature = "eggplant")]
const DROP_AT_SEED_RULE: &str = r#"(rule ((= lhs (DropAt ctx idx in))
       (HasArgType in (TupleT oldty)))

      ((let newty (TupleT (TypeListRemoveAt oldty idx)))
       (union lhs (DropAtInternal newty ctx idx in)))
      :ruleset drop)
"#;

#[cfg(feature = "eggplant")]
const DROP_AT_CONST_RULE: &str = r#"(rule ((= lhs (DropAtInternal newty newctx idx (Const c oldty oldctx))))
      ((DelayedDropUnion lhs (Const c newty newctx)))
      :ruleset drop)
"#;

#[cfg(feature = "eggplant")]
const DROP_AT_EMPTY_RULE: &str = r#"(rule ((= lhs (DropAtInternal newty newctx idx (Empty oldty oldctx))))
      ((DelayedDropUnion lhs (Empty newty newctx)))
      :ruleset drop)
"#;

#[cfg(feature = "eggplant")]
const DROP_AT_ARG_GET_LT_RULE: &str = r#"(rule ((= lhs (DropAtInternal newty newctx idx (Get (Arg oldty oldctx) i)))
       (< i idx))
      ((DelayedDropUnion lhs (Get (Arg newty newctx) i)))
      :ruleset drop)
"#;

#[cfg(feature = "eggplant")]
const DROP_AT_ARG_GET_GT_RULE: &str = r#"(rule ((= lhs (DropAtInternal newty newctx idx (Get (Arg oldty oldctx) i)))
       (> i idx))
      ((DelayedDropUnion lhs (Get (Arg newty newctx) (- i 1))))
      :ruleset drop)
"#;

#[cfg(feature = "eggplant")]
const DROP_AT_TOP_RULE: &str = r#"(rule ((= lhs (DropAtInternal newty newctx idx (Top op c1 c2 c3)))
       (ExprIsResolved (Top op c1 c2 c3)))
      ((DelayedDropUnion lhs (Top op
            (DropAtInternal newty newctx idx c1)
            (DropAtInternal newty newctx idx c2)
            (DropAtInternal newty newctx idx c3))))
      :ruleset drop)
"#;

#[cfg(feature = "eggplant")]
const DROP_AT_BOP_RULE: &str = r#"(rule ((= lhs (DropAtInternal newty newctx idx (Bop op c1 c2)))
       (ExprIsResolved (Bop op c1 c2)))
      ((DelayedDropUnion lhs (Bop op
            (DropAtInternal newty newctx idx c1)
            (DropAtInternal newty newctx idx c2))))
      :ruleset drop)
"#;

#[cfg(feature = "eggplant")]
const DROP_AT_UOP_RULE: &str = r#"(rule ((= lhs (DropAtInternal newty newctx idx (Uop op c1)))
       (ExprIsResolved (Uop op c1)))
      ((DelayedDropUnion lhs (Uop op
            (DropAtInternal newty newctx idx c1))))
      :ruleset drop)
"#;

#[cfg(feature = "eggplant")]
const DROP_AT_GENERIC_GET_RULE: &str = r#"(rule ((= lhs (DropAtInternal newty newctx idx (Get c1 index)))
       (ExprIsResolved (Get c1 index)))
      ((DelayedDropUnion lhs (Get
            (DropAtInternal newty newctx idx c1)
            index)))
      :ruleset drop)
"#;

#[cfg(feature = "eggplant")]
const DROP_AT_ALLOC_RULE: &str = r#"(rule ((= lhs (DropAtInternal newty newctx idx (Alloc id c1 c2 ty)))
       (ExprIsResolved (Alloc id c1 c2 ty)))
      ((DelayedDropUnion lhs (Alloc id
            (DropAtInternal newty newctx idx c1)
            (DropAtInternal newty newctx idx c2)
            ty)))
      :ruleset drop)
"#;

#[cfg(feature = "eggplant")]
const DROP_AT_CALL_RULE: &str = r#"(rule ((= lhs (DropAtInternal newty newctx idx (Call name c1)))
       (ExprIsResolved (Call name c1)))
      ((DelayedDropUnion lhs (Call name
            (DropAtInternal newty newctx idx c1))))
      :ruleset drop)
"#;

#[cfg(feature = "eggplant")]
const DROP_AT_SWITCH_RULE: &str = r#"(rule ((= lhs (DropAtInternal newty newctx idx (Switch pred inputs c1)))
       (ExprIsResolved (Switch pred inputs c1)))
      ((DelayedDropUnion lhs (Switch
            (DropAtInternal newty newctx idx pred)
            (DropAtInternal newty newctx idx inputs)
            c1)))
      :ruleset drop)
"#;

#[cfg(feature = "eggplant")]
const DROP_AT_IF_RULE: &str = r#"(rule ((= lhs (DropAtInternal newty newctx idx (If pred inputs c1 c2)))
       (ExprIsResolved (If pred inputs c1 c2)))
      ((DelayedDropUnion lhs (If
            (DropAtInternal newty newctx idx pred)
            (DropAtInternal newty newctx idx inputs)
            c1
            c2)))
      :ruleset drop)
"#;

#[cfg(feature = "eggplant")]
const DROP_AT_DOWHILE_RULE: &str = r#"(rule ((= lhs (DropAtInternal newty newctx idx (DoWhile in out)))
       (ExprIsResolved (DoWhile in out)))
      ((DelayedDropUnion lhs (DoWhile
            (DropAtInternal newty newctx idx in)
            out)))
      :ruleset drop)
"#;

#[cfg(feature = "eggplant")]
const DROP_AT_FUNCTION_RULE: &str = r#"(rewrite (DropAtInternal newty newctx idx (Function name inty outty body))
         (Function name inty outty (DropAtInternal newty newctx idx body))
         :when ((ExprIsResolved body))
         :ruleset drop)
"#;

#[cfg(feature = "eggplant")]
const DROP_AT_SINGLE_RULE: &str = r#"(rule ((= lhs (DropAtInternal newty newctx idx (Single c1)))
       (ExprIsResolved (Single c1)))
      ((DelayedDropUnion lhs (Single
            (DropAtInternal newty newctx idx c1))))
      :ruleset drop)
"#;

#[cfg(feature = "eggplant")]
const DROP_AT_CONCAT_RULE: &str = r#"(rule ((= lhs (DropAtInternal newty newctx idx (Concat c1 c2)))
       (ExprIsResolved (Concat c1 c2)))
      ((DelayedDropUnion lhs (Concat
            (DropAtInternal newty newctx idx c1)
            (DropAtInternal newty newctx idx c2))))
      :ruleset drop)
"#;

#[cfg(feature = "eggplant")]
pub(crate) mod native {
    use super::super::native_rule_helpers::insert_call;
    use super::super::schema_dsl;
    use crate::eggplant_backend::peepholes::native::PeepholeTx;
    use eggplant::prelude::{
        prim_call, prim_fact, BaseVar, IntoHandleTy, PEq, PatRecSgl, RuleRunnerSgl, RuleSetId,
    };

    fn expr_leaf<PR: PatRecSgl>() -> schema_dsl::Expr<PR> {
        schema_dsl::Expr::query_leaf()
    }

    fn type_leaf<PR: PatRecSgl>() -> schema_dsl::Type<PR> {
        schema_dsl::Type::query_leaf()
    }

    fn type_list_leaf<PR: PatRecSgl>() -> schema_dsl::TypeList<PR> {
        schema_dsl::TypeList::query_leaf()
    }

    fn assumption_leaf<PR: PatRecSgl>() -> schema_dsl::Assumption<PR> {
        schema_dsl::Assumption::query_leaf()
    }

    fn constant_leaf<PR: PatRecSgl>() -> schema_dsl::Constant<PR> {
        schema_dsl::Constant::query_leaf()
    }

    fn unary_op_leaf<PR: PatRecSgl>() -> schema_dsl::UnaryOp<PR> {
        schema_dsl::UnaryOp::query_leaf()
    }

    fn ternary_op_leaf<PR: PatRecSgl>() -> schema_dsl::TernaryOp<PR> {
        schema_dsl::TernaryOp::query_leaf()
    }

    fn binary_op_leaf<PR: PatRecSgl>() -> schema_dsl::BinaryOp<PR> {
        schema_dsl::BinaryOp::query_leaf()
    }

    fn base_type_leaf<PR: PatRecSgl>() -> schema_dsl::BaseType<PR> {
        schema_dsl::BaseType::query_leaf()
    }

    fn list_expr_leaf<PR: PatRecSgl>() -> schema_dsl::ListExpr<PR> {
        schema_dsl::ListExpr::query_leaf()
    }

    #[eggplant::pat_vars]
    struct DropAtSeedPat<PR: PatRecSgl> {
        lhs: schema_dsl::Expr,
        ctx: schema_dsl::Assumption,
        idx: i64,
        input: schema_dsl::Expr,
        old_tylist: schema_dsl::TypeList,
    }

    #[eggplant::pat_vars]
    struct DropAtArgGetPat<PR: PatRecSgl> {
        lhs: schema_dsl::Expr,
        new_ty: schema_dsl::Type,
        new_ctx: schema_dsl::Assumption,
        idx: i64,
        source_index: i64,
    }

    #[eggplant::pat_vars]
    struct DropAtConstPat<PR: PatRecSgl> {
        lhs: schema_dsl::Expr,
        new_ty: schema_dsl::Type,
        new_ctx: schema_dsl::Assumption,
        constant: schema_dsl::Constant,
    }

    #[eggplant::pat_vars]
    struct DropAtEmptyPat<PR: PatRecSgl> {
        lhs: schema_dsl::Expr,
        new_ty: schema_dsl::Type,
        new_ctx: schema_dsl::Assumption,
    }

    #[eggplant::pat_vars]
    struct DropAtTopPat<PR: PatRecSgl> {
        lhs: schema_dsl::Expr,
        new_ty: schema_dsl::Type,
        new_ctx: schema_dsl::Assumption,
        idx: i64,
        op: schema_dsl::TernaryOp,
        first: schema_dsl::Expr,
        second: schema_dsl::Expr,
        third: schema_dsl::Expr,
    }

    #[eggplant::pat_vars]
    struct DropAtBopPat<PR: PatRecSgl> {
        lhs: schema_dsl::Expr,
        new_ty: schema_dsl::Type,
        new_ctx: schema_dsl::Assumption,
        idx: i64,
        op: schema_dsl::BinaryOp,
        lhs_inner: schema_dsl::Expr,
        rhs_inner: schema_dsl::Expr,
    }

    #[eggplant::pat_vars]
    struct DropAtUopPat<PR: PatRecSgl> {
        lhs: schema_dsl::Expr,
        new_ty: schema_dsl::Type,
        new_ctx: schema_dsl::Assumption,
        idx: i64,
        op: schema_dsl::UnaryOp,
        inner: schema_dsl::Expr,
    }

    #[eggplant::pat_vars]
    struct DropAtGetPat<PR: PatRecSgl> {
        lhs: schema_dsl::Expr,
        new_ty: schema_dsl::Type,
        new_ctx: schema_dsl::Assumption,
        idx: i64,
        inner: schema_dsl::Expr,
        index: i64,
    }

    #[eggplant::pat_vars]
    struct DropAtAllocPat<PR: PatRecSgl> {
        lhs: schema_dsl::Expr,
        new_ty: schema_dsl::Type,
        new_ctx: schema_dsl::Assumption,
        idx: i64,
        alloc: schema_dsl::Alloc,
        amount: schema_dsl::Expr,
        state_edge: schema_dsl::Expr,
        pointer_ty: schema_dsl::BaseType,
    }

    #[eggplant::pat_vars]
    struct DropAtCallPat<PR: PatRecSgl> {
        lhs: schema_dsl::Expr,
        new_ty: schema_dsl::Type,
        new_ctx: schema_dsl::Assumption,
        idx: i64,
        call: schema_dsl::Call,
        arg: schema_dsl::Expr,
    }

    #[eggplant::pat_vars]
    struct DropAtSwitchPat<PR: PatRecSgl> {
        lhs: schema_dsl::Expr,
        new_ty: schema_dsl::Type,
        new_ctx: schema_dsl::Assumption,
        idx: i64,
        pred: schema_dsl::Expr,
        inputs: schema_dsl::Expr,
        branches: schema_dsl::ListExpr,
    }

    #[eggplant::pat_vars]
    struct DropAtIfPat<PR: PatRecSgl> {
        lhs: schema_dsl::Expr,
        new_ty: schema_dsl::Type,
        new_ctx: schema_dsl::Assumption,
        idx: i64,
        pred: schema_dsl::Expr,
        inputs: schema_dsl::Expr,
        then_branch: schema_dsl::Expr,
        else_branch: schema_dsl::Expr,
    }

    #[eggplant::pat_vars]
    struct DropAtDoWhilePat<PR: PatRecSgl> {
        lhs: schema_dsl::Expr,
        new_ty: schema_dsl::Type,
        new_ctx: schema_dsl::Assumption,
        idx: i64,
        inputs: schema_dsl::Expr,
        body: schema_dsl::Expr,
    }

    #[eggplant::pat_vars]
    struct DropAtFunctionPat<PR: PatRecSgl> {
        lhs: schema_dsl::Expr,
        new_ty: schema_dsl::Type,
        new_ctx: schema_dsl::Assumption,
        idx: i64,
        function: schema_dsl::Function,
        input_ty: schema_dsl::Type,
        output_ty: schema_dsl::Type,
        body: schema_dsl::Expr,
    }

    #[eggplant::pat_vars]
    struct DropAtSinglePat<PR: PatRecSgl> {
        lhs: schema_dsl::Expr,
        new_ty: schema_dsl::Type,
        new_ctx: schema_dsl::Assumption,
        idx: i64,
        inner: schema_dsl::Expr,
    }

    #[eggplant::pat_vars]
    struct DropAtConcatPat<PR: PatRecSgl> {
        lhs: schema_dsl::Expr,
        new_ty: schema_dsl::Type,
        new_ctx: schema_dsl::Assumption,
        idx: i64,
        lhs_inner: schema_dsl::Expr,
        rhs_inner: schema_dsl::Expr,
    }

    fn drop_at_seed_pat<PR: PatRecSgl>() -> DropAtSeedPat<PR> {
        let lhs = expr_leaf::<PR>();
        let ctx = assumption_leaf::<PR>();
        let idx = BaseVar::<i64, PR>::query_named("drop_at_idx");
        let input = expr_leaf::<PR>();
        let old_tylist = type_list_leaf::<PR>();
        let input_ty = schema_dsl::TupleT::query(&old_tylist);
        let lhs_is_drop_at = lhs.handle().eq(&prim_call::<schema_dsl::Expr>(
            "DropAt",
            vec![
                ctx.handle().into_handle_ty(),
                idx.handle().into_handle_ty(),
                input.handle().into_handle_ty(),
            ],
        ));
        let has_arg_type = prim_fact(
            "HasArgType",
            vec![
                input.handle().into_handle_ty(),
                input_ty.handle().into_handle_ty(),
            ],
        );

        DropAtSeedPat::new(lhs, ctx, idx, input, old_tylist)
            .assert(lhs_is_drop_at)
            .assert(has_arg_type)
    }

    fn drop_at_arg_get_pat<PR: PatRecSgl>() -> DropAtArgGetPat<PR> {
        let lhs = expr_leaf::<PR>();
        let new_ty = type_leaf::<PR>();
        let new_ctx = assumption_leaf::<PR>();
        let idx = BaseVar::<i64, PR>::query_named("drop_at_internal_idx");
        let source_index = BaseVar::<i64, PR>::query_named("drop_at_source_index");
        let old_ty = type_leaf::<PR>();
        let old_ctx = assumption_leaf::<PR>();
        let arg = schema_dsl::Arg::query(&old_ty, &old_ctx);
        let matched = schema_dsl::Get::query(&arg);
        let matched_index = matched.handle_index().eq(&source_index.handle());
        let lhs_is_drop_internal = lhs.handle().eq(&prim_call::<schema_dsl::Expr>(
            "DropAtInternal",
            vec![
                new_ty.handle().into_handle_ty(),
                new_ctx.handle().into_handle_ty(),
                idx.handle().into_handle_ty(),
                matched.handle().into_handle_ty(),
            ],
        ));

        DropAtArgGetPat::new(lhs, new_ty, new_ctx, idx, source_index)
            .assert(matched_index)
            .assert(lhs_is_drop_internal)
    }

    fn drop_at_const_pat<PR: PatRecSgl>() -> DropAtConstPat<PR> {
        let lhs = expr_leaf::<PR>();
        let new_ty = type_leaf::<PR>();
        let new_ctx = assumption_leaf::<PR>();
        let idx = BaseVar::<i64, PR>::query_named("drop_at_const_idx");
        let constant = constant_leaf::<PR>();
        let old_ty = type_leaf::<PR>();
        let old_ctx = assumption_leaf::<PR>();
        let matched = schema_dsl::Const::query(&constant, &old_ty, &old_ctx);
        let lhs_is_drop_internal = lhs.handle().eq(&prim_call::<schema_dsl::Expr>(
            "DropAtInternal",
            vec![
                new_ty.handle().into_handle_ty(),
                new_ctx.handle().into_handle_ty(),
                idx.handle().into_handle_ty(),
                matched.handle().into_handle_ty(),
            ],
        ));

        DropAtConstPat::new(lhs, new_ty, new_ctx, constant).assert(lhs_is_drop_internal)
    }

    fn drop_at_empty_pat<PR: PatRecSgl>() -> DropAtEmptyPat<PR> {
        let lhs = expr_leaf::<PR>();
        let new_ty = type_leaf::<PR>();
        let new_ctx = assumption_leaf::<PR>();
        let idx = BaseVar::<i64, PR>::query_named("drop_at_empty_idx");
        let old_ty = type_leaf::<PR>();
        let old_ctx = assumption_leaf::<PR>();
        let matched = schema_dsl::Empty::query(&old_ty, &old_ctx);
        let lhs_is_drop_internal = lhs.handle().eq(&prim_call::<schema_dsl::Expr>(
            "DropAtInternal",
            vec![
                new_ty.handle().into_handle_ty(),
                new_ctx.handle().into_handle_ty(),
                idx.handle().into_handle_ty(),
                matched.handle().into_handle_ty(),
            ],
        ));

        DropAtEmptyPat::new(lhs, new_ty, new_ctx).assert(lhs_is_drop_internal)
    }

    fn drop_at_top_pat<PR: PatRecSgl>() -> DropAtTopPat<PR> {
        let lhs = expr_leaf::<PR>();
        let new_ty = type_leaf::<PR>();
        let new_ctx = assumption_leaf::<PR>();
        let idx = BaseVar::<i64, PR>::query_named("drop_at_top_idx");
        let op = ternary_op_leaf::<PR>();
        let first = expr_leaf::<PR>();
        let second = expr_leaf::<PR>();
        let third = expr_leaf::<PR>();
        let matched = schema_dsl::Top::query(&op, &first, &second, &third);
        let lhs_is_drop_internal = lhs.handle().eq(&prim_call::<schema_dsl::Expr>(
            "DropAtInternal",
            vec![
                new_ty.handle().into_handle_ty(),
                new_ctx.handle().into_handle_ty(),
                idx.handle().into_handle_ty(),
                matched.handle().into_handle_ty(),
            ],
        ));
        let expr_is_resolved = prim_fact("ExprIsResolved", vec![matched.handle().into_handle_ty()]);

        DropAtTopPat::new(lhs, new_ty, new_ctx, idx, op, first, second, third)
            .assert(lhs_is_drop_internal)
            .assert(expr_is_resolved)
    }

    fn drop_at_bop_pat<PR: PatRecSgl>() -> DropAtBopPat<PR> {
        let lhs = expr_leaf::<PR>();
        let new_ty = type_leaf::<PR>();
        let new_ctx = assumption_leaf::<PR>();
        let idx = BaseVar::<i64, PR>::query_named("drop_at_bop_idx");
        let op = binary_op_leaf::<PR>();
        let lhs_inner = expr_leaf::<PR>();
        let rhs_inner = expr_leaf::<PR>();
        let matched = schema_dsl::Bop::query(&op, &lhs_inner, &rhs_inner);
        let lhs_is_drop_internal = lhs.handle().eq(&prim_call::<schema_dsl::Expr>(
            "DropAtInternal",
            vec![
                new_ty.handle().into_handle_ty(),
                new_ctx.handle().into_handle_ty(),
                idx.handle().into_handle_ty(),
                matched.handle().into_handle_ty(),
            ],
        ));
        let expr_is_resolved = prim_fact("ExprIsResolved", vec![matched.handle().into_handle_ty()]);

        DropAtBopPat::new(lhs, new_ty, new_ctx, idx, op, lhs_inner, rhs_inner)
            .assert(lhs_is_drop_internal)
            .assert(expr_is_resolved)
    }

    fn drop_at_uop_pat<PR: PatRecSgl>() -> DropAtUopPat<PR> {
        let lhs = expr_leaf::<PR>();
        let new_ty = type_leaf::<PR>();
        let new_ctx = assumption_leaf::<PR>();
        let idx = BaseVar::<i64, PR>::query_named("drop_at_uop_idx");
        let op = unary_op_leaf::<PR>();
        let inner = expr_leaf::<PR>();
        let matched = schema_dsl::Uop::query(&op, &inner);
        let lhs_is_drop_internal = lhs.handle().eq(&prim_call::<schema_dsl::Expr>(
            "DropAtInternal",
            vec![
                new_ty.handle().into_handle_ty(),
                new_ctx.handle().into_handle_ty(),
                idx.handle().into_handle_ty(),
                matched.handle().into_handle_ty(),
            ],
        ));
        let expr_is_resolved = prim_fact("ExprIsResolved", vec![matched.handle().into_handle_ty()]);

        DropAtUopPat::new(lhs, new_ty, new_ctx, idx, op, inner)
            .assert(lhs_is_drop_internal)
            .assert(expr_is_resolved)
    }

    fn drop_at_get_pat<PR: PatRecSgl>() -> DropAtGetPat<PR> {
        let lhs = expr_leaf::<PR>();
        let new_ty = type_leaf::<PR>();
        let new_ctx = assumption_leaf::<PR>();
        let idx = BaseVar::<i64, PR>::query_named("drop_at_get_idx");
        let inner = expr_leaf::<PR>();
        let index = BaseVar::<i64, PR>::query_named("drop_at_get_index");
        let matched = schema_dsl::Get::query(&inner);
        let matched_index = matched.handle_index().eq(&index.handle());
        let lhs_is_drop_internal = lhs.handle().eq(&prim_call::<schema_dsl::Expr>(
            "DropAtInternal",
            vec![
                new_ty.handle().into_handle_ty(),
                new_ctx.handle().into_handle_ty(),
                idx.handle().into_handle_ty(),
                matched.handle().into_handle_ty(),
            ],
        ));
        let expr_is_resolved = prim_fact("ExprIsResolved", vec![matched.handle().into_handle_ty()]);

        DropAtGetPat::new(lhs, new_ty, new_ctx, idx, inner, index)
            .assert(matched_index)
            .assert(lhs_is_drop_internal)
            .assert(expr_is_resolved)
    }

    fn drop_at_alloc_pat<PR: PatRecSgl>() -> DropAtAllocPat<PR> {
        let lhs = expr_leaf::<PR>();
        let new_ty = type_leaf::<PR>();
        let new_ctx = assumption_leaf::<PR>();
        let idx = BaseVar::<i64, PR>::query_named("drop_at_alloc_idx");
        let amount = expr_leaf::<PR>();
        let state_edge = expr_leaf::<PR>();
        let pointer_ty = base_type_leaf::<PR>();
        let alloc = schema_dsl::Alloc::query(&amount, &state_edge, &pointer_ty);
        let lhs_is_drop_internal = lhs.handle().eq(&prim_call::<schema_dsl::Expr>(
            "DropAtInternal",
            vec![
                new_ty.handle().into_handle_ty(),
                new_ctx.handle().into_handle_ty(),
                idx.handle().into_handle_ty(),
                alloc.handle().into_handle_ty(),
            ],
        ));
        let expr_is_resolved = prim_fact("ExprIsResolved", vec![alloc.handle().into_handle_ty()]);

        DropAtAllocPat::new(
            lhs, new_ty, new_ctx, idx, alloc, amount, state_edge, pointer_ty,
        )
        .assert(lhs_is_drop_internal)
        .assert(expr_is_resolved)
    }

    fn drop_at_call_pat<PR: PatRecSgl>() -> DropAtCallPat<PR> {
        let lhs = expr_leaf::<PR>();
        let new_ty = type_leaf::<PR>();
        let new_ctx = assumption_leaf::<PR>();
        let idx = BaseVar::<i64, PR>::query_named("drop_at_call_idx");
        let arg = expr_leaf::<PR>();
        let call = schema_dsl::Call::query(&arg);
        let lhs_is_drop_internal = lhs.handle().eq(&prim_call::<schema_dsl::Expr>(
            "DropAtInternal",
            vec![
                new_ty.handle().into_handle_ty(),
                new_ctx.handle().into_handle_ty(),
                idx.handle().into_handle_ty(),
                call.handle().into_handle_ty(),
            ],
        ));
        let expr_is_resolved = prim_fact("ExprIsResolved", vec![call.handle().into_handle_ty()]);

        DropAtCallPat::new(lhs, new_ty, new_ctx, idx, call, arg)
            .assert(lhs_is_drop_internal)
            .assert(expr_is_resolved)
    }

    fn drop_at_switch_pat<PR: PatRecSgl>() -> DropAtSwitchPat<PR> {
        let lhs = expr_leaf::<PR>();
        let new_ty = type_leaf::<PR>();
        let new_ctx = assumption_leaf::<PR>();
        let idx = BaseVar::<i64, PR>::query_named("drop_at_switch_idx");
        let pred = expr_leaf::<PR>();
        let inputs = expr_leaf::<PR>();
        let branches = list_expr_leaf::<PR>();
        let switch = schema_dsl::Switch::query(&pred, &inputs, &branches);
        let lhs_is_drop_internal = lhs.handle().eq(&prim_call::<schema_dsl::Expr>(
            "DropAtInternal",
            vec![
                new_ty.handle().into_handle_ty(),
                new_ctx.handle().into_handle_ty(),
                idx.handle().into_handle_ty(),
                switch.handle().into_handle_ty(),
            ],
        ));
        let expr_is_resolved = prim_fact("ExprIsResolved", vec![switch.handle().into_handle_ty()]);

        DropAtSwitchPat::new(lhs, new_ty, new_ctx, idx, pred, inputs, branches)
            .assert(lhs_is_drop_internal)
            .assert(expr_is_resolved)
    }

    fn drop_at_if_pat<PR: PatRecSgl>() -> DropAtIfPat<PR> {
        let lhs = expr_leaf::<PR>();
        let new_ty = type_leaf::<PR>();
        let new_ctx = assumption_leaf::<PR>();
        let idx = BaseVar::<i64, PR>::query_named("drop_at_if_idx");
        let pred = expr_leaf::<PR>();
        let inputs = expr_leaf::<PR>();
        let then_branch = expr_leaf::<PR>();
        let else_branch = expr_leaf::<PR>();
        let if_expr = schema_dsl::If::query(&pred, &inputs, &then_branch, &else_branch);
        let lhs_is_drop_internal = lhs.handle().eq(&prim_call::<schema_dsl::Expr>(
            "DropAtInternal",
            vec![
                new_ty.handle().into_handle_ty(),
                new_ctx.handle().into_handle_ty(),
                idx.handle().into_handle_ty(),
                if_expr.handle().into_handle_ty(),
            ],
        ));
        let expr_is_resolved = prim_fact("ExprIsResolved", vec![if_expr.handle().into_handle_ty()]);

        DropAtIfPat::new(
            lhs,
            new_ty,
            new_ctx,
            idx,
            pred,
            inputs,
            then_branch,
            else_branch,
        )
        .assert(lhs_is_drop_internal)
        .assert(expr_is_resolved)
    }

    fn drop_at_dowhile_pat<PR: PatRecSgl>() -> DropAtDoWhilePat<PR> {
        let lhs = expr_leaf::<PR>();
        let new_ty = type_leaf::<PR>();
        let new_ctx = assumption_leaf::<PR>();
        let idx = BaseVar::<i64, PR>::query_named("drop_at_dowhile_idx");
        let inputs = expr_leaf::<PR>();
        let body = expr_leaf::<PR>();
        let loop_expr = schema_dsl::DoWhile::query(&inputs, &body);
        let lhs_is_drop_internal = lhs.handle().eq(&prim_call::<schema_dsl::Expr>(
            "DropAtInternal",
            vec![
                new_ty.handle().into_handle_ty(),
                new_ctx.handle().into_handle_ty(),
                idx.handle().into_handle_ty(),
                loop_expr.handle().into_handle_ty(),
            ],
        ));
        let expr_is_resolved =
            prim_fact("ExprIsResolved", vec![loop_expr.handle().into_handle_ty()]);

        DropAtDoWhilePat::new(lhs, new_ty, new_ctx, idx, inputs, body)
            .assert(lhs_is_drop_internal)
            .assert(expr_is_resolved)
    }

    fn drop_at_function_pat<PR: PatRecSgl>() -> DropAtFunctionPat<PR> {
        let lhs = expr_leaf::<PR>();
        let new_ty = type_leaf::<PR>();
        let new_ctx = assumption_leaf::<PR>();
        let idx = BaseVar::<i64, PR>::query_named("drop_at_function_idx");
        let input_ty = type_leaf::<PR>();
        let output_ty = type_leaf::<PR>();
        let body = expr_leaf::<PR>();
        let function = schema_dsl::Function::query(&input_ty, &output_ty, &body);
        let lhs_is_drop_internal = lhs.handle().eq(&prim_call::<schema_dsl::Expr>(
            "DropAtInternal",
            vec![
                new_ty.handle().into_handle_ty(),
                new_ctx.handle().into_handle_ty(),
                idx.handle().into_handle_ty(),
                function.handle().into_handle_ty(),
            ],
        ));
        let expr_is_resolved = prim_fact("ExprIsResolved", vec![body.handle().into_handle_ty()]);

        DropAtFunctionPat::new(
            lhs, new_ty, new_ctx, idx, function, input_ty, output_ty, body,
        )
        .assert(lhs_is_drop_internal)
        .assert(expr_is_resolved)
    }

    fn drop_at_single_pat<PR: PatRecSgl>() -> DropAtSinglePat<PR> {
        let lhs = expr_leaf::<PR>();
        let new_ty = type_leaf::<PR>();
        let new_ctx = assumption_leaf::<PR>();
        let idx = BaseVar::<i64, PR>::query_named("drop_at_single_idx");
        let inner = expr_leaf::<PR>();
        let matched = schema_dsl::Single::query(&inner);
        let lhs_is_drop_internal = lhs.handle().eq(&prim_call::<schema_dsl::Expr>(
            "DropAtInternal",
            vec![
                new_ty.handle().into_handle_ty(),
                new_ctx.handle().into_handle_ty(),
                idx.handle().into_handle_ty(),
                matched.handle().into_handle_ty(),
            ],
        ));
        let expr_is_resolved = prim_fact("ExprIsResolved", vec![matched.handle().into_handle_ty()]);

        DropAtSinglePat::new(lhs, new_ty, new_ctx, idx, inner)
            .assert(lhs_is_drop_internal)
            .assert(expr_is_resolved)
    }

    fn drop_at_concat_pat<PR: PatRecSgl>() -> DropAtConcatPat<PR> {
        let lhs = expr_leaf::<PR>();
        let new_ty = type_leaf::<PR>();
        let new_ctx = assumption_leaf::<PR>();
        let idx = BaseVar::<i64, PR>::query_named("drop_at_concat_idx");
        let lhs_inner = expr_leaf::<PR>();
        let rhs_inner = expr_leaf::<PR>();
        let matched = schema_dsl::Concat::query(&lhs_inner, &rhs_inner);
        let lhs_is_drop_internal = lhs.handle().eq(&prim_call::<schema_dsl::Expr>(
            "DropAtInternal",
            vec![
                new_ty.handle().into_handle_ty(),
                new_ctx.handle().into_handle_ty(),
                idx.handle().into_handle_ty(),
                matched.handle().into_handle_ty(),
            ],
        ));
        let expr_is_resolved = prim_fact("ExprIsResolved", vec![matched.handle().into_handle_ty()]);

        DropAtConcatPat::new(lhs, new_ty, new_ctx, idx, lhs_inner, rhs_inner)
            .assert(lhs_is_drop_internal)
            .assert(expr_is_resolved)
    }

    pub(crate) fn register_native_rules() -> RuleSetId {
        let ruleset = RuleSetId("drop");

        PeepholeTx::add_rule("drop_at_seed", ruleset, drop_at_seed_pat, |ctx, pat| {
            let new_tylist = insert_call::<schema_dsl::TypeList>(
                &ctx.ctx,
                "TypeListRemoveAt",
                &[pat.old_tylist.val, pat.idx.val],
            );
            let new_ty = insert_call::<schema_dsl::Type>(&ctx.ctx, "TupleT", &[new_tylist.0.val]);
            ctx.union(
                pat.lhs,
                insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "DropAtInternal",
                    &[new_ty.0.val, pat.ctx.val, pat.idx.val, pat.input.val],
                ),
            );
        });
        PeepholeTx::add_rule("drop_at_const", ruleset, drop_at_const_pat, |ctx, pat| {
            let rewritten = insert_call::<schema_dsl::Expr>(
                &ctx.ctx,
                "Const",
                &[pat.constant.val, pat.new_ty.val, pat.new_ctx.val],
            );
            let _ = insert_call::<schema_dsl::Expr>(
                &ctx.ctx,
                "DelayedDropUnion",
                &[pat.lhs.val, rewritten.0.val],
            );
        });
        PeepholeTx::add_rule("drop_at_empty", ruleset, drop_at_empty_pat, |ctx, pat| {
            let rewritten = insert_call::<schema_dsl::Expr>(
                &ctx.ctx,
                "Empty",
                &[pat.new_ty.val, pat.new_ctx.val],
            );
            let _ = insert_call::<schema_dsl::Expr>(
                &ctx.ctx,
                "DelayedDropUnion",
                &[pat.lhs.val, rewritten.0.val],
            );
        });
        PeepholeTx::add_rule("drop_at_top", ruleset, drop_at_top_pat, |ctx, pat| {
            let first = insert_call::<schema_dsl::Expr>(
                &ctx.ctx,
                "DropAtInternal",
                &[pat.new_ty.val, pat.new_ctx.val, pat.idx.val, pat.first.val],
            );
            let second = insert_call::<schema_dsl::Expr>(
                &ctx.ctx,
                "DropAtInternal",
                &[pat.new_ty.val, pat.new_ctx.val, pat.idx.val, pat.second.val],
            );
            let third = insert_call::<schema_dsl::Expr>(
                &ctx.ctx,
                "DropAtInternal",
                &[pat.new_ty.val, pat.new_ctx.val, pat.idx.val, pat.third.val],
            );
            let rewritten = insert_call::<schema_dsl::Expr>(
                &ctx.ctx,
                "Top",
                &[pat.op.val, first.0.val, second.0.val, third.0.val],
            );
            let _ = insert_call::<schema_dsl::Expr>(
                &ctx.ctx,
                "DelayedDropUnion",
                &[pat.lhs.val, rewritten.0.val],
            );
        });
        PeepholeTx::add_rule(
            "drop_at_arg_get",
            ruleset,
            drop_at_arg_get_pat,
            |ctx, pat| {
                let idx = ctx.devalue(pat.idx);
                let source_index = ctx.devalue(pat.source_index);
                if source_index == idx {
                    return;
                }

                let new_arg = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "Arg",
                    &[pat.new_ty.val, pat.new_ctx.val],
                );
                let new_index = if source_index < idx {
                    pat.source_index.val
                } else {
                    ctx._intern_base::<i64, i64>(source_index - 1)
                };
                let rewritten =
                    insert_call::<schema_dsl::Expr>(&ctx.ctx, "Get", &[new_arg.0.val, new_index]);

                let _ = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "DelayedDropUnion",
                    &[pat.lhs.val, rewritten.0.val],
                );
            },
        );
        PeepholeTx::add_rule("drop_at_bop", ruleset, drop_at_bop_pat, |ctx, pat| {
            let lhs_inner = insert_call::<schema_dsl::Expr>(
                &ctx.ctx,
                "DropAtInternal",
                &[
                    pat.new_ty.val,
                    pat.new_ctx.val,
                    pat.idx.val,
                    pat.lhs_inner.val,
                ],
            );
            let rhs_inner = insert_call::<schema_dsl::Expr>(
                &ctx.ctx,
                "DropAtInternal",
                &[
                    pat.new_ty.val,
                    pat.new_ctx.val,
                    pat.idx.val,
                    pat.rhs_inner.val,
                ],
            );
            let rewritten = insert_call::<schema_dsl::Expr>(
                &ctx.ctx,
                "Bop",
                &[pat.op.val, lhs_inner.0.val, rhs_inner.0.val],
            );
            let _ = insert_call::<schema_dsl::Expr>(
                &ctx.ctx,
                "DelayedDropUnion",
                &[pat.lhs.val, rewritten.0.val],
            );
        });
        PeepholeTx::add_rule("drop_at_uop", ruleset, drop_at_uop_pat, |ctx, pat| {
            let inner = insert_call::<schema_dsl::Expr>(
                &ctx.ctx,
                "DropAtInternal",
                &[pat.new_ty.val, pat.new_ctx.val, pat.idx.val, pat.inner.val],
            );
            let rewritten =
                insert_call::<schema_dsl::Expr>(&ctx.ctx, "Uop", &[pat.op.val, inner.0.val]);
            let _ = insert_call::<schema_dsl::Expr>(
                &ctx.ctx,
                "DelayedDropUnion",
                &[pat.lhs.val, rewritten.0.val],
            );
        });
        PeepholeTx::add_rule("drop_at_get", ruleset, drop_at_get_pat, |ctx, pat| {
            let inner = insert_call::<schema_dsl::Expr>(
                &ctx.ctx,
                "DropAtInternal",
                &[pat.new_ty.val, pat.new_ctx.val, pat.idx.val, pat.inner.val],
            );
            let rewritten =
                insert_call::<schema_dsl::Expr>(&ctx.ctx, "Get", &[inner.0.val, pat.index.val]);
            let _ = insert_call::<schema_dsl::Expr>(
                &ctx.ctx,
                "DelayedDropUnion",
                &[pat.lhs.val, rewritten.0.val],
            );
        });
        PeepholeTx::add_rule("drop_at_alloc", ruleset, drop_at_alloc_pat, |ctx, pat| {
            let amount = insert_call::<schema_dsl::Expr>(
                &ctx.ctx,
                "DropAtInternal",
                &[pat.new_ty.val, pat.new_ctx.val, pat.idx.val, pat.amount.val],
            );
            let state_edge = insert_call::<schema_dsl::Expr>(
                &ctx.ctx,
                "DropAtInternal",
                &[
                    pat.new_ty.val,
                    pat.new_ctx.val,
                    pat.idx.val,
                    pat.state_edge.val,
                ],
            );
            let rewritten = insert_call::<schema_dsl::Expr>(
                &ctx.ctx,
                "Alloc",
                &[
                    pat.alloc.id.val,
                    amount.0.val,
                    state_edge.0.val,
                    pat.pointer_ty.val,
                ],
            );
            let _ = insert_call::<schema_dsl::Expr>(
                &ctx.ctx,
                "DelayedDropUnion",
                &[pat.lhs.val, rewritten.0.val],
            );
        });
        PeepholeTx::add_rule("drop_at_call", ruleset, drop_at_call_pat, |ctx, pat| {
            let arg = insert_call::<schema_dsl::Expr>(
                &ctx.ctx,
                "DropAtInternal",
                &[pat.new_ty.val, pat.new_ctx.val, pat.idx.val, pat.arg.val],
            );
            let rewritten =
                insert_call::<schema_dsl::Expr>(&ctx.ctx, "Call", &[pat.call.name.val, arg.0.val]);
            let _ = insert_call::<schema_dsl::Expr>(
                &ctx.ctx,
                "DelayedDropUnion",
                &[pat.lhs.val, rewritten.0.val],
            );
        });
        PeepholeTx::add_rule("drop_at_switch", ruleset, drop_at_switch_pat, |ctx, pat| {
            let pred = insert_call::<schema_dsl::Expr>(
                &ctx.ctx,
                "DropAtInternal",
                &[pat.new_ty.val, pat.new_ctx.val, pat.idx.val, pat.pred.val],
            );
            let inputs = insert_call::<schema_dsl::Expr>(
                &ctx.ctx,
                "DropAtInternal",
                &[pat.new_ty.val, pat.new_ctx.val, pat.idx.val, pat.inputs.val],
            );
            let rewritten = insert_call::<schema_dsl::Expr>(
                &ctx.ctx,
                "Switch",
                &[pred.0.val, inputs.0.val, pat.branches.val],
            );
            let _ = insert_call::<schema_dsl::Expr>(
                &ctx.ctx,
                "DelayedDropUnion",
                &[pat.lhs.val, rewritten.0.val],
            );
        });
        PeepholeTx::add_rule("drop_at_if", ruleset, drop_at_if_pat, |ctx, pat| {
            let pred = insert_call::<schema_dsl::Expr>(
                &ctx.ctx,
                "DropAtInternal",
                &[pat.new_ty.val, pat.new_ctx.val, pat.idx.val, pat.pred.val],
            );
            let inputs = insert_call::<schema_dsl::Expr>(
                &ctx.ctx,
                "DropAtInternal",
                &[pat.new_ty.val, pat.new_ctx.val, pat.idx.val, pat.inputs.val],
            );
            let rewritten = insert_call::<schema_dsl::Expr>(
                &ctx.ctx,
                "If",
                &[
                    pred.0.val,
                    inputs.0.val,
                    pat.then_branch.val,
                    pat.else_branch.val,
                ],
            );
            let _ = insert_call::<schema_dsl::Expr>(
                &ctx.ctx,
                "DelayedDropUnion",
                &[pat.lhs.val, rewritten.0.val],
            );
        });
        PeepholeTx::add_rule(
            "drop_at_dowhile",
            ruleset,
            drop_at_dowhile_pat,
            |ctx, pat| {
                let inputs = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "DropAtInternal",
                    &[pat.new_ty.val, pat.new_ctx.val, pat.idx.val, pat.inputs.val],
                );
                let rewritten = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "DoWhile",
                    &[inputs.0.val, pat.body.val],
                );
                let _ = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "DelayedDropUnion",
                    &[pat.lhs.val, rewritten.0.val],
                );
            },
        );
        PeepholeTx::add_rule(
            "drop_at_function",
            ruleset,
            drop_at_function_pat,
            |ctx, pat| {
                let body = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "DropAtInternal",
                    &[pat.new_ty.val, pat.new_ctx.val, pat.idx.val, pat.body.val],
                );
                let rewritten = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "Function",
                    &[
                        pat.function.name.val,
                        pat.input_ty.val,
                        pat.output_ty.val,
                        body.0.val,
                    ],
                );
                ctx.union(pat.lhs, rewritten);
            },
        );
        PeepholeTx::add_rule("drop_at_single", ruleset, drop_at_single_pat, |ctx, pat| {
            let inner = insert_call::<schema_dsl::Expr>(
                &ctx.ctx,
                "DropAtInternal",
                &[pat.new_ty.val, pat.new_ctx.val, pat.idx.val, pat.inner.val],
            );
            let rewritten = insert_call::<schema_dsl::Expr>(&ctx.ctx, "Single", &[inner.0.val]);
            let _ = insert_call::<schema_dsl::Expr>(
                &ctx.ctx,
                "DelayedDropUnion",
                &[pat.lhs.val, rewritten.0.val],
            );
        });
        PeepholeTx::add_rule("drop_at_concat", ruleset, drop_at_concat_pat, |ctx, pat| {
            let lhs_inner = insert_call::<schema_dsl::Expr>(
                &ctx.ctx,
                "DropAtInternal",
                &[
                    pat.new_ty.val,
                    pat.new_ctx.val,
                    pat.idx.val,
                    pat.lhs_inner.val,
                ],
            );
            let rhs_inner = insert_call::<schema_dsl::Expr>(
                &ctx.ctx,
                "DropAtInternal",
                &[
                    pat.new_ty.val,
                    pat.new_ctx.val,
                    pat.idx.val,
                    pat.rhs_inner.val,
                ],
            );
            let rewritten = insert_call::<schema_dsl::Expr>(
                &ctx.ctx,
                "Concat",
                &[lhs_inner.0.val, rhs_inner.0.val],
            );
            let _ = insert_call::<schema_dsl::Expr>(
                &ctx.ctx,
                "DelayedDropUnion",
                &[pat.lhs.val, rewritten.0.val],
            );
        });

        ruleset
    }
}

#[cfg(all(test, feature = "eggplant"))]
mod native_tests {
    use crate::eggplant_backend::test_lock;
    use egglog::EGraph;

    fn candidate_parts() -> (String, String, String) {
        let ctx = "(InFunc \"RLCR\")".to_string();
        let input = "(Get (Arg (TupleT (TCons (IntT) (TCons (BoolT) (TNil)))) (InFunc \"OLD\")) 1)"
            .to_string();
        let expected =
            "(Get (Arg (TupleT (TCons (BoolT) (TNil))) (InFunc \"RLCR\")) 0)".to_string();
        (ctx, input, expected)
    }

    fn structural_candidate_parts() -> (String, String, String) {
        let ctx = "(InFunc \"RLCR\")".to_string();
        let input = "(Concat (Single (Get (Arg (TupleT (TCons (IntT) (TCons (BoolT) (TNil)))) (InFunc \"OLD\")) 1)) (Single (Get (Arg (TupleT (TCons (IntT) (TCons (BoolT) (TNil)))) (InFunc \"OLD\")) 1)))".to_string();
        let expected = "(Concat (Single (Get (Arg (TupleT (TCons (BoolT) (TNil))) (InFunc \"RLCR\")) 0)) (Single (Get (Arg (TupleT (TCons (BoolT) (TNil))) (InFunc \"RLCR\")) 0)))".to_string();
        (ctx, input, expected)
    }

    fn const_leaf_candidate_parts() -> (String, String, String) {
        let ctx = "(InFunc \"RLCR\")".to_string();
        let input =
            "(Const (Int 7) (TupleT (TCons (IntT) (TCons (BoolT) (TNil)))) (InFunc \"OLD\"))"
                .to_string();
        let expected =
            "(Const (Int 7) (TupleT (TCons (BoolT) (TNil))) (InFunc \"RLCR\"))".to_string();
        (ctx, input, expected)
    }

    fn empty_leaf_candidate_parts() -> (String, String, String) {
        let ctx = "(InFunc \"RLCR\")".to_string();
        let input =
            "(Empty (TupleT (TCons (IntT) (TCons (BoolT) (TNil)))) (InFunc \"OLD\"))".to_string();
        let expected = "(Empty (TupleT (TCons (BoolT) (TNil))) (InFunc \"RLCR\"))".to_string();
        (ctx, input, expected)
    }

    fn binary_operator_candidate_parts() -> (String, String, String) {
        let ctx = "(InFunc \"RLCR\")".to_string();
        let input = "(Bop (And) (Get (Arg (TupleT (TCons (IntT) (TCons (BoolT) (TNil)))) (InFunc \"OLD\")) 1) (Get (Single (Uop (Not) (Get (Arg (TupleT (TCons (IntT) (TCons (BoolT) (TNil)))) (InFunc \"OLD\")) 1))) 0))".to_string();
        let expected = "(Bop (And) (Get (Arg (TupleT (TCons (BoolT) (TNil))) (InFunc \"RLCR\")) 0) (Get (Single (Uop (Not) (Get (Arg (TupleT (TCons (BoolT) (TNil))) (InFunc \"RLCR\")) 0))) 0))".to_string();
        (ctx, input, expected)
    }

    fn ternary_operator_candidate_parts() -> (String, String, String) {
        let ctx = "(InFunc \"RLCR\")".to_string();
        let input = "(Top (Select) (Get (Arg (TupleT (TCons (IntT) (TCons (BoolT) (TCons (BoolT) (TNil))))) (InFunc \"OLD\")) 1) (Get (Arg (TupleT (TCons (IntT) (TCons (BoolT) (TCons (BoolT) (TNil))))) (InFunc \"OLD\")) 2) (Get (Single (Get (Arg (TupleT (TCons (IntT) (TCons (BoolT) (TCons (BoolT) (TNil))))) (InFunc \"OLD\")) 1)) 0))".to_string();
        let expected = "(Top (Select) (Get (Arg (TupleT (TCons (BoolT) (TCons (BoolT) (TNil)))) (InFunc \"RLCR\")) 0) (Get (Arg (TupleT (TCons (BoolT) (TCons (BoolT) (TNil)))) (InFunc \"RLCR\")) 1) (Get (Single (Get (Arg (TupleT (TCons (BoolT) (TCons (BoolT) (TNil)))) (InFunc \"RLCR\")) 0)) 0))".to_string();
        (ctx, input, expected)
    }

    fn single_input_operator_candidate_parts() -> (String, String, String) {
        let ctx = "(InFunc \"RLCR\")".to_string();
        let input = "(Single (Get (Single (Uop (Not) (Get (Arg (TupleT (TCons (IntT) (TCons (BoolT) (TNil)))) (InFunc \"OLD\")) 1))) 0))".to_string();
        let expected = "(Single (Get (Single (Uop (Not) (Get (Arg (TupleT (TCons (BoolT) (TNil))) (InFunc \"RLCR\")) 0))) 0))".to_string();
        (ctx, input, expected)
    }

    fn alloc_candidate_parts() -> (String, String, String) {
        let ctx = "(InFunc \"RLCR\")".to_string();
        let input = "(Alloc 7 (Get (Arg (TupleT (TCons (IntT) (TCons (BoolT) (TNil)))) (InFunc \"OLD\")) 1) (Get (Arg (TupleT (TCons (IntT) (TCons (BoolT) (TNil)))) (InFunc \"OLD\")) 1) (PointerT (BoolT)))".to_string();
        let expected = "(Alloc 7 (Get (Arg (TupleT (TCons (BoolT) (TNil))) (InFunc \"RLCR\")) 0) (Get (Arg (TupleT (TCons (BoolT) (TNil))) (InFunc \"RLCR\")) 0) (PointerT (BoolT)))".to_string();
        (ctx, input, expected)
    }

    fn call_candidate_parts() -> (String, String, String) {
        let ctx = "(InFunc \"RLCR\")".to_string();
        let input =
            "(Call \"drop_at_helper\" (Get (Arg (TupleT (TCons (IntT) (TCons (BoolT) (TNil)))) (InFunc \"OLD\")) 1))".to_string();
        let expected =
            "(Call \"drop_at_helper\" (Get (Arg (TupleT (TCons (BoolT) (TNil))) (InFunc \"RLCR\")) 0))".to_string();
        (ctx, input, expected)
    }

    fn switch_candidate_parts() -> (String, String, String) {
        let ctx = "(InFunc \"RLCR\")".to_string();
        let input = "(Switch (Get (Arg (TupleT (TCons (IntT) (TCons (BoolT) (TNil)))) (InFunc \"OLD\")) 1) (Single (Get (Arg (TupleT (TCons (IntT) (TCons (BoolT) (TNil)))) (InFunc \"OLD\")) 1)) (Cons (Const (Int 0) (Base (IntT)) (InSwitch 0 (Get (Arg (TupleT (TCons (IntT) (TCons (BoolT) (TNil)))) (InFunc \"OLD\")) 1) (Single (Get (Arg (TupleT (TCons (IntT) (TCons (BoolT) (TNil)))) (InFunc \"OLD\")) 1)))) (Cons (Const (Int 1) (Base (IntT)) (InSwitch 1 (Get (Arg (TupleT (TCons (IntT) (TCons (BoolT) (TNil)))) (InFunc \"OLD\")) 1) (Single (Get (Arg (TupleT (TCons (IntT) (TCons (BoolT) (TNil)))) (InFunc \"OLD\")) 1)))) (Nil))))".to_string();
        let expected = "(Switch (Get (Arg (TupleT (TCons (BoolT) (TNil))) (InFunc \"RLCR\")) 0) (Single (Get (Arg (TupleT (TCons (BoolT) (TNil))) (InFunc \"RLCR\")) 0)) (Cons (Const (Int 0) (Base (IntT)) (InSwitch 0 (Get (Arg (TupleT (TCons (IntT) (TCons (BoolT) (TNil)))) (InFunc \"OLD\")) 1) (Single (Get (Arg (TupleT (TCons (IntT) (TCons (BoolT) (TNil)))) (InFunc \"OLD\")) 1)))) (Cons (Const (Int 1) (Base (IntT)) (InSwitch 1 (Get (Arg (TupleT (TCons (IntT) (TCons (BoolT) (TNil)))) (InFunc \"OLD\")) 1) (Single (Get (Arg (TupleT (TCons (IntT) (TCons (BoolT) (TNil)))) (InFunc \"OLD\")) 1)))) (Nil))))".to_string();
        (ctx, input, expected)
    }

    fn if_candidate_parts() -> (String, String, String) {
        let ctx = "(InFunc \"RLCR\")".to_string();
        let input = "(If (Get (Arg (TupleT (TCons (IntT) (TCons (BoolT) (TNil)))) (InFunc \"OLD\")) 1) (Single (Get (Arg (TupleT (TCons (IntT) (TCons (BoolT) (TNil)))) (InFunc \"OLD\")) 1)) (Const (Int 1) (Base (IntT)) (InIf true (Get (Arg (TupleT (TCons (IntT) (TCons (BoolT) (TNil)))) (InFunc \"OLD\")) 1) (Single (Get (Arg (TupleT (TCons (IntT) (TCons (BoolT) (TNil)))) (InFunc \"OLD\")) 1)))) (Const (Int 0) (Base (IntT)) (InIf false (Get (Arg (TupleT (TCons (IntT) (TCons (BoolT) (TNil)))) (InFunc \"OLD\")) 1) (Single (Get (Arg (TupleT (TCons (IntT) (TCons (BoolT) (TNil)))) (InFunc \"OLD\")) 1)))))".to_string();
        let expected = "(If (Get (Arg (TupleT (TCons (BoolT) (TNil))) (InFunc \"RLCR\")) 0) (Single (Get (Arg (TupleT (TCons (BoolT) (TNil))) (InFunc \"RLCR\")) 0)) (Const (Int 1) (Base (IntT)) (InIf true (Get (Arg (TupleT (TCons (IntT) (TCons (BoolT) (TNil)))) (InFunc \"OLD\")) 1) (Single (Get (Arg (TupleT (TCons (IntT) (TCons (BoolT) (TNil)))) (InFunc \"OLD\")) 1)))) (Const (Int 0) (Base (IntT)) (InIf false (Get (Arg (TupleT (TCons (IntT) (TCons (BoolT) (TNil)))) (InFunc \"OLD\")) 1) (Single (Get (Arg (TupleT (TCons (IntT) (TCons (BoolT) (TNil)))) (InFunc \"OLD\")) 1)))))".to_string();
        (ctx, input, expected)
    }

    fn dowhile_candidate_parts() -> (String, String, String) {
        let ctx = "(InFunc \"RLCR\")".to_string();
        let input = "(DoWhile (Concat (Single (Const (Int 9) (Base (IntT)) (InFunc \"OLD\"))) (Single (Get (Arg (TupleT (TCons (IntT) (TCons (BoolT) (TNil)))) (InFunc \"OLD\")) 1))) (Concat (Single (Const (Bool false) (Base (BoolT)) (InFunc \"BODY\"))) (Concat (Single (Const (Int 9) (Base (IntT)) (InFunc \"BODY\"))) (Single (Const (Bool true) (Base (BoolT)) (InFunc \"BODY\"))))))".to_string();
        let expected = "(DoWhile (Concat (Single (Const (Int 9) (TupleT (TCons (BoolT) (TNil))) (InFunc \"RLCR\"))) (Single (Get (Arg (TupleT (TCons (BoolT) (TNil))) (InFunc \"RLCR\")) 0))) (Concat (Single (Const (Bool false) (Base (BoolT)) (InFunc \"BODY\"))) (Concat (Single (Const (Int 9) (Base (IntT)) (InFunc \"BODY\"))) (Single (Const (Bool true) (Base (BoolT)) (InFunc \"BODY\"))))))".to_string();
        (ctx, input, expected)
    }

    fn function_candidate_parts() -> (String, String, String) {
        let ctx = "(InFunc \"RLCR\")".to_string();
        let input = "(Concat (Single (Function \"drop_at_helper\" (TupleT (TCons (IntT) (TCons (BoolT) (TNil)))) (Base (BoolT)) (Get (Arg (TupleT (TCons (IntT) (TCons (BoolT) (TNil)))) (InFunc \"OLD\")) 1))) (Single (Get (Arg (TupleT (TCons (IntT) (TCons (BoolT) (TNil)))) (InFunc \"OLD\")) 1)))".to_string();
        let expected = "(Concat (Single (Function \"drop_at_helper\" (TupleT (TCons (IntT) (TCons (BoolT) (TNil)))) (Base (BoolT)) (Get (Arg (TupleT (TCons (BoolT) (TNil))) (InFunc \"RLCR\")) 0))) (Single (Get (Arg (TupleT (TCons (BoolT) (TNil))) (InFunc \"RLCR\")) 0)))".to_string();
        (ctx, input, expected)
    }

    fn drop_at_schedule() -> String {
        let type_schedule = "(saturate (saturate type-helpers) type-analysis)";
        format!(
            "(run-schedule {type_schedule})\n(run-schedule (saturate is-resolved))\n(run-schedule (saturate drop))\n(run-schedule apply-drop-unions)\n(run-schedule cleanup-drop)\n(run-schedule {type_schedule})\n"
        )
    }

    fn ablated_drop_at_schedule() -> String {
        drop_at_schedule().replacen(
            "(run-schedule (saturate drop))\n",
            "(run-schedule (saturate never))\n",
            1,
        )
    }

    fn text_drop_at_holds(prologue: &str, ctx: &str, input: &str, expected: &str, schedule: &str) {
        let program = format!(
            "{prologue}\n(let __rlcr_input {input})\n(let __rlcr_drop (DropAt {ctx} 0 __rlcr_input))\n{schedule}(check (= __rlcr_drop {expected}))\n"
        );
        let mut egraph = EGraph::default();
        egraph.parse_and_run_program(None, &program).unwrap();
    }

    fn native_drop_at_holds(
        prologue: &str,
        ctx: &str,
        input: &str,
        expected: &str,
        schedule: &str,
        ablate: Option<&str>,
    ) -> std::result::Result<(), eggplant::egglog::Error> {
        let initialization =
            format!("(let __rlcr_input {input})\n(let __rlcr_drop (DropAt {ctx} 0 __rlcr_input))");
        crate::with_native_rules_egraph(prologue, &initialization, schedule, ablate, |egraph| {
            egraph.parse_and_run_program(None, &format!("(check (= __rlcr_drop {expected}))"))?;
            Ok(())
        })
    }

    #[test]
    fn native_feature_path_matches_text_backend_for_drop_at_case() {
        let _guard = test_lock::lock();
        let (ctx, input, expected) = candidate_parts();
        let schedule = drop_at_schedule();

        text_drop_at_holds(
            &crate::prologue_egglog_text(),
            &ctx,
            &input,
            &expected,
            &schedule,
        );
        native_drop_at_holds(
            &crate::feature_execution_prologue(true, None),
            &ctx,
            &input,
            &expected,
            &schedule,
            None,
        )
        .unwrap();

        let ablated = native_drop_at_holds(
            &crate::feature_execution_prologue(true, Some("drop-at")),
            &ctx,
            &input,
            &expected,
            &ablated_drop_at_schedule(),
            Some("drop-at"),
        );

        assert!(
            ablated.is_err(),
            "ablating drop-at should make the drop-at witness fail",
        );
    }

    #[test]
    fn native_feature_path_matches_text_backend_for_drop_at_tuple_structural_case() {
        let _guard = test_lock::lock();
        let (ctx, input, expected) = structural_candidate_parts();
        let schedule = drop_at_schedule();

        text_drop_at_holds(
            &crate::prologue_egglog_text(),
            &ctx,
            &input,
            &expected,
            &schedule,
        );
        native_drop_at_holds(
            &crate::feature_execution_prologue(true, None),
            &ctx,
            &input,
            &expected,
            &schedule,
            None,
        )
        .unwrap();

        let ablated = native_drop_at_holds(
            &crate::feature_execution_prologue(true, Some("drop-at")),
            &ctx,
            &input,
            &expected,
            &ablated_drop_at_schedule(),
            Some("drop-at"),
        );

        assert!(
            ablated.is_err(),
            "ablating drop-at should make the tuple structural witness fail",
        );
    }

    #[test]
    fn native_feature_path_matches_text_backend_for_drop_at_leaf_cases() {
        let _guard = test_lock::lock();
        let schedule = drop_at_schedule();

        for (ctx, input, expected) in [const_leaf_candidate_parts(), empty_leaf_candidate_parts()] {
            text_drop_at_holds(
                &crate::prologue_egglog_text(),
                &ctx,
                &input,
                &expected,
                &schedule,
            );
            native_drop_at_holds(
                &crate::feature_execution_prologue(true, None),
                &ctx,
                &input,
                &expected,
                &schedule,
                None,
            )
            .unwrap();

            let ablated = native_drop_at_holds(
                &crate::feature_execution_prologue(true, Some("drop-at")),
                &ctx,
                &input,
                &expected,
                &ablated_drop_at_schedule(),
                Some("drop-at"),
            );

            assert!(
                ablated.is_err(),
                "ablating drop-at should make the leaf witness fail",
            );
        }
    }

    #[test]
    fn native_feature_path_matches_text_backend_for_drop_at_single_input_operator_case() {
        let _guard = test_lock::lock();
        let (ctx, input, expected) = single_input_operator_candidate_parts();
        let schedule = drop_at_schedule();

        text_drop_at_holds(
            &crate::prologue_egglog_text(),
            &ctx,
            &input,
            &expected,
            &schedule,
        );
        native_drop_at_holds(
            &crate::feature_execution_prologue(true, None),
            &ctx,
            &input,
            &expected,
            &schedule,
            None,
        )
        .unwrap();

        let ablated = native_drop_at_holds(
            &crate::feature_execution_prologue(true, Some("drop-at")),
            &ctx,
            &input,
            &expected,
            &ablated_drop_at_schedule(),
            Some("drop-at"),
        );

        assert!(
            ablated.is_err(),
            "ablating drop-at should make the single-input operator witness fail",
        );
    }

    #[test]
    fn native_feature_path_matches_text_backend_for_drop_at_binary_operator_case() {
        let _guard = test_lock::lock();
        let (ctx, input, expected) = binary_operator_candidate_parts();
        let schedule = drop_at_schedule();

        text_drop_at_holds(
            &crate::prologue_egglog_text(),
            &ctx,
            &input,
            &expected,
            &schedule,
        );
        native_drop_at_holds(
            &crate::feature_execution_prologue(true, None),
            &ctx,
            &input,
            &expected,
            &schedule,
            None,
        )
        .unwrap();

        let ablated = native_drop_at_holds(
            &crate::feature_execution_prologue(true, Some("drop-at")),
            &ctx,
            &input,
            &expected,
            &ablated_drop_at_schedule(),
            Some("drop-at"),
        );

        assert!(
            ablated.is_err(),
            "ablating drop-at should make the binary operator witness fail",
        );
    }

    #[test]
    fn native_feature_path_matches_text_backend_for_drop_at_ternary_operator_case() {
        let _guard = test_lock::lock();
        let (ctx, input, expected) = ternary_operator_candidate_parts();
        let schedule = drop_at_schedule();

        text_drop_at_holds(
            &crate::prologue_egglog_text(),
            &ctx,
            &input,
            &expected,
            &schedule,
        );
        native_drop_at_holds(
            &crate::feature_execution_prologue(true, None),
            &ctx,
            &input,
            &expected,
            &schedule,
            None,
        )
        .unwrap();

        let ablated = native_drop_at_holds(
            &crate::feature_execution_prologue(true, Some("drop-at")),
            &ctx,
            &input,
            &expected,
            &ablated_drop_at_schedule(),
            Some("drop-at"),
        );

        assert!(
            ablated.is_err(),
            "ablating drop-at should make the ternary operator witness fail",
        );
    }

    #[test]
    fn native_feature_path_matches_text_backend_for_drop_at_alloc_case() {
        let _guard = test_lock::lock();
        let (ctx, input, expected) = alloc_candidate_parts();
        let schedule = drop_at_schedule();

        text_drop_at_holds(
            &crate::prologue_egglog_text(),
            &ctx,
            &input,
            &expected,
            &schedule,
        );
        native_drop_at_holds(
            &crate::feature_execution_prologue(true, None),
            &ctx,
            &input,
            &expected,
            &schedule,
            None,
        )
        .unwrap();

        let ablated = native_drop_at_holds(
            &crate::feature_execution_prologue(true, Some("drop-at")),
            &ctx,
            &input,
            &expected,
            &ablated_drop_at_schedule(),
            Some("drop-at"),
        );

        assert!(
            ablated.is_err(),
            "ablating drop-at should make the alloc witness fail",
        );
    }

    #[test]
    fn native_feature_path_matches_text_backend_for_drop_at_call_case() {
        let _guard = test_lock::lock();
        let (ctx, input, expected) = call_candidate_parts();
        let schedule = drop_at_schedule();

        text_drop_at_holds(
            &crate::prologue_egglog_text(),
            &ctx,
            &input,
            &expected,
            &schedule,
        );
        native_drop_at_holds(
            &crate::feature_execution_prologue(true, None),
            &ctx,
            &input,
            &expected,
            &schedule,
            None,
        )
        .unwrap();

        let ablated = native_drop_at_holds(
            &crate::feature_execution_prologue(true, Some("drop-at")),
            &ctx,
            &input,
            &expected,
            &ablated_drop_at_schedule(),
            Some("drop-at"),
        );

        assert!(
            ablated.is_err(),
            "ablating drop-at should make the call witness fail",
        );
    }

    #[test]
    fn native_feature_path_matches_text_backend_for_drop_at_switch_case() {
        let _guard = test_lock::lock();
        let (ctx, input, expected) = switch_candidate_parts();
        let schedule = drop_at_schedule();

        text_drop_at_holds(
            &crate::prologue_egglog_text(),
            &ctx,
            &input,
            &expected,
            &schedule,
        );
        native_drop_at_holds(
            &crate::feature_execution_prologue(true, None),
            &ctx,
            &input,
            &expected,
            &schedule,
            None,
        )
        .unwrap();

        let ablated = native_drop_at_holds(
            &crate::feature_execution_prologue(true, Some("drop-at")),
            &ctx,
            &input,
            &expected,
            &ablated_drop_at_schedule(),
            Some("drop-at"),
        );

        assert!(
            ablated.is_err(),
            "ablating drop-at should make the switch witness fail",
        );
    }

    #[test]
    fn native_feature_path_matches_text_backend_for_drop_at_if_case() {
        let _guard = test_lock::lock();
        let (ctx, input, expected) = if_candidate_parts();
        let schedule = drop_at_schedule();

        text_drop_at_holds(
            &crate::prologue_egglog_text(),
            &ctx,
            &input,
            &expected,
            &schedule,
        );
        native_drop_at_holds(
            &crate::feature_execution_prologue(true, None),
            &ctx,
            &input,
            &expected,
            &schedule,
            None,
        )
        .unwrap();

        let ablated = native_drop_at_holds(
            &crate::feature_execution_prologue(true, Some("drop-at")),
            &ctx,
            &input,
            &expected,
            &ablated_drop_at_schedule(),
            Some("drop-at"),
        );

        assert!(
            ablated.is_err(),
            "ablating drop-at should make the if witness fail",
        );
    }

    #[test]
    fn native_feature_path_matches_text_backend_for_drop_at_dowhile_case() {
        let _guard = test_lock::lock();
        let (ctx, input, expected) = dowhile_candidate_parts();
        let schedule = drop_at_schedule();

        text_drop_at_holds(
            &crate::prologue_egglog_text(),
            &ctx,
            &input,
            &expected,
            &schedule,
        );
        native_drop_at_holds(
            &crate::feature_execution_prologue(true, None),
            &ctx,
            &input,
            &expected,
            &schedule,
            None,
        )
        .unwrap();

        let ablated = native_drop_at_holds(
            &crate::feature_execution_prologue(true, Some("drop-at")),
            &ctx,
            &input,
            &expected,
            &ablated_drop_at_schedule(),
            Some("drop-at"),
        );

        assert!(
            ablated.is_err(),
            "ablating drop-at should make the dowhile witness fail",
        );
    }

    #[test]
    fn native_feature_path_matches_text_backend_for_drop_at_function_case() {
        let _guard = test_lock::lock();
        let (ctx, input, expected) = function_candidate_parts();
        let schedule = drop_at_schedule();

        text_drop_at_holds(
            &crate::prologue_egglog_text(),
            &ctx,
            &input,
            &expected,
            &schedule,
        );
        native_drop_at_holds(
            &crate::feature_execution_prologue(true, None),
            &ctx,
            &input,
            &expected,
            &schedule,
            None,
        )
        .unwrap();

        let ablated = native_drop_at_holds(
            &crate::feature_execution_prologue(true, Some("drop-at")),
            &ctx,
            &input,
            &expected,
            &ablated_drop_at_schedule(),
            Some("drop-at"),
        );

        assert!(
            ablated.is_err(),
            "ablating drop-at should make the function witness fail",
        );
    }
}
