pub(crate) fn fragment() -> String {
    let mut out = String::new();
    out.push_str(GENERATED_MARKER);
    out.push_str(TERM_SUBST);
    out.push('\n');
    out
}

#[cfg(feature = "eggplant")]
pub(crate) fn native_fragment() -> String {
    let mut out = String::new();
    out.push_str(GENERATED_MARKER);
    out.push_str(TERM_SUBST_DECLS);
    out.push('\n');
    out
}

const GENERATED_MARKER: &str =
    "; (Generated from eggplant Rust: src/eggplant_backend/term_subst.rs)\n";
const TERM_SUBST: &str = r#"(ruleset term-subst)


; Instantiate the term as an Expr in the provided context
; where references to (Arg) in the term are replaced by Expr
(constructor TermSubst (Assumption Expr Term) Expr :unextractable)

; type rule to get the arg type of a substitution
(rule (
        (= lhs (TermSubst ctx e1 term))
        (HasArgType e1 ty)       
      )
      ((HasArgType lhs ty))
      :ruleset term-subst)


; leaf node
; replace the context
(rule ((= lhs (TermSubst ctx e (TermArg))))
      ((union lhs (AddContext ctx e)))
      :ruleset term-subst)

(rule ((= lhs (TermSubst ctx e (TermConst c)))
       (HasArgType e newty))
      ((union lhs (Const c newty ctx)))
      :ruleset term-subst)

(rule ((= lhs (TermSubst ctx e (TermEmpty)))
       (HasArgType e newty))
      ((union lhs (Empty newty ctx)))
      :ruleset term-subst)

; Operators
(rule ((= lhs (TermSubst ctx e (TermTop op t1 t2 t3))))
      ((union lhs (Top op (TermSubst ctx e t1)
                          (TermSubst ctx e t2)
                          (TermSubst ctx e t3))))
      :ruleset term-subst)

(rule ((= lhs (TermSubst ctx e (TermBop op t1 t2))))
      ((union lhs (Bop op (TermSubst ctx e t1)
                          (TermSubst ctx e t2))))
      :ruleset term-subst)

(rule ((= lhs (TermSubst ctx e (TermUop op t1))))
      ((union lhs (Uop op (TermSubst ctx e t1))))
      :ruleset term-subst)

(rule ((= lhs (TermSubst ctx e (TermGet t idx))))
      ((union lhs (Get (TermSubst ctx e t) idx)))
      :ruleset term-subst)

(rule ((= lhs (TermSubst ctx e (TermAlloc id t1 t2 ty))))
      ((union lhs (Alloc id (TermSubst ctx e t1)
                            (TermSubst ctx e t2)
                            ty)))
      :ruleset term-subst)

(rule ((= lhs (TermSubst ctx e (TermCall name t))))
      ((union lhs (Call name (TermSubst ctx e t))))
      :ruleset term-subst)

(rule ((= lhs (TermSubst ctx e (TermSingle t))))
      ((union lhs (Single (TermSubst ctx e t))))
      :ruleset term-subst)

(rule ((= lhs (TermSubst ctx e (TermConcat t1 t2))))
      ((union lhs (Concat (TermSubst ctx e t1)
                          (TermSubst ctx e t2))))
      :ruleset term-subst)

; Control Flow
; TODO"#;

#[cfg(feature = "eggplant")]
const TERM_SUBST_DECLS: &str = r#"(ruleset term-subst)


; Instantiate the term as an Expr in the provided context
; where references to (Arg) in the term are replaced by Expr
(constructor TermSubst (Assumption Expr Term) Expr :unextractable)
"#;

#[cfg(feature = "eggplant")]
pub(crate) mod native {
    use super::super::schema_dsl::{self, ExprRuleCtx, HasArgTypePRRuleCtx};
    use crate::eggplant_backend::peepholes::native::PeepholeTx;
    use eggplant::prelude::{
        prim_call, BaseVar, IntoHandleTy, PEq, PatRecSgl, RuleRunnerSgl, RuleSetId,
    };

    fn expr_leaf<PR: PatRecSgl>() -> schema_dsl::Expr<PR> {
        schema_dsl::Expr::query_leaf()
    }

    fn term_leaf<PR: PatRecSgl>() -> schema_dsl::Term<PR> {
        schema_dsl::Term::query_leaf()
    }

    fn assumption_leaf<PR: PatRecSgl>() -> schema_dsl::Assumption<PR> {
        schema_dsl::Assumption::query_leaf()
    }

    fn type_leaf<PR: PatRecSgl>() -> schema_dsl::Type<PR> {
        schema_dsl::Type::query_leaf()
    }

    fn constant_leaf<PR: PatRecSgl>() -> schema_dsl::Constant<PR> {
        schema_dsl::Constant::query_leaf()
    }

    fn ternary_op_leaf<PR: PatRecSgl>() -> schema_dsl::TernaryOp<PR> {
        schema_dsl::TernaryOp::query_leaf()
    }

    fn binary_op_leaf<PR: PatRecSgl>() -> schema_dsl::BinaryOp<PR> {
        schema_dsl::BinaryOp::query_leaf()
    }

    fn unary_op_leaf<PR: PatRecSgl>() -> schema_dsl::UnaryOp<PR> {
        schema_dsl::UnaryOp::query_leaf()
    }

    fn base_type_leaf<PR: PatRecSgl>() -> schema_dsl::BaseType<PR> {
        schema_dsl::BaseType::query_leaf()
    }

    fn insert_term_subst(
        ctx: &eggplant::wrap::RuleCtx,
        assumption: eggplant::egglog::Value,
        expr: eggplant::egglog::Value,
        term: eggplant::egglog::Value,
    ) -> eggplant::wrap::Value<schema_dsl::Expr> {
        eggplant::wrap::Value::<schema_dsl::Expr>::new((ctx).insert("TermSubst", &[assumption, expr, term]))
    }

    #[eggplant::pat_vars]
    struct TermSubstTypePat<PR: PatRecSgl> {
        lhs: schema_dsl::Expr,
        ty: schema_dsl::Type,
        has_arg_type: schema_dsl::HasArgType,
    }

    #[eggplant::pat_vars]
    struct TermSubstArgPat<PR: PatRecSgl> {
        lhs: schema_dsl::Expr,
        ctx: schema_dsl::Assumption,
        expr: schema_dsl::Expr,
    }

    #[eggplant::pat_vars]
    struct TermSubstConstPat<PR: PatRecSgl> {
        lhs: schema_dsl::Expr,
        ctx: schema_dsl::Assumption,
        expr: schema_dsl::Expr,
        constant: schema_dsl::Constant,
        ty: schema_dsl::Type,
        has_arg_type: schema_dsl::HasArgType,
    }

    #[eggplant::pat_vars]
    struct TermSubstEmptyPat<PR: PatRecSgl> {
        lhs: schema_dsl::Expr,
        ctx: schema_dsl::Assumption,
        expr: schema_dsl::Expr,
        ty: schema_dsl::Type,
        has_arg_type: schema_dsl::HasArgType,
    }

    #[eggplant::pat_vars]
    struct TermSubstTopPat<PR: PatRecSgl> {
        lhs: schema_dsl::Expr,
        ctx: schema_dsl::Assumption,
        expr: schema_dsl::Expr,
        op: schema_dsl::TernaryOp,
        a: schema_dsl::Term,
        b: schema_dsl::Term,
        c: schema_dsl::Term,
    }

    #[eggplant::pat_vars]
    struct TermSubstBopPat<PR: PatRecSgl> {
        lhs: schema_dsl::Expr,
        ctx: schema_dsl::Assumption,
        expr: schema_dsl::Expr,
        op: schema_dsl::BinaryOp,
        lhs_term: schema_dsl::Term,
        rhs_term: schema_dsl::Term,
    }

    #[eggplant::pat_vars]
    struct TermSubstUopPat<PR: PatRecSgl> {
        lhs: schema_dsl::Expr,
        ctx: schema_dsl::Assumption,
        expr: schema_dsl::Expr,
        op: schema_dsl::UnaryOp,
        term: schema_dsl::Term,
    }

    #[eggplant::pat_vars]
    struct TermSubstGetPat<PR: PatRecSgl> {
        lhs: schema_dsl::Expr,
        ctx: schema_dsl::Assumption,
        expr: schema_dsl::Expr,
        term: schema_dsl::Term,
        index: i64,
    }

    #[eggplant::pat_vars]
    struct TermSubstAllocPat<PR: PatRecSgl> {
        lhs: schema_dsl::Expr,
        ctx: schema_dsl::Assumption,
        expr: schema_dsl::Expr,
        id: i64,
        amount: schema_dsl::Term,
        state: schema_dsl::Term,
        ty: schema_dsl::BaseType,
    }

    #[eggplant::pat_vars]
    struct TermSubstCallPat<PR: PatRecSgl> {
        lhs: schema_dsl::Expr,
        ctx: schema_dsl::Assumption,
        expr: schema_dsl::Expr,
        name: String,
        term: schema_dsl::Term,
    }

    #[eggplant::pat_vars]
    struct TermSubstSinglePat<PR: PatRecSgl> {
        lhs: schema_dsl::Expr,
        ctx: schema_dsl::Assumption,
        expr: schema_dsl::Expr,
        term: schema_dsl::Term,
    }

    #[eggplant::pat_vars]
    struct TermSubstConcatPat<PR: PatRecSgl> {
        lhs: schema_dsl::Expr,
        ctx: schema_dsl::Assumption,
        expr: schema_dsl::Expr,
        lhs_term: schema_dsl::Term,
        rhs_term: schema_dsl::Term,
    }

    fn term_subst_type_pat<PR: PatRecSgl>() -> TermSubstTypePat<PR> {
        let lhs = expr_leaf::<PR>();
        let ctx = assumption_leaf::<PR>();
        let expr = expr_leaf::<PR>();
        let term = term_leaf::<PR>();
        let ty = type_leaf::<PR>();
        let lhs_is_term_subst = lhs.handle().eq(&prim_call::<schema_dsl::Expr>(
            "TermSubst",
            vec![
                ctx.handle().into_handle_ty(),
                expr.handle().into_handle_ty(),
                term.handle().into_handle_ty(),
            ],
        ));
        let has_arg_type = schema_dsl::HasArgType::query();
        let same_expr = has_arg_type.expr.handle().eq(&expr.handle());
        let same_ty = has_arg_type.ty.handle().eq(&ty.handle());

        TermSubstTypePat::new(lhs, ty, has_arg_type)
            .assert(lhs_is_term_subst)
            .assert(same_expr)
            .assert(same_ty)
    }

    fn term_subst_arg_pat<PR: PatRecSgl>() -> TermSubstArgPat<PR> {
        let lhs = expr_leaf::<PR>();
        let ctx = assumption_leaf::<PR>();
        let expr = expr_leaf::<PR>();
        let lhs_is_term_subst = lhs.handle().eq(&prim_call::<schema_dsl::Expr>(
            "TermSubst",
            vec![
                ctx.handle().into_handle_ty(),
                expr.handle().into_handle_ty(),
                prim_call::<schema_dsl::Term>("TermArg", vec![]).into_handle_ty(),
            ],
        ));

        TermSubstArgPat::new(lhs, ctx, expr).assert(lhs_is_term_subst)
    }

    fn term_subst_const_pat<PR: PatRecSgl>() -> TermSubstConstPat<PR> {
        let lhs = expr_leaf::<PR>();
        let ctx = assumption_leaf::<PR>();
        let expr = expr_leaf::<PR>();
        let constant = constant_leaf::<PR>();
        let ty = type_leaf::<PR>();
        let term = schema_dsl::TermConst::query(&constant);
        let lhs_is_term_subst = lhs.handle().eq(&prim_call::<schema_dsl::Expr>(
            "TermSubst",
            vec![
                ctx.handle().into_handle_ty(),
                expr.handle().into_handle_ty(),
                term.handle().into_handle_ty(),
            ],
        ));
        let has_arg_type = schema_dsl::HasArgType::query();
        let same_expr = has_arg_type.expr.handle().eq(&expr.handle());
        let same_ty = has_arg_type.ty.handle().eq(&ty.handle());

        TermSubstConstPat::new(lhs, ctx, expr, constant, ty, has_arg_type)
            .assert(lhs_is_term_subst)
            .assert(same_expr)
            .assert(same_ty)
    }

    fn term_subst_empty_pat<PR: PatRecSgl>() -> TermSubstEmptyPat<PR> {
        let lhs = expr_leaf::<PR>();
        let ctx = assumption_leaf::<PR>();
        let expr = expr_leaf::<PR>();
        let ty = type_leaf::<PR>();
        let lhs_is_term_subst = lhs.handle().eq(&prim_call::<schema_dsl::Expr>(
            "TermSubst",
            vec![
                ctx.handle().into_handle_ty(),
                expr.handle().into_handle_ty(),
                prim_call::<schema_dsl::Term>("TermEmpty", vec![]).into_handle_ty(),
            ],
        ));
        let has_arg_type = schema_dsl::HasArgType::query();
        let same_expr = has_arg_type.expr.handle().eq(&expr.handle());
        let same_ty = has_arg_type.ty.handle().eq(&ty.handle());

        TermSubstEmptyPat::new(lhs, ctx, expr, ty, has_arg_type)
            .assert(lhs_is_term_subst)
            .assert(same_expr)
            .assert(same_ty)
    }

    fn term_subst_top_pat<PR: PatRecSgl>() -> TermSubstTopPat<PR> {
        let lhs = expr_leaf::<PR>();
        let ctx = assumption_leaf::<PR>();
        let expr = expr_leaf::<PR>();
        let op = ternary_op_leaf::<PR>();
        let a = term_leaf::<PR>();
        let b = term_leaf::<PR>();
        let c = term_leaf::<PR>();
        let term = schema_dsl::TermTop::query(&op, &a, &b, &c);
        let lhs_is_term_subst = lhs.handle().eq(&prim_call::<schema_dsl::Expr>(
            "TermSubst",
            vec![
                ctx.handle().into_handle_ty(),
                expr.handle().into_handle_ty(),
                term.handle().into_handle_ty(),
            ],
        ));

        TermSubstTopPat::new(lhs, ctx, expr, op, a, b, c).assert(lhs_is_term_subst)
    }

    fn term_subst_bop_pat<PR: PatRecSgl>() -> TermSubstBopPat<PR> {
        let lhs = expr_leaf::<PR>();
        let ctx = assumption_leaf::<PR>();
        let expr = expr_leaf::<PR>();
        let op = binary_op_leaf::<PR>();
        let lhs_term = term_leaf::<PR>();
        let rhs_term = term_leaf::<PR>();
        let term = schema_dsl::TermBop::query(&op, &lhs_term, &rhs_term);
        let lhs_is_term_subst = lhs.handle().eq(&prim_call::<schema_dsl::Expr>(
            "TermSubst",
            vec![
                ctx.handle().into_handle_ty(),
                expr.handle().into_handle_ty(),
                term.handle().into_handle_ty(),
            ],
        ));

        TermSubstBopPat::new(lhs, ctx, expr, op, lhs_term, rhs_term).assert(lhs_is_term_subst)
    }

    fn term_subst_uop_pat<PR: PatRecSgl>() -> TermSubstUopPat<PR> {
        let lhs = expr_leaf::<PR>();
        let ctx = assumption_leaf::<PR>();
        let expr = expr_leaf::<PR>();
        let op = unary_op_leaf::<PR>();
        let term = term_leaf::<PR>();
        let matched = schema_dsl::TermUop::query(&op, &term);
        let lhs_is_term_subst = lhs.handle().eq(&prim_call::<schema_dsl::Expr>(
            "TermSubst",
            vec![
                ctx.handle().into_handle_ty(),
                expr.handle().into_handle_ty(),
                matched.handle().into_handle_ty(),
            ],
        ));

        TermSubstUopPat::new(lhs, ctx, expr, op, term).assert(lhs_is_term_subst)
    }

    fn term_subst_get_pat<PR: PatRecSgl>() -> TermSubstGetPat<PR> {
        let lhs = expr_leaf::<PR>();
        let ctx = assumption_leaf::<PR>();
        let expr = expr_leaf::<PR>();
        let term = term_leaf::<PR>();
        let index = BaseVar::<i64, PR>::query_named("term_subst_get_index");
        let lhs_is_term_subst = lhs.handle().eq(&prim_call::<schema_dsl::Expr>(
            "TermSubst",
            vec![
                ctx.handle().into_handle_ty(),
                expr.handle().into_handle_ty(),
                prim_call::<schema_dsl::Term>(
                    "TermGet",
                    vec![
                        term.handle().into_handle_ty(),
                        index.handle().into_handle_ty(),
                    ],
                )
                .into_handle_ty(),
            ],
        ));

        TermSubstGetPat::new(lhs, ctx, expr, term, index).assert(lhs_is_term_subst)
    }

    fn term_subst_alloc_pat<PR: PatRecSgl>() -> TermSubstAllocPat<PR> {
        let lhs = expr_leaf::<PR>();
        let ctx = assumption_leaf::<PR>();
        let expr = expr_leaf::<PR>();
        let id = BaseVar::<i64, PR>::query_named("term_subst_alloc_id");
        let amount = term_leaf::<PR>();
        let state = term_leaf::<PR>();
        let ty = base_type_leaf::<PR>();
        let lhs_is_term_subst = lhs.handle().eq(&prim_call::<schema_dsl::Expr>(
            "TermSubst",
            vec![
                ctx.handle().into_handle_ty(),
                expr.handle().into_handle_ty(),
                prim_call::<schema_dsl::Term>(
                    "TermAlloc",
                    vec![
                        id.handle().into_handle_ty(),
                        amount.handle().into_handle_ty(),
                        state.handle().into_handle_ty(),
                        ty.handle().into_handle_ty(),
                    ],
                )
                .into_handle_ty(),
            ],
        ));

        TermSubstAllocPat::new(lhs, ctx, expr, id, amount, state, ty).assert(lhs_is_term_subst)
    }

    fn term_subst_call_pat<PR: PatRecSgl>() -> TermSubstCallPat<PR> {
        let lhs = expr_leaf::<PR>();
        let ctx = assumption_leaf::<PR>();
        let expr = expr_leaf::<PR>();
        let name = BaseVar::<String, PR>::query_named("term_subst_call_name");
        let term = term_leaf::<PR>();
        let lhs_is_term_subst = lhs.handle().eq(&prim_call::<schema_dsl::Expr>(
            "TermSubst",
            vec![
                ctx.handle().into_handle_ty(),
                expr.handle().into_handle_ty(),
                prim_call::<schema_dsl::Term>(
                    "TermCall",
                    vec![
                        name.handle().into_handle_ty(),
                        term.handle().into_handle_ty(),
                    ],
                )
                .into_handle_ty(),
            ],
        ));

        TermSubstCallPat::new(lhs, ctx, expr, name, term).assert(lhs_is_term_subst)
    }

    fn term_subst_single_pat<PR: PatRecSgl>() -> TermSubstSinglePat<PR> {
        let lhs = expr_leaf::<PR>();
        let ctx = assumption_leaf::<PR>();
        let expr = expr_leaf::<PR>();
        let term = term_leaf::<PR>();
        let matched = schema_dsl::TermSingle::query(&term);
        let lhs_is_term_subst = lhs.handle().eq(&prim_call::<schema_dsl::Expr>(
            "TermSubst",
            vec![
                ctx.handle().into_handle_ty(),
                expr.handle().into_handle_ty(),
                matched.handle().into_handle_ty(),
            ],
        ));

        TermSubstSinglePat::new(lhs, ctx, expr, term).assert(lhs_is_term_subst)
    }

    fn term_subst_concat_pat<PR: PatRecSgl>() -> TermSubstConcatPat<PR> {
        let lhs = expr_leaf::<PR>();
        let ctx = assumption_leaf::<PR>();
        let expr = expr_leaf::<PR>();
        let lhs_term = term_leaf::<PR>();
        let rhs_term = term_leaf::<PR>();
        let matched = schema_dsl::TermConcat::query(&lhs_term, &rhs_term);
        let lhs_is_term_subst = lhs.handle().eq(&prim_call::<schema_dsl::Expr>(
            "TermSubst",
            vec![
                ctx.handle().into_handle_ty(),
                expr.handle().into_handle_ty(),
                matched.handle().into_handle_ty(),
            ],
        ));

        TermSubstConcatPat::new(lhs, ctx, expr, lhs_term, rhs_term).assert(lhs_is_term_subst)
    }

    pub(crate) fn register_native_rules() -> RuleSetId {
        let ruleset = RuleSetId("term-subst");

        PeepholeTx::add_rule(
            "term_subst_has_arg_type",
            ruleset,
            term_subst_type_pat,
            |ctx, pat| {
                ctx.insert_has_arg_type(pat.lhs, pat.ty);
            },
        );
        PeepholeTx::add_rule("term_subst_arg", ruleset, term_subst_arg_pat, |ctx, pat| {
            ctx.union(
                pat.lhs,
                eggplant::wrap::Value::<schema_dsl::Expr>::new((ctx).insert("AddContext", &[pat.ctx.val, pat.expr.val])),
            );
        });
        PeepholeTx::add_rule(
            "term_subst_const",
            ruleset,
            term_subst_const_pat,
            |ctx, pat| {
                ctx.union(
                    pat.lhs,
                    eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert("Const", &[pat.constant.val, pat.ty.val, pat.ctx.val])),
                );
            },
        );
        PeepholeTx::add_rule(
            "term_subst_empty",
            ruleset,
            term_subst_empty_pat,
            |ctx, pat| {
                ctx.union(
                    pat.lhs,
                    eggplant::wrap::Value::<schema_dsl::Expr>::new((ctx).insert("Empty", &[pat.ty.val, pat.ctx.val])),
                );
            },
        );
        PeepholeTx::add_rule("term_subst_top", ruleset, term_subst_top_pat, |ctx, pat| {
            let a = insert_term_subst(ctx, pat.ctx.val, pat.expr.val, pat.a.val);
            let b = insert_term_subst(ctx, pat.ctx.val, pat.expr.val, pat.b.val);
            let c = insert_term_subst(ctx, pat.ctx.val, pat.expr.val, pat.c.val);
            ctx.union(
                pat.lhs,
                eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert("Top", &[pat.op.val, a.val, b.val, c.val])),
            );
        });
        PeepholeTx::add_rule("term_subst_bop", ruleset, term_subst_bop_pat, |ctx, pat| {
            let lhs = insert_term_subst(ctx, pat.ctx.val, pat.expr.val, pat.lhs_term.val);
            let rhs = insert_term_subst(ctx, pat.ctx.val, pat.expr.val, pat.rhs_term.val);
            ctx.union(
                pat.lhs,
                eggplant::wrap::Value::<schema_dsl::Expr>::new((ctx).insert("Bop", &[pat.op.val, lhs.val, rhs.val])),
            );
        });
        PeepholeTx::add_rule("term_subst_uop", ruleset, term_subst_uop_pat, |ctx, pat| {
            let inner = insert_term_subst(ctx, pat.ctx.val, pat.expr.val, pat.term.val);
            ctx.union(
                pat.lhs,
                eggplant::wrap::Value::<schema_dsl::Expr>::new((ctx).insert("Uop", &[pat.op.val, inner.val])),
            );
        });
        PeepholeTx::add_rule("term_subst_get", ruleset, term_subst_get_pat, |ctx, pat| {
            let inner = insert_term_subst(ctx, pat.ctx.val, pat.expr.val, pat.term.val);
            ctx.union(pat.lhs, ctx.insert_get(inner, ctx.devalue(pat.index)));
        });
        PeepholeTx::add_rule(
            "term_subst_alloc",
            ruleset,
            term_subst_alloc_pat,
            |ctx, pat| {
                let amount = insert_term_subst(ctx, pat.ctx.val, pat.expr.val, pat.amount.val);
                let state = insert_term_subst(ctx, pat.ctx.val, pat.expr.val, pat.state.val);
                ctx.union(
                    pat.lhs,
                    eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert("Alloc", &[pat.id.val, amount.val, state.val, pat.ty.val])),
                );
            },
        );
        PeepholeTx::add_rule(
            "term_subst_call",
            ruleset,
            term_subst_call_pat,
            |ctx, pat| {
                let arg = insert_term_subst(ctx, pat.ctx.val, pat.expr.val, pat.term.val);
                ctx.union(
                    pat.lhs,
                    eggplant::wrap::Value::<schema_dsl::Expr>::new((ctx).insert("Call", &[pat.name.val, arg.val])),
                );
            },
        );
        PeepholeTx::add_rule(
            "term_subst_single",
            ruleset,
            term_subst_single_pat,
            |ctx, pat| {
                let inner = insert_term_subst(ctx, pat.ctx.val, pat.expr.val, pat.term.val);
                ctx.union(pat.lhs, ctx.insert_single(inner));
            },
        );
        PeepholeTx::add_rule(
            "term_subst_concat",
            ruleset,
            term_subst_concat_pat,
            |ctx, pat| {
                let lhs = insert_term_subst(ctx, pat.ctx.val, pat.expr.val, pat.lhs_term.val);
                let rhs = insert_term_subst(ctx, pat.ctx.val, pat.expr.val, pat.rhs_term.val);
                ctx.union(
                    pat.lhs,
                    eggplant::wrap::Value::<schema_dsl::Expr>::new((ctx).insert("Concat", &[lhs.val, rhs.val])),
                );
            },
        );

        ruleset
    }
}

#[cfg(all(test, feature = "eggplant"))]
mod native_tests {
    use crate::eggplant_backend::test_lock;
    use egglog::EGraph;

    fn candidate_parts() -> (String, String, String, String) {
        let ctx = "(InFunc \"RLCR\")".to_string();
        let expr = "(Const (Int 3) (Base (IntT)) (InFunc \"OLD\"))".to_string();
        let term =
            "(TermConcat (TermSingle (TermArg)) (TermSingle (TermConst (Int 9))))".to_string();
        let expected = "(Concat (Single (Const (Int 3) (Base (IntT)) (InFunc \"RLCR\"))) (Single (Const (Int 9) (Base (IntT)) (InFunc \"RLCR\"))))".to_string();
        (ctx, expr, term, expected)
    }

    fn term_subst_schedule() -> String {
        format!(
            "(run-schedule {})\n(run-schedule (saturate term-subst))\n(run-schedule (saturate context))",
            crate::schedule::types_and_indexing()
        )
    }

    fn text_term_subst_holds(
        prologue: &str,
        ctx: &str,
        expr: &str,
        term: &str,
        expected: &str,
        schedule: &str,
    ) {
        let program = format!(
            "{prologue}\n(let __rlcr_ctx {ctx})\n(let __rlcr_expr {expr})\n(let __rlcr_term {term})\n(let __rlcr_subst (TermSubst __rlcr_ctx __rlcr_expr __rlcr_term))\n{schedule}\n(check (= __rlcr_subst {expected}))\n"
        );
        let mut egraph = EGraph::default();
        egraph.parse_and_run_program(None, &program).unwrap();
    }

    fn native_term_subst_holds(
        prologue: &str,
        ctx: &str,
        expr: &str,
        term: &str,
        expected: &str,
        schedule: &str,
        ablate: Option<&str>,
    ) -> std::result::Result<(), eggplant::egglog::Error> {
        let initialization = format!(
            "(let __rlcr_ctx {ctx})\n(let __rlcr_expr {expr})\n(let __rlcr_term {term})\n(let __rlcr_subst (TermSubst __rlcr_ctx __rlcr_expr __rlcr_term))"
        );
        crate::with_native_rules_egraph(prologue, &initialization, schedule, ablate, |egraph| {
            egraph.parse_and_run_program(None, &format!("(check (= __rlcr_subst {expected}))"))?;
            Ok(())
        })
    }

    #[test]
    fn native_feature_path_matches_text_backend_for_term_subst_case() {
        let _guard = test_lock::lock();
        let (ctx, expr, term, expected) = candidate_parts();
        let schedule = term_subst_schedule();

        text_term_subst_holds(
            &crate::prologue_egglog_text(),
            &ctx,
            &expr,
            &term,
            &expected,
            &schedule,
        );
        native_term_subst_holds(
            &crate::feature_execution_prologue(true, None),
            &ctx,
            &expr,
            &term,
            &expected,
            &schedule,
            None,
        )
        .unwrap();

        let ablated = native_term_subst_holds(
            &crate::feature_execution_prologue(true, Some("term-subst")),
            &ctx,
            &expr,
            &term,
            &expected,
            &schedule,
            Some("term-subst"),
        );

        assert!(
            ablated.is_err(),
            "ablating term-subst should make the TermSubst witness fail",
        );
    }
}
