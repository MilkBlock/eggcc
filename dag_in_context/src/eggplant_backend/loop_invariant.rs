pub(crate) fn fragment() -> String {
    let mut out = String::new();
    out.push_str(GENERATED_MARKER);
    out.push_str(LOOP_INVARIANT);
    out.push('\n');
    out
}

#[cfg(feature = "eggplant")]
pub(crate) fn native_fragment() -> String {
    let mut out = String::new();
    out.push_str(GENERATED_MARKER);
    out.push_str(LOOP_INVARIANT_SUPPORT);
    out.push('\n');
    use crate::schema_helpers::Constructor::{
        Bop, Call, Concat, Const, DoWhile, Empty, Function, Get, If, Single, Top, Uop,
    };
    out.push_str(
        &crate::optimizations::loop_invariant::generated_rules_excluding(&[
            Function, Const, Empty, Get, Top, Bop, Uop, Call, If, Single, Concat, DoWhile,
        ])
        .join("\n"),
    );
    out
}

pub(crate) fn rules() -> String {
    let mut out = fragment();
    out.push('\n');
    out.push_str(&crate::optimizations::loop_invariant::generated_rules().join("\n"));
    out
}

const GENERATED_MARKER: &str =
    "; (Generated from eggplant Rust: src/eggplant_backend/loop_invariant.rs)\n";
const LOOP_INVARIANT_SUPPORT: &str = r#";; Loop Invariant

;; For a loop body, this expression is invariant (calculates the same value every loop)
;;                     body, inv-expr
(relation is-inv-Expr (Expr Expr))
(relation is-inv-ListExpr (Expr ListExpr))

(relation is-inv-ListExpr-helper (Expr ListExpr i64))


(ruleset loop-invariant-generated)

(ruleset boundary-analysis)
(ruleset boundary-analysis-prep)

;; We use a function so that we choose just one
;; thing to hoist at a time
;; This is an evil hack!
;                   inputs body invariant-expr
(function to-hoist (Expr Expr) Expr :merge new)
;; The biggest invariant expression's size
(function to-hoist-size (Expr Expr) i64 :merge (max old new))

;; mock function
(ruleset loop-inv-motion)"#;
const LOOP_INVARIANT: &str = r#";; Loop Invariant

;; For a loop body, this expression is invariant (calculates the same value every loop)
;;                     body, inv-expr
(relation is-inv-Expr (Expr Expr))
(relation is-inv-ListExpr (Expr ListExpr))

(relation is-inv-ListExpr-helper (Expr ListExpr i64))
(rule ((BodyContainsListExpr body list) ) 
      ((is-inv-ListExpr-helper body list 0)) :ruleset always-run)

(rule ((is-inv-ListExpr-helper body list i)
       (is-inv-Expr body (ListExpr-ith list i)))
    ((is-inv-ListExpr-helper body list (+ i 1))) :ruleset always-run)

(rule ((is-inv-ListExpr-helper body list i)
       (= i (ListExpr-length list)))
    ((is-inv-ListExpr body list)) :ruleset always-run)


(ruleset loop-invariant-generated)

(ruleset boundary-analysis)
(ruleset boundary-analysis-prep)

;; We use a function so that we choose just one
;; thing to hoist at a time
;; This is an evil hack!
;                   inputs body invariant-expr
(function to-hoist (Expr Expr) Expr :merge new)
;; The biggest invariant expression's size
(function to-hoist-size (Expr Expr) i64 :merge (max old new))

;; figure out max cost of invariant to hoist
;; pick a random invariant expression
(rule ((is-inv-Expr body expr)
       (DoWhile inputs body)
       (HasType expr inv_type)
       (= inv_type (Base base_inv_ty))
       (= size (Expr-size expr)))
      ((set (to-hoist-size inputs body) size)) :ruleset boundary-analysis-prep)

;; pick a bigger one if possible
(rule ((is-inv-Expr body expr)
       (DoWhile inputs body)
       (HasType expr inv_type)
       (= inv_type (Base base_inv_ty))
       (= (Expr-size expr) (to-hoist-size inputs body)))
      ((set (to-hoist inputs body) expr)) :ruleset boundary-analysis)

;; mock function
(ruleset loop-inv-motion)

(rule ((= (to-hoist in body) inv)
       (> (Expr-size inv) 1)
       ;; TODO: replace Expr-size when cost model is ready
       (= loop (DoWhile in body))
       ;; the outter assumption of the loop 
       (ContextOf loop loop_ctx)
       (HasType in in_type)
       (HasType inv inv_type)
       (= inv_type (Base base_inv_ty))
       (= in_type (TupleT tylist))
       (= len (tuple-length in))
       (= iter-guess (LoopNumItersGuess in body)))
      ((RELIESONCONTEXT)
       (let new_input (Concat in (Single (Subst loop_ctx in inv))))
       (let new_input_type (TupleT (TLConcat tylist (TCons base_inv_ty (TNil)))))

       ;; create an virtual assume node, union it with actuall InLoop later
       (let assum (TmpCtx))
       (let new_out_branch (Get (Arg new_input_type assum) len))

       ;; this two subst only change arg to arg with new type
       (let substed_body
         (Subst assum
               (SubTuple (Arg new_input_type assum) 0 len) body))
       (let inv_in_new_loop
            (Subst assum (SubTuple (Arg new_input_type assum) 0 len) inv))
       (let new_body (Concat substed_body (Single new_out_branch)))
       
       (let new_loop (DoWhile new_input new_body))
       (union assum (InLoop new_input new_body))
       (union inv_in_new_loop new_out_branch)
       (let wrapper (SubTuple new_loop 0 len))
       (union loop wrapper)
       (subsume (DoWhile in body)) 
       (delete (TmpCtx))
       (set (LoopNumItersGuess new_input new_body) iter-guess)
      )
       :ruleset loop-inv-motion)"#;

#[cfg(feature = "eggplant")]
pub(crate) mod native {
    use super::super::native_rule_helpers::insert_call;
    use super::super::schema_dsl;
    use crate::eggplant_backend::peepholes::native::PeepholeTx;
    use eggplant::prelude::{
        prim_call, AsHandle, BaseVar, Insertable, IntoHandleTy, PEq, PatRecSgl, RuleRunnerSgl,
        RuleSetId,
    };

    #[eggplant::pat_vars]
    struct LoopInvariantListHelperSeedPat<PR: PatRecSgl> {
        body: schema_dsl::Expr,
        list: schema_dsl::ListExpr,
    }

    fn loop_invariant_list_helper_seed_pat<PR: PatRecSgl>() -> LoopInvariantListHelperSeedPat<PR> {
        let body = schema_dsl::Expr::query_leaf();
        let list = schema_dsl::ListExpr::query_leaf();
        let body_contains_list = body_contains_list_expr(&body, &list);

        LoopInvariantListHelperSeedPat::new(body, list).assert(body_contains_list)
    }

    #[eggplant::pat_vars]
    struct LoopInvariantListHelperRecursePat<PR: PatRecSgl> {
        body: schema_dsl::Expr,
        list: schema_dsl::ListExpr,
        i: i64,
    }

    fn loop_invariant_list_helper_recurse_pat<PR: PatRecSgl>(
    ) -> LoopInvariantListHelperRecursePat<PR> {
        let body = schema_dsl::Expr::query_leaf();
        let list = schema_dsl::ListExpr::query_leaf();
        let i = BaseVar::<i64, PR>::query_named("list_index");
        let helper_entry = eggplant::wrap::FactCallConstraint {
            op: "is-inv-ListExpr-helper",
            operands: vec![
                body.handle().into_handle_ty(),
                list.handle().into_handle_ty(),
                i.handle().into_handle_ty(),
            ],
        };
        let ith_is_inv = eggplant::wrap::FactCallConstraint {
            op: "is-inv-Expr",
            operands: vec![
                body.handle().into_handle_ty(),
                prim_call::<schema_dsl::Expr>(
                    "ListExpr-ith",
                    vec![list.handle().into_handle_ty(), i.handle().into_handle_ty()],
                )
                .into_handle_ty(),
            ],
        };

        LoopInvariantListHelperRecursePat::new(body, list, i)
            .assert(helper_entry)
            .assert(ith_is_inv)
    }

    #[eggplant::pat_vars]
    struct LoopInvariantListHelperFinishPat<PR: PatRecSgl> {
        body: schema_dsl::Expr,
        list: schema_dsl::ListExpr,
    }

    fn loop_invariant_list_helper_finish_pat<PR: PatRecSgl>() -> LoopInvariantListHelperFinishPat<PR>
    {
        let body = schema_dsl::Expr::query_leaf();
        let list = schema_dsl::ListExpr::query_leaf();
        let i = BaseVar::<i64, PR>::query_named("list_index");
        let helper_entry = eggplant::wrap::FactCallConstraint {
            op: "is-inv-ListExpr-helper",
            operands: vec![
                body.handle().into_handle_ty(),
                list.handle().into_handle_ty(),
                i.handle().into_handle_ty(),
            ],
        };
        let reaches_end = list_expr_length_query(&list, &i);

        LoopInvariantListHelperFinishPat::new(body, list)
            .assert(helper_entry)
            .assert(reaches_end)
    }

    fn fresh_const_expr<PR: PatRecSgl>() -> schema_dsl::Expr<PR, schema_dsl::ConstTy> {
        schema_dsl::Const::query(
            &schema_dsl::Constant::query_leaf(),
            &schema_dsl::Type::query_leaf(),
            &schema_dsl::Assumption::query_leaf(),
        )
    }

    fn fresh_empty_expr<PR: PatRecSgl>() -> schema_dsl::Expr<PR, schema_dsl::EmptyTy> {
        schema_dsl::Empty::query(
            &schema_dsl::Type::query_leaf(),
            &schema_dsl::Assumption::query_leaf(),
        )
    }

    fn fresh_arg_expr<PR: PatRecSgl>() -> schema_dsl::Expr<PR, schema_dsl::ArgTy> {
        schema_dsl::Arg::query(
            &schema_dsl::Type::query_leaf(),
            &schema_dsl::Assumption::query_leaf(),
        )
    }

    fn expr_leaf<PR: PatRecSgl>() -> schema_dsl::Expr<PR> {
        schema_dsl::Expr::query_leaf()
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

    fn type_leaf<PR: PatRecSgl>() -> schema_dsl::Type<PR> {
        schema_dsl::Type::query_leaf()
    }

    fn base_type_leaf<PR: PatRecSgl>() -> schema_dsl::BaseType<PR> {
        schema_dsl::BaseType::query_leaf()
    }

    fn type_list_leaf<PR: PatRecSgl>() -> schema_dsl::TypeList<PR> {
        schema_dsl::TypeList::query_leaf()
    }

    fn assumption_leaf<PR: PatRecSgl>() -> schema_dsl::Assumption<PR> {
        schema_dsl::Assumption::query_leaf()
    }

    fn body_contains_expr<PR: PatRecSgl>(
        body: &schema_dsl::Expr<PR>,
        expr: &schema_dsl::Expr<PR>,
    ) -> eggplant::wrap::FactCallConstraint {
        eggplant::wrap::FactCallConstraint {
            op: "BodyContainsExpr",
            operands: vec![
                body.handle().into_handle_ty(),
                expr.handle().into_handle_ty(),
            ],
        }
    }

    fn body_contains_list_expr<PR: PatRecSgl>(
        body: &schema_dsl::Expr<PR>,
        list: &schema_dsl::ListExpr<PR>,
    ) -> eggplant::wrap::FactCallConstraint {
        eggplant::wrap::FactCallConstraint {
            op: "BodyContainsListExpr",
            operands: vec![
                body.handle().into_handle_ty(),
                list.handle().into_handle_ty(),
            ],
        }
    }

    fn is_inv_expr<PR: PatRecSgl>(
        body: &schema_dsl::Expr<PR>,
        expr: &schema_dsl::Expr<PR>,
    ) -> eggplant::wrap::FactCallConstraint {
        eggplant::wrap::FactCallConstraint {
            op: "is-inv-Expr",
            operands: vec![
                body.handle().into_handle_ty(),
                expr.handle().into_handle_ty(),
            ],
        }
    }

    fn expr_size_query<PR: PatRecSgl>(
        expr: &schema_dsl::Expr<PR>,
        size: &BaseVar<i64, PR>,
    ) -> impl eggplant::wrap::constraint::IntoConstraintFact {
        size.handle().eq(&prim_call::<i64>(
            "Expr-size",
            vec![expr.handle().into_handle_ty()],
        ))
    }

    fn list_expr_length_query<PR: PatRecSgl>(
        list: &schema_dsl::ListExpr<PR>,
        len: &BaseVar<i64, PR>,
    ) -> impl eggplant::wrap::constraint::IntoConstraintFact {
        len.handle().eq(&prim_call::<i64>(
            "ListExpr-length",
            vec![list.handle().into_handle_ty()],
        ))
    }

    #[eggplant::pat_vars]
    struct LoopInvariantGeneratedConstBasePat<PR: PatRecSgl> {
        body: schema_dsl::Expr,
        expr: schema_dsl::Expr,
        loop_expr: schema_dsl::DoWhile,
    }

    fn loop_invariant_generated_const_base_pat<PR: PatRecSgl>(
    ) -> LoopInvariantGeneratedConstBasePat<PR> {
        let body = schema_dsl::Expr::query_leaf();
        let expr = schema_dsl::Expr::query_leaf();
        let inputs = schema_dsl::Expr::query_leaf();
        let loop_expr = schema_dsl::DoWhile::query(&inputs, &body);
        let body_contains_expr = body_contains_expr(&body, &expr);
        let expr_is_const = expr.handle().eq(&fresh_const_expr::<PR>().handle());

        LoopInvariantGeneratedConstBasePat::new(body, expr, loop_expr)
            .assert(body_contains_expr)
            .assert(expr_is_const)
    }

    #[eggplant::pat_vars]
    struct LoopInvariantGeneratedEmptyBasePat<PR: PatRecSgl> {
        body: schema_dsl::Expr,
        expr: schema_dsl::Expr,
        loop_expr: schema_dsl::DoWhile,
    }

    fn loop_invariant_generated_empty_base_pat<PR: PatRecSgl>(
    ) -> LoopInvariantGeneratedEmptyBasePat<PR> {
        let body = schema_dsl::Expr::query_leaf();
        let expr = schema_dsl::Expr::query_leaf();
        let inputs = schema_dsl::Expr::query_leaf();
        let loop_expr = schema_dsl::DoWhile::query(&inputs, &body);
        let body_contains_expr = body_contains_expr(&body, &expr);
        let expr_is_empty = expr.handle().eq(&fresh_empty_expr::<PR>().handle());

        LoopInvariantGeneratedEmptyBasePat::new(body, expr, loop_expr)
            .assert(body_contains_expr)
            .assert(expr_is_empty)
    }

    #[eggplant::pat_vars]
    struct LoopInvariantGeneratedGetBasePat<PR: PatRecSgl> {
        body: schema_dsl::Expr,
        expr: schema_dsl::Expr,
        loop_expr: schema_dsl::DoWhile,
    }

    fn loop_invariant_generated_get_base_pat<PR: PatRecSgl>() -> LoopInvariantGeneratedGetBasePat<PR>
    {
        let body = schema_dsl::Expr::query_leaf();
        let expr = schema_dsl::Expr::query_leaf();
        let inputs = schema_dsl::Expr::query_leaf();
        let loop_expr = schema_dsl::DoWhile::query(&inputs, &body);
        let arg_expr = fresh_arg_expr::<PR>();
        let i = BaseVar::<i64, PR>::query_named("inv_get_i");
        let next_i = BaseVar::<i64, PR>::query_named("inv_get_next_i");
        let body_contains_expr = body_contains_expr(&body, &expr);
        let expr_is_get_arg = expr.handle().eq(&prim_call::<schema_dsl::Expr>(
            "Get",
            vec![
                arg_expr.handle().into_handle_ty(),
                i.handle().into_handle_ty(),
            ],
        ));
        let expr_is_get_next = expr.handle().eq(&prim_call::<schema_dsl::Expr>(
            "Get",
            vec![
                body.handle().into_handle_ty(),
                next_i.handle().into_handle_ty(),
            ],
        ));
        let next_constraint = next_i.handle().eq(&(i.handle() + (&1_i64).as_handle()));

        LoopInvariantGeneratedGetBasePat::new(body, expr, loop_expr)
            .assert(body_contains_expr)
            .assert(expr_is_get_arg)
            .assert(expr_is_get_next)
            .assert(next_constraint)
    }

    #[eggplant::pat_vars]
    struct LoopInvariantGeneratedGetPat<PR: PatRecSgl> {
        body: schema_dsl::Expr,
        expr: schema_dsl::Expr,
        loop_expr: schema_dsl::DoWhile,
    }

    fn loop_invariant_generated_get_pat<PR: PatRecSgl>() -> LoopInvariantGeneratedGetPat<PR> {
        let body = schema_dsl::Expr::query_leaf();
        let expr = schema_dsl::Expr::query_leaf();
        let inputs = schema_dsl::Expr::query_leaf();
        let loop_expr = schema_dsl::DoWhile::query(&inputs, &body);
        let tuple_expr = expr_leaf::<PR>();
        let i = BaseVar::<i64, PR>::query_named("inv_generic_get_i");
        let body_contains_expr = body_contains_expr(&body, &expr);
        let expr_is_get = expr.handle().eq(&prim_call::<schema_dsl::Expr>(
            "Get",
            vec![
                tuple_expr.handle().into_handle_ty(),
                i.handle().into_handle_ty(),
            ],
        ));
        let tuple_is_inv = is_inv_expr(&body, &tuple_expr);

        LoopInvariantGeneratedGetPat::new(body, expr, loop_expr)
            .assert(body_contains_expr)
            .assert(expr_is_get)
            .assert(tuple_is_inv)
    }

    #[eggplant::pat_vars]
    struct LoopInvariantGeneratedTopPat<PR: PatRecSgl> {
        body: schema_dsl::Expr,
        expr: schema_dsl::Expr,
        loop_expr: schema_dsl::DoWhile,
    }

    fn loop_invariant_generated_top_pat<PR: PatRecSgl>() -> LoopInvariantGeneratedTopPat<PR> {
        let body = schema_dsl::Expr::query_leaf();
        let expr = schema_dsl::Expr::query_leaf();
        let inputs = schema_dsl::Expr::query_leaf();
        let loop_expr = schema_dsl::DoWhile::query(&inputs, &body);
        let op = ternary_op_leaf::<PR>();
        let a = expr_leaf::<PR>();
        let b = expr_leaf::<PR>();
        let c = expr_leaf::<PR>();
        let top_expr = schema_dsl::Top::query(&op, &a, &b, &c);
        let body_contains_expr = body_contains_expr(&body, &expr);
        let expr_is_top = expr.handle().eq(&top_expr.handle());
        let op_is_pure = eggplant::wrap::FactCallConstraint {
            op: "TernaryOpIsPure",
            operands: vec![op.handle().into_handle_ty()],
        };
        let a_is_inv = is_inv_expr(&body, &a);
        let b_is_inv = is_inv_expr(&body, &b);
        let c_is_inv = is_inv_expr(&body, &c);

        LoopInvariantGeneratedTopPat::new(body, expr, loop_expr)
            .assert(body_contains_expr)
            .assert(expr_is_top)
            .assert(op_is_pure)
            .assert(a_is_inv)
            .assert(b_is_inv)
            .assert(c_is_inv)
    }

    #[eggplant::pat_vars]
    struct LoopInvariantGeneratedBopPat<PR: PatRecSgl> {
        body: schema_dsl::Expr,
        expr: schema_dsl::Expr,
        loop_expr: schema_dsl::DoWhile,
    }

    fn loop_invariant_generated_bop_pat<PR: PatRecSgl>() -> LoopInvariantGeneratedBopPat<PR> {
        let body = schema_dsl::Expr::query_leaf();
        let expr = schema_dsl::Expr::query_leaf();
        let inputs = schema_dsl::Expr::query_leaf();
        let loop_expr = schema_dsl::DoWhile::query(&inputs, &body);
        let op = binary_op_leaf::<PR>();
        let lhs = expr_leaf::<PR>();
        let rhs = expr_leaf::<PR>();
        let body_contains_expr = body_contains_expr(&body, &expr);
        let expr_is_bop = expr
            .handle()
            .eq(&schema_dsl::Bop::query(&op, &lhs, &rhs).handle());
        let op_is_pure = eggplant::wrap::FactCallConstraint {
            op: "BinaryOpIsPure",
            operands: vec![op.handle().into_handle_ty()],
        };
        let lhs_is_inv = is_inv_expr(&body, &lhs);
        let rhs_is_inv = is_inv_expr(&body, &rhs);

        LoopInvariantGeneratedBopPat::new(body, expr, loop_expr)
            .assert(body_contains_expr)
            .assert(expr_is_bop)
            .assert(op_is_pure)
            .assert(lhs_is_inv)
            .assert(rhs_is_inv)
    }

    #[eggplant::pat_vars]
    struct LoopInvariantGeneratedUopPat<PR: PatRecSgl> {
        body: schema_dsl::Expr,
        expr: schema_dsl::Expr,
        loop_expr: schema_dsl::DoWhile,
    }

    fn loop_invariant_generated_uop_pat<PR: PatRecSgl>() -> LoopInvariantGeneratedUopPat<PR> {
        let body = schema_dsl::Expr::query_leaf();
        let expr = schema_dsl::Expr::query_leaf();
        let inputs = schema_dsl::Expr::query_leaf();
        let loop_expr = schema_dsl::DoWhile::query(&inputs, &body);
        let op = unary_op_leaf::<PR>();
        let inner = expr_leaf::<PR>();
        let body_contains_expr = body_contains_expr(&body, &expr);
        let expr_is_uop = expr
            .handle()
            .eq(&schema_dsl::Uop::query(&op, &inner).handle());
        let op_is_pure = eggplant::wrap::FactCallConstraint {
            op: "UnaryOpIsPure",
            operands: vec![op.handle().into_handle_ty()],
        };
        let inner_is_inv = is_inv_expr(&body, &inner);

        LoopInvariantGeneratedUopPat::new(body, expr, loop_expr)
            .assert(body_contains_expr)
            .assert(expr_is_uop)
            .assert(op_is_pure)
            .assert(inner_is_inv)
    }

    #[eggplant::pat_vars]
    struct LoopInvariantGeneratedFunctionPat<PR: PatRecSgl> {
        body: schema_dsl::Expr,
        expr: schema_dsl::Expr,
        loop_expr: schema_dsl::DoWhile,
    }

    fn loop_invariant_generated_function_pat<PR: PatRecSgl>(
    ) -> LoopInvariantGeneratedFunctionPat<PR> {
        let body = schema_dsl::Expr::query_leaf();
        let expr = schema_dsl::Expr::query_leaf();
        let inputs = schema_dsl::Expr::query_leaf();
        let loop_expr = schema_dsl::DoWhile::query(&inputs, &body);
        let input_ty = type_leaf::<PR>();
        let output_ty = type_leaf::<PR>();
        let output = expr_leaf::<PR>();
        let body_contains_expr = body_contains_expr(&body, &expr);
        let expr_is_function = expr
            .handle()
            .eq(&schema_dsl::Function::query(&input_ty, &output_ty, &output).handle());

        LoopInvariantGeneratedFunctionPat::new(body, expr, loop_expr)
            .assert(body_contains_expr)
            .assert(expr_is_function)
    }

    #[eggplant::pat_vars]
    struct LoopInvariantGeneratedIfPat<PR: PatRecSgl> {
        body: schema_dsl::Expr,
        expr: schema_dsl::Expr,
        loop_expr: schema_dsl::DoWhile,
    }

    fn loop_invariant_generated_if_pat<PR: PatRecSgl>() -> LoopInvariantGeneratedIfPat<PR> {
        let body = schema_dsl::Expr::query_leaf();
        let expr = schema_dsl::Expr::query_leaf();
        let inputs = schema_dsl::Expr::query_leaf();
        let loop_expr = schema_dsl::DoWhile::query(&inputs, &body);
        let pred = expr_leaf::<PR>();
        let input = expr_leaf::<PR>();
        let then_branch = expr_leaf::<PR>();
        let else_branch = expr_leaf::<PR>();
        let body_contains_expr = body_contains_expr(&body, &expr);
        let expr_is_if =
            expr.handle()
                .eq(&schema_dsl::If::query(&pred, &input, &then_branch, &else_branch).handle());
        let pred_is_inv = is_inv_expr(&body, &pred);
        let input_is_inv = is_inv_expr(&body, &input);

        LoopInvariantGeneratedIfPat::new(body, expr, loop_expr)
            .assert(body_contains_expr)
            .assert(expr_is_if)
            .assert(pred_is_inv)
            .assert(input_is_inv)
    }

    #[eggplant::pat_vars]
    struct LoopInvariantGeneratedCallPat<PR: PatRecSgl> {
        body: schema_dsl::Expr,
        expr: schema_dsl::Expr,
        loop_expr: schema_dsl::DoWhile,
    }

    fn loop_invariant_generated_call_pat<PR: PatRecSgl>() -> LoopInvariantGeneratedCallPat<PR> {
        let body = schema_dsl::Expr::query_leaf();
        let expr = schema_dsl::Expr::query_leaf();
        let inputs = schema_dsl::Expr::query_leaf();
        let loop_expr = schema_dsl::DoWhile::query(&inputs, &body);
        let arg = expr_leaf::<PR>();
        let body_contains_expr = body_contains_expr(&body, &expr);
        let expr_is_call = expr.handle().eq(&schema_dsl::Call::query(&arg).handle());
        let arg_is_inv = is_inv_expr(&body, &arg);
        let expr_is_pure = eggplant::wrap::FactCallConstraint {
            op: "ExprIsPure",
            operands: vec![expr.handle().into_handle_ty()],
        };

        LoopInvariantGeneratedCallPat::new(body, expr, loop_expr)
            .assert(body_contains_expr)
            .assert(expr_is_call)
            .assert(arg_is_inv)
            .assert(expr_is_pure)
    }

    #[eggplant::pat_vars]
    struct LoopInvariantGeneratedSinglePat<PR: PatRecSgl> {
        body: schema_dsl::Expr,
        expr: schema_dsl::Expr,
        loop_expr: schema_dsl::DoWhile,
    }

    fn loop_invariant_generated_single_pat<PR: PatRecSgl>() -> LoopInvariantGeneratedSinglePat<PR> {
        let body = schema_dsl::Expr::query_leaf();
        let expr = schema_dsl::Expr::query_leaf();
        let inputs = schema_dsl::Expr::query_leaf();
        let loop_expr = schema_dsl::DoWhile::query(&inputs, &body);
        let inner = expr_leaf::<PR>();
        let body_contains_expr = body_contains_expr(&body, &expr);
        let expr_is_single = expr
            .handle()
            .eq(&schema_dsl::Single::query(&inner).handle());
        let inner_is_inv = is_inv_expr(&body, &inner);

        LoopInvariantGeneratedSinglePat::new(body, expr, loop_expr)
            .assert(body_contains_expr)
            .assert(expr_is_single)
            .assert(inner_is_inv)
    }

    #[eggplant::pat_vars]
    struct LoopInvariantGeneratedDoWhilePat<PR: PatRecSgl> {
        body: schema_dsl::Expr,
        expr: schema_dsl::Expr,
        loop_expr: schema_dsl::DoWhile,
    }

    fn loop_invariant_generated_dowhile_pat<PR: PatRecSgl>() -> LoopInvariantGeneratedDoWhilePat<PR>
    {
        let body = schema_dsl::Expr::query_leaf();
        let expr = schema_dsl::Expr::query_leaf();
        let inputs = schema_dsl::Expr::query_leaf();
        let loop_expr = schema_dsl::DoWhile::query(&inputs, &body);
        let inner_inputs = expr_leaf::<PR>();
        let pred_and_outputs = expr_leaf::<PR>();
        let body_contains_expr = body_contains_expr(&body, &expr);
        let expr_is_dowhile =
            expr.handle()
                .eq(&schema_dsl::DoWhile::query(&inner_inputs, &pred_and_outputs).handle());
        let inner_inputs_is_inv = is_inv_expr(&body, &inner_inputs);
        let expr_is_pure = eggplant::wrap::FactCallConstraint {
            op: "ExprIsPure",
            operands: vec![expr.handle().into_handle_ty()],
        };

        LoopInvariantGeneratedDoWhilePat::new(body, expr, loop_expr)
            .assert(body_contains_expr)
            .assert(expr_is_dowhile)
            .assert(inner_inputs_is_inv)
            .assert(expr_is_pure)
    }

    #[eggplant::pat_vars]
    struct LoopInvariantGeneratedConcatPat<PR: PatRecSgl> {
        body: schema_dsl::Expr,
        expr: schema_dsl::Expr,
        loop_expr: schema_dsl::DoWhile,
    }

    fn loop_invariant_generated_concat_pat<PR: PatRecSgl>() -> LoopInvariantGeneratedConcatPat<PR> {
        let body = schema_dsl::Expr::query_leaf();
        let expr = schema_dsl::Expr::query_leaf();
        let inputs = schema_dsl::Expr::query_leaf();
        let loop_expr = schema_dsl::DoWhile::query(&inputs, &body);
        let lhs = expr_leaf::<PR>();
        let rhs = expr_leaf::<PR>();
        let body_contains_expr = body_contains_expr(&body, &expr);
        let expr_is_concat = expr
            .handle()
            .eq(&schema_dsl::Concat::query(&lhs, &rhs).handle());
        let lhs_is_inv = is_inv_expr(&body, &lhs);
        let rhs_is_inv = is_inv_expr(&body, &rhs);

        LoopInvariantGeneratedConcatPat::new(body, expr, loop_expr)
            .assert(body_contains_expr)
            .assert(expr_is_concat)
            .assert(lhs_is_inv)
            .assert(rhs_is_inv)
    }

    #[eggplant::pat_vars]
    struct LoopInvariantBoundaryAnalysisPrepPat<PR: PatRecSgl> {
        inputs: schema_dsl::Expr,
        body: schema_dsl::Expr,
        size: i64,
        loop_expr: schema_dsl::DoWhile,
    }

    fn loop_invariant_boundary_analysis_prep_pat<PR: PatRecSgl>(
    ) -> LoopInvariantBoundaryAnalysisPrepPat<PR> {
        let inputs = expr_leaf::<PR>();
        let body = expr_leaf::<PR>();
        let expr = expr_leaf::<PR>();
        let loop_expr = schema_dsl::DoWhile::query(&inputs, &body);
        let base_inv_ty = base_type_leaf::<PR>();
        let size = BaseVar::<i64, PR>::query_named("inv_size");
        let expr_is_inv = is_inv_expr(&body, &expr);
        let expr_has_base_type = eggplant::wrap::FactCallConstraint {
            op: "HasType",
            operands: vec![
                expr.handle().into_handle_ty(),
                schema_dsl::Base::query(&base_inv_ty)
                    .handle()
                    .into_handle_ty(),
            ],
        };
        let expr_size = expr_size_query(&expr, &size);

        LoopInvariantBoundaryAnalysisPrepPat::new(inputs, body, size, loop_expr)
            .assert(expr_is_inv)
            .assert(expr_has_base_type)
            .assert(expr_size)
    }

    #[eggplant::pat_vars]
    struct LoopInvariantBoundaryAnalysisPickPat<PR: PatRecSgl> {
        inputs: schema_dsl::Expr,
        body: schema_dsl::Expr,
        expr: schema_dsl::Expr,
        loop_expr: schema_dsl::DoWhile,
    }

    fn loop_invariant_boundary_analysis_pick_pat<PR: PatRecSgl>(
    ) -> LoopInvariantBoundaryAnalysisPickPat<PR> {
        let inputs = expr_leaf::<PR>();
        let body = expr_leaf::<PR>();
        let expr = expr_leaf::<PR>();
        let loop_expr = schema_dsl::DoWhile::query(&inputs, &body);
        let base_inv_ty = base_type_leaf::<PR>();
        let size = BaseVar::<i64, PR>::query_named("inv_size");
        let expr_is_inv = is_inv_expr(&body, &expr);
        let expr_has_base_type = eggplant::wrap::FactCallConstraint {
            op: "HasType",
            operands: vec![
                expr.handle().into_handle_ty(),
                schema_dsl::Base::query(&base_inv_ty)
                    .handle()
                    .into_handle_ty(),
            ],
        };
        let expr_size = expr_size_query(&expr, &size);
        let matches_best_size = size.handle().eq(&prim_call::<i64>(
            "to-hoist-size",
            vec![
                inputs.handle().into_handle_ty(),
                body.handle().into_handle_ty(),
            ],
        ));

        LoopInvariantBoundaryAnalysisPickPat::new(inputs, body, expr, loop_expr)
            .assert(expr_is_inv)
            .assert(expr_has_base_type)
            .assert(expr_size)
            .assert(matches_best_size)
    }

    #[eggplant::pat_vars]
    struct LoopInvMotionPat<PR: PatRecSgl> {
        loop_expr: schema_dsl::DoWhile,
        in_expr: schema_dsl::Expr,
        body: schema_dsl::Expr,
        inv: schema_dsl::Expr,
        loop_ctx: schema_dsl::Assumption,
        tylist: schema_dsl::TypeList,
        base_inv_ty: schema_dsl::BaseType,
    }

    fn loop_invariant_motion_pat<PR: PatRecSgl>() -> LoopInvMotionPat<PR> {
        let in_expr = expr_leaf::<PR>();
        let body = expr_leaf::<PR>();
        let loop_expr = schema_dsl::DoWhile::query(&in_expr, &body);
        let inv = expr_leaf::<PR>();
        let loop_ctx = assumption_leaf::<PR>();
        let tylist = type_list_leaf::<PR>();
        let base_inv_ty = base_type_leaf::<PR>();
        let inv_size = BaseVar::<i64, PR>::query_named("inv_size");
        let hoisted_inv = inv.handle().eq(&prim_call::<schema_dsl::Expr>(
            "to-hoist",
            vec![
                in_expr.handle().into_handle_ty(),
                body.handle().into_handle_ty(),
            ],
        ));
        let inv_size_fact = expr_size_query(&inv, &inv_size);
        let inv_is_large_enough = eggplant::wrap::FactCallConstraint {
            op: ">",
            operands: vec![
                inv_size.handle().into_handle_ty(),
                (&1_i64).as_handle().into_handle_ty(),
            ],
        };
        let loop_has_context = eggplant::wrap::FactCallConstraint {
            op: "ContextOf",
            operands: vec![
                loop_expr.handle().into_handle_ty(),
                loop_ctx.handle().into_handle_ty(),
            ],
        };
        let input_has_tuple_type = eggplant::wrap::FactCallConstraint {
            op: "HasType",
            operands: vec![
                in_expr.handle().into_handle_ty(),
                schema_dsl::TupleT::query(&tylist).handle().into_handle_ty(),
            ],
        };
        let invariant_has_base_type = eggplant::wrap::FactCallConstraint {
            op: "HasType",
            operands: vec![
                inv.handle().into_handle_ty(),
                schema_dsl::Base::query(&base_inv_ty)
                    .handle()
                    .into_handle_ty(),
            ],
        };
        let len = BaseVar::<i64, PR>::query_named("len");
        let iter_guess = BaseVar::<i64, PR>::query_named("iter_guess");
        let input_tuple_len = len.handle().eq(&prim_call::<i64>(
            "tuple-length",
            vec![in_expr.handle().into_handle_ty()],
        ));
        let iter_guess = iter_guess.handle().eq(&prim_call::<i64>(
            "LoopNumItersGuess",
            vec![
                in_expr.handle().into_handle_ty(),
                body.handle().into_handle_ty(),
            ],
        ));

        LoopInvMotionPat::new(loop_expr, in_expr, body, inv, loop_ctx, tylist, base_inv_ty)
            .assert(hoisted_inv)
            .assert(inv_size_fact)
            .assert(inv_is_large_enough)
            .assert(loop_has_context)
            .assert(input_has_tuple_type)
            .assert(invariant_has_base_type)
            .assert(input_tuple_len)
            .assert(iter_guess)
    }

    pub(crate) fn register_native_support_rules() -> RuleSetId {
        let always_run = RuleSetId("always-run");
        let generated_ruleset = RuleSetId("loop-invariant-generated");
        let boundary_analysis = RuleSetId("boundary-analysis");
        let boundary_analysis_prep = RuleSetId("boundary-analysis-prep");

        PeepholeTx::add_rule(
            "loop_invariant_list_helper_seed",
            always_run,
            loop_invariant_list_helper_seed_pat,
            |ctx, pat| {
                let zero = ctx._intern_base::<i64, i64>(0);
                ctx.insert_func_tbl(
                    "is-inv-ListExpr-helper",
                    &[
                        pat.body.to_value(&ctx.ctx).val,
                        pat.list.to_value(&ctx.ctx).val,
                        zero,
                    ],
                );
            },
        );

        PeepholeTx::add_rule(
            "loop_invariant_list_helper_recurse",
            always_run,
            loop_invariant_list_helper_recurse_pat,
            |ctx, pat| {
                let next_i = ctx._intern_base::<i64, i64>(ctx.devalue(pat.i) + 1);
                ctx.insert_func_tbl(
                    "is-inv-ListExpr-helper",
                    &[
                        pat.body.to_value(&ctx.ctx).val,
                        pat.list.to_value(&ctx.ctx).val,
                        next_i,
                    ],
                );
            },
        );

        PeepholeTx::add_rule(
            "loop_invariant_list_helper_finish",
            always_run,
            loop_invariant_list_helper_finish_pat,
            |ctx, pat| {
                ctx.insert_func_tbl(
                    "is-inv-ListExpr",
                    &[
                        pat.body.to_value(&ctx.ctx).val,
                        pat.list.to_value(&ctx.ctx).val,
                    ],
                );
            },
        );

        PeepholeTx::add_rule(
            "loop_invariant_generated_const_base",
            generated_ruleset,
            loop_invariant_generated_const_base_pat,
            |ctx, pat| {
                ctx.insert_func_tbl(
                    "is-inv-Expr",
                    &[
                        pat.body.to_value(&ctx.ctx).val,
                        pat.expr.to_value(&ctx.ctx).val,
                    ],
                );
            },
        );

        PeepholeTx::add_rule(
            "loop_invariant_generated_empty_base",
            generated_ruleset,
            loop_invariant_generated_empty_base_pat,
            |ctx, pat| {
                ctx.insert_func_tbl(
                    "is-inv-Expr",
                    &[
                        pat.body.to_value(&ctx.ctx).val,
                        pat.expr.to_value(&ctx.ctx).val,
                    ],
                );
            },
        );

        PeepholeTx::add_rule(
            "loop_invariant_generated_get_base",
            generated_ruleset,
            loop_invariant_generated_get_base_pat,
            |ctx, pat| {
                ctx.insert_func_tbl(
                    "is-inv-Expr",
                    &[
                        pat.body.to_value(&ctx.ctx).val,
                        pat.expr.to_value(&ctx.ctx).val,
                    ],
                );
            },
        );

        PeepholeTx::add_rule(
            "loop_invariant_generated_get",
            generated_ruleset,
            loop_invariant_generated_get_pat,
            |ctx, pat| {
                ctx.insert_func_tbl(
                    "is-inv-Expr",
                    &[
                        pat.body.to_value(&ctx.ctx).val,
                        pat.expr.to_value(&ctx.ctx).val,
                    ],
                );
            },
        );

        PeepholeTx::add_rule(
            "loop_invariant_generated_top",
            generated_ruleset,
            loop_invariant_generated_top_pat,
            |ctx, pat| {
                ctx.insert_func_tbl(
                    "is-inv-Expr",
                    &[
                        pat.body.to_value(&ctx.ctx).val,
                        pat.expr.to_value(&ctx.ctx).val,
                    ],
                );
            },
        );

        PeepholeTx::add_rule(
            "loop_invariant_generated_bop",
            generated_ruleset,
            loop_invariant_generated_bop_pat,
            |ctx, pat| {
                ctx.insert_func_tbl(
                    "is-inv-Expr",
                    &[
                        pat.body.to_value(&ctx.ctx).val,
                        pat.expr.to_value(&ctx.ctx).val,
                    ],
                );
            },
        );

        PeepholeTx::add_rule(
            "loop_invariant_generated_uop",
            generated_ruleset,
            loop_invariant_generated_uop_pat,
            |ctx, pat| {
                ctx.insert_func_tbl(
                    "is-inv-Expr",
                    &[
                        pat.body.to_value(&ctx.ctx).val,
                        pat.expr.to_value(&ctx.ctx).val,
                    ],
                );
            },
        );

        PeepholeTx::add_rule(
            "loop_invariant_generated_function",
            generated_ruleset,
            loop_invariant_generated_function_pat,
            |ctx, pat| {
                ctx.insert_func_tbl(
                    "is-inv-Expr",
                    &[
                        pat.body.to_value(&ctx.ctx).val,
                        pat.expr.to_value(&ctx.ctx).val,
                    ],
                );
            },
        );

        PeepholeTx::add_rule(
            "loop_invariant_generated_if",
            generated_ruleset,
            loop_invariant_generated_if_pat,
            |ctx, pat| {
                ctx.insert_func_tbl(
                    "is-inv-Expr",
                    &[
                        pat.body.to_value(&ctx.ctx).val,
                        pat.expr.to_value(&ctx.ctx).val,
                    ],
                );
            },
        );

        PeepholeTx::add_rule(
            "loop_invariant_generated_call",
            generated_ruleset,
            loop_invariant_generated_call_pat,
            |ctx, pat| {
                ctx.insert_func_tbl(
                    "is-inv-Expr",
                    &[
                        pat.body.to_value(&ctx.ctx).val,
                        pat.expr.to_value(&ctx.ctx).val,
                    ],
                );
            },
        );

        PeepholeTx::add_rule(
            "loop_invariant_generated_single",
            generated_ruleset,
            loop_invariant_generated_single_pat,
            |ctx, pat| {
                ctx.insert_func_tbl(
                    "is-inv-Expr",
                    &[
                        pat.body.to_value(&ctx.ctx).val,
                        pat.expr.to_value(&ctx.ctx).val,
                    ],
                );
            },
        );

        PeepholeTx::add_rule(
            "loop_invariant_generated_dowhile",
            generated_ruleset,
            loop_invariant_generated_dowhile_pat,
            |ctx, pat| {
                ctx.insert_func_tbl(
                    "is-inv-Expr",
                    &[
                        pat.body.to_value(&ctx.ctx).val,
                        pat.expr.to_value(&ctx.ctx).val,
                    ],
                );
            },
        );

        PeepholeTx::add_rule(
            "loop_invariant_generated_concat",
            generated_ruleset,
            loop_invariant_generated_concat_pat,
            |ctx, pat| {
                ctx.insert_func_tbl(
                    "is-inv-Expr",
                    &[
                        pat.body.to_value(&ctx.ctx).val,
                        pat.expr.to_value(&ctx.ctx).val,
                    ],
                );
            },
        );

        PeepholeTx::add_rule(
            "loop_invariant_boundary_analysis_prep",
            boundary_analysis_prep,
            loop_invariant_boundary_analysis_prep_pat,
            |ctx, pat| {
                let size = ctx._intern_base::<i64, i64>(ctx.devalue(pat.size));
                ctx.insert_func_tbl(
                    "to-hoist-size",
                    &[
                        pat.inputs.to_value(&ctx.ctx).val,
                        pat.body.to_value(&ctx.ctx).val,
                        size,
                    ],
                );
            },
        );

        PeepholeTx::add_rule(
            "loop_invariant_boundary_analysis_pick",
            boundary_analysis,
            loop_invariant_boundary_analysis_pick_pat,
            |ctx, pat| {
                ctx.insert_func_tbl(
                    "to-hoist",
                    &[
                        pat.inputs.to_value(&ctx.ctx).val,
                        pat.body.to_value(&ctx.ctx).val,
                        pat.expr.to_value(&ctx.ctx).val,
                    ],
                );
            },
        );

        boundary_analysis
    }

    pub(crate) fn register_native_rules() -> RuleSetId {
        let ruleset = RuleSetId("loop-inv-motion");
        PeepholeTx::add_rule(
            "loop_invariant_motion",
            ruleset,
            loop_invariant_motion_pat,
            |ctx, pat| {
                let zero = ctx._intern_base::<i64, i64>(0);
                let len = ctx.lookup_expect("tuple-length", &[pat.in_expr.to_value(&ctx.ctx).val]);
                let iter_guess = ctx.lookup_expect(
                    "LoopNumItersGuess",
                    &[
                        pat.in_expr.to_value(&ctx.ctx).val,
                        pat.body.to_value(&ctx.ctx).val,
                    ],
                );

                let hoisted_inv = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "Subst",
                    &[
                        pat.loop_ctx.to_value(&ctx.ctx).val,
                        pat.in_expr.to_value(&ctx.ctx).val,
                        pat.inv.to_value(&ctx.ctx).val,
                    ],
                );
                let new_input = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "Concat",
                    &[
                        pat.in_expr.to_value(&ctx.ctx).val,
                        insert_call::<schema_dsl::Expr>(
                            &ctx.ctx,
                            "Single",
                            &[hoisted_inv.to_value(&ctx.ctx).val],
                        )
                        .to_value(&ctx.ctx)
                        .val,
                    ],
                );

                let tnil = insert_call::<schema_dsl::TypeList>(&ctx.ctx, "TNil", &[]);
                let appended_tail = insert_call::<schema_dsl::TypeList>(
                    &ctx.ctx,
                    "TCons",
                    &[pat.base_inv_ty.to_value(&ctx.ctx).val, tnil.0.val],
                );
                let new_tylist = insert_call::<schema_dsl::TypeList>(
                    &ctx.ctx,
                    "TLConcat",
                    &[
                        pat.tylist.to_value(&ctx.ctx).val,
                        appended_tail.to_value(&ctx.ctx).val,
                    ],
                );
                let new_input_type =
                    insert_call::<schema_dsl::Type>(&ctx.ctx, "TupleT", &[new_tylist.0.val]);

                let assum = insert_call::<schema_dsl::Assumption>(&ctx.ctx, "TmpCtx", &[]);
                let new_arg = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "Arg",
                    &[
                        new_input_type.to_value(&ctx.ctx).val,
                        assum.to_value(&ctx.ctx).val,
                    ],
                );
                let new_out_branch = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "Get",
                    &[new_arg.to_value(&ctx.ctx).val, len],
                );
                let old_arg_prefix = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "SubTuple",
                    &[new_arg.to_value(&ctx.ctx).val, zero, len],
                );
                let substed_body = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "Subst",
                    &[
                        assum.to_value(&ctx.ctx).val,
                        old_arg_prefix.to_value(&ctx.ctx).val,
                        pat.body.to_value(&ctx.ctx).val,
                    ],
                );
                let inv_in_new_loop = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "Subst",
                    &[
                        assum.to_value(&ctx.ctx).val,
                        old_arg_prefix.to_value(&ctx.ctx).val,
                        pat.inv.to_value(&ctx.ctx).val,
                    ],
                );
                let new_body = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "Concat",
                    &[
                        substed_body.to_value(&ctx.ctx).val,
                        insert_call::<schema_dsl::Expr>(
                            &ctx.ctx,
                            "Single",
                            &[new_out_branch.to_value(&ctx.ctx).val],
                        )
                        .to_value(&ctx.ctx)
                        .val,
                    ],
                );
                let new_loop = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "DoWhile",
                    &[
                        new_input.to_value(&ctx.ctx).val,
                        new_body.to_value(&ctx.ctx).val,
                    ],
                );
                let loop_ctx = insert_call::<schema_dsl::Assumption>(
                    &ctx.ctx,
                    "InLoop",
                    &[
                        new_input.to_value(&ctx.ctx).val,
                        new_body.to_value(&ctx.ctx).val,
                    ],
                );
                ctx.union(assum, loop_ctx);
                ctx.union(inv_in_new_loop, new_out_branch);

                let wrapper = insert_call::<schema_dsl::Expr>(
                    &ctx.ctx,
                    "SubTuple",
                    &[new_loop.to_value(&ctx.ctx).val, zero, len],
                );
                ctx.union(pat.loop_expr, wrapper);
                ctx.subsume(
                    "DoWhile",
                    &[
                        pat.in_expr.to_value(&ctx.ctx).val,
                        pat.body.to_value(&ctx.ctx).val,
                    ],
                );
                ctx.remove("TmpCtx", &[]);
                ctx.insert_func_tbl(
                    "LoopNumItersGuess",
                    &[
                        new_input.to_value(&ctx.ctx).val,
                        new_body.to_value(&ctx.ctx).val,
                        iter_guess,
                    ],
                );
            },
        );

        ruleset
    }
}

#[cfg(all(test, feature = "eggplant"))]
mod native_tests {
    use crate::add_context::ContextCache;
    use crate::ast::*;
    use crate::eggplant_backend::test_lock;
    use crate::schedule;
    use crate::schema::{Assumption, Expr, TreeProgram};
    use egglog::EGraph;

    fn support_witness_terms() -> (String, String, String, String, String, String, String) {
        let mut cache = ContextCache::new_dummy_ctx();
        let output_ty = tuplet!(intt(), intt(), intt(), statet());
        let inner_inv = getat(1);
        let inv_expr = add(add(add(inner_inv.clone(), int(1)), int(2)), int(3))
            .with_arg_types(output_ty.clone(), base(intt()));
        let single_inv_expr =
            single(inner_inv.clone()).with_arg_types(output_ty.clone(), tuplet!(intt()));
        let concat_inv_expr = concat(single_inv_expr.clone(), single(int(9)))
            .with_arg_types(output_ty.clone(), tuplet!(intt(), intt()));
        let empty_inv_expr = empty().with_arg_types(output_ty.clone(), emptyt());
        let concat_carrier_expr =
            get(concat_inv_expr.clone(), 0).with_arg_types(output_ty.clone(), base(intt()));
        let empty_tail_expr = concat(empty_inv_expr.clone(), single(int(9)))
            .with_arg_types(output_ty.clone(), tuplet!(intt()));
        let empty_carrier_expr =
            get(empty_tail_expr, 0).with_arg_types(output_ty.clone(), base(intt()));
        let print =
            tprint(inv_expr.clone(), getat(3)).with_arg_types(output_ty.clone(), base(statet()));
        let inputs_expr = parallel!(getat(0), getat(1), getat(2), getat(3))
            .with_arg_types(output_ty.clone(), output_ty.clone());
        let body_expr = parallel!(
            less_than(getat(0), getat(1)),
            int(3),
            concat_carrier_expr,
            empty_carrier_expr,
            print,
        )
        .with_arg_types(
            output_ty.clone(),
            tuplet!(boolt(), intt(), intt(), intt(), statet()),
        );
        let loop_ctx = inloop(inputs_expr.clone(), body_expr.clone());

        let inputs = inputs_expr
            .clone()
            .add_ctx_with_cache(Assumption::dummy(), &mut cache)
            .to_string();
        let loop_expr = dowhile(
            parallel!(getat(0), getat(1), getat(2), getat(3)),
            body_expr.clone(),
        )
        .with_arg_types(output_ty.clone(), output_ty.clone())
        .add_ctx_with_cache(Assumption::dummy(), &mut cache)
        .to_string();
        let body = body_expr
            .add_ctx_with_cache(loop_ctx.clone(), &mut cache)
            .to_string();
        let inv = inv_expr
            .add_ctx_with_cache(loop_ctx.clone(), &mut cache)
            .to_string();
        let single_inv = single_inv_expr
            .add_ctx_with_cache(loop_ctx.clone(), &mut cache)
            .to_string();
        let concat_inv = concat_inv_expr
            .add_ctx_with_cache(loop_ctx.clone(), &mut cache)
            .to_string();
        let empty_inv = empty_inv_expr
            .add_ctx_with_cache(loop_ctx, &mut cache)
            .to_string();

        (
            loop_expr, inputs, body, inv, single_inv, concat_inv, empty_inv,
        )
    }

    fn top_witness_terms() -> (String, String, String, String) {
        let mut cache = ContextCache::new_dummy_ctx();
        let output_ty = tuplet!(intt(), intt(), statet());
        let top_inv_expr = select(
            less_than(getat(1), int(10)).with_arg_types(output_ty.clone(), base(boolt())),
            add(getat(1), int(1)).with_arg_types(output_ty.clone(), base(intt())),
            add(getat(1), int(2)).with_arg_types(output_ty.clone(), base(intt())),
        )
        .with_arg_types(output_ty.clone(), base(intt()));
        let inputs_expr = parallel!(getat(0), getat(1), getat(2))
            .with_arg_types(output_ty.clone(), output_ty.clone());
        let body_expr = parallel!(
            less_than(getat(0), int(8)),
            top_inv_expr.clone(),
            getat(1),
            getat(2),
        )
        .with_arg_types(
            output_ty.clone(),
            tuplet!(boolt(), intt(), intt(), statet()),
        );
        let loop_ctx = inloop(inputs_expr.clone(), body_expr.clone());

        let inputs = inputs_expr
            .clone()
            .add_ctx_with_cache(Assumption::dummy(), &mut cache)
            .to_string();
        let loop_expr = dowhile(parallel!(getat(0), getat(1), getat(2)), body_expr.clone())
            .with_arg_types(output_ty.clone(), output_ty.clone())
            .add_ctx_with_cache(Assumption::dummy(), &mut cache)
            .to_string();
        let body = body_expr
            .add_ctx_with_cache(loop_ctx.clone(), &mut cache)
            .to_string();
        let top_inv = top_inv_expr
            .add_ctx_with_cache(loop_ctx, &mut cache)
            .to_string();

        (loop_expr, inputs, body, top_inv)
    }

    fn if_witness_terms() -> (String, String, String, String) {
        let mut cache = ContextCache::new_dummy_ctx();
        let output_ty = tuplet!(intt(), intt(), intt(), statet());
        let if_inv_expr =
            tif(ttrue(), empty(), int(4), int(5)).with_arg_types(output_ty.clone(), base(intt()));
        let inputs_expr = parallel!(getat(0), getat(1), getat(2), getat(3))
            .with_arg_types(output_ty.clone(), output_ty.clone());
        let body_expr = parallel!(
            less_than(getat(0), getat(1)),
            int(3),
            if_inv_expr.clone(),
            getat(2),
            getat(3),
        )
        .with_arg_types(
            output_ty.clone(),
            tuplet!(boolt(), intt(), intt(), intt(), statet()),
        );
        let loop_ctx = inloop(inputs_expr.clone(), body_expr.clone());

        let inputs = inputs_expr
            .clone()
            .add_ctx_with_cache(Assumption::dummy(), &mut cache)
            .to_string();
        let loop_expr = dowhile(
            parallel!(getat(0), getat(1), getat(2), getat(3)),
            body_expr.clone(),
        )
        .with_arg_types(output_ty.clone(), output_ty.clone())
        .add_ctx_with_cache(Assumption::dummy(), &mut cache)
        .to_string();
        let body = body_expr
            .add_ctx_with_cache(loop_ctx.clone(), &mut cache)
            .to_string();
        let if_inv = if_inv_expr
            .add_ctx_with_cache(loop_ctx, &mut cache)
            .to_string();

        (loop_expr, inputs, body, if_inv)
    }

    fn dowhile_witness_terms() -> (String, String, String, String) {
        let mut cache = ContextCache::new_dummy_ctx();
        let output_ty = tuplet!(intt(), intt(), statet());
        let dowhile_inv_expr = dowhile(single(int(9)), parallel!(ttrue(), getat(0)))
            .with_arg_types(output_ty.clone(), tuplet!(intt()));
        let carrier_expr =
            get(dowhile_inv_expr.clone(), 0).with_arg_types(output_ty.clone(), base(intt()));
        let inputs_expr = parallel!(getat(0), getat(1), getat(2))
            .with_arg_types(output_ty.clone(), output_ty.clone());
        let body_expr = parallel!(
            less_than(getat(0), int(8)),
            carrier_expr,
            getat(1),
            getat(2),
        )
        .with_arg_types(
            output_ty.clone(),
            tuplet!(boolt(), intt(), intt(), statet()),
        );
        let loop_ctx = inloop(inputs_expr.clone(), body_expr.clone());

        let inputs = inputs_expr
            .clone()
            .add_ctx_with_cache(Assumption::dummy(), &mut cache)
            .to_string();
        let loop_expr = dowhile(parallel!(getat(0), getat(1), getat(2)), body_expr.clone())
            .with_arg_types(output_ty.clone(), output_ty.clone())
            .add_ctx_with_cache(Assumption::dummy(), &mut cache)
            .to_string();
        let body = body_expr
            .add_ctx_with_cache(loop_ctx.clone(), &mut cache)
            .to_string();
        let dowhile_inv = dowhile_inv_expr
            .add_ctx_with_cache(loop_ctx, &mut cache)
            .to_string();

        (loop_expr, inputs, body, dowhile_inv)
    }

    fn helper_schedule() -> String {
        format!("(run-schedule {})", schedule::helpers())
    }

    fn run_text_check(prologue: &str, initialization: &str, schedule: &str) {
        let program = format!("{prologue}\n{initialization}\n{schedule}");
        let mut egraph = EGraph::default();
        egraph.parse_and_run_program(None, &program).unwrap();
    }

    fn run_native_check(
        prologue: &str,
        initialization: &str,
        schedule: &str,
        ablate: Option<&str>,
    ) {
        crate::with_native_rules_egraph(
            prologue,
            initialization,
            schedule,
            ablate,
            |_egraph| Ok(()),
        )
        .unwrap()
    }

    fn build_feature_program_parts(program: &TreeProgram, schedule: &str) -> (String, String) {
        let egglog_prog = crate::build_program(program, None, &program.fns(), schedule, None, true);
        let suffix_anchor = "(relation InlinedCall (String Expr))";
        let suffix_start = egglog_prog.find(suffix_anchor).expect(
            "build_program output must contain the InlinedCall relation (used as the prologue boundary for loop_invariant feature-path tests)",
        );
        let suffix = &egglog_prog[suffix_start..];
        let schedule_marker = "\n; Schedule\n";
        let (initialization, schedule) = suffix.split_once(schedule_marker).expect(
            "build_program output must contain a schedule section after the initialization suffix",
        );
        (initialization.to_string(), schedule.to_string())
    }

    fn call_witness_program_terms() -> (TreeProgram, String, String, String, String) {
        fn find_named_call(
            expr: &crate::schema::RcExpr,
            target: &str,
        ) -> Result<crate::schema::RcExpr, ()> {
            match expr.as_ref() {
                Expr::Call(name, args) => {
                    if name == target {
                        Ok(expr.clone())
                    } else {
                        find_named_call(args, target)
                    }
                }
                Expr::Top(_, a, b, c) => find_named_call(a, target)
                    .or_else(|_| find_named_call(b, target))
                    .or_else(|_| find_named_call(c, target)),
                Expr::Bop(_, lhs, rhs) | Expr::Concat(lhs, rhs) => {
                    find_named_call(lhs, target).or_else(|_| find_named_call(rhs, target))
                }
                Expr::Uop(_, inner)
                | Expr::Get(inner, _)
                | Expr::Single(inner)
                | Expr::Function(_, _, _, inner) => find_named_call(inner, target),
                Expr::Alloc(_, amount, state, _) => {
                    find_named_call(amount, target).or_else(|_| find_named_call(state, target))
                }
                Expr::If(pred, input, then_branch, else_branch) => find_named_call(pred, target)
                    .or_else(|_| find_named_call(input, target))
                    .or_else(|_| find_named_call(then_branch, target))
                    .or_else(|_| find_named_call(else_branch, target)),
                Expr::Switch(pred, input, branches) => find_named_call(pred, target)
                    .or_else(|_| find_named_call(input, target))
                    .or_else(|_| {
                        branches
                            .iter()
                            .map(|branch| find_named_call(branch, target))
                            .find_map(Result::ok)
                            .ok_or(())
                    }),
                Expr::DoWhile(inputs, body) => {
                    find_named_call(inputs, target).or_else(|_| find_named_call(body, target))
                }
                Expr::Const(..) | Expr::Empty(..) | Expr::Arg(..) | Expr::Symbolic(..) => Err(()),
            }
        }

        let output_ty = tuplet!(intt(), intt(), intt(), statet());
        let helper = function("inv_helper", emptyt(), base(intt()), int(7));
        let inputs_expr = parallel!(getat(0), getat(1), getat(2), getat(3))
            .with_arg_types(output_ty.clone(), output_ty.clone());
        let invariant_arg = empty().with_arg_types(output_ty.clone(), emptyt());
        let call_inv = call("inv_helper", invariant_arg);
        let body_expr = parallel!(
            less_than(getat(0), getat(1)),
            int(3),
            call_inv.clone(),
            getat(2),
            getat(3),
        );
        let loop_expr = dowhile(inputs_expr.clone(), body_expr.clone());
        let main = function("main", output_ty.clone(), output_ty.clone(), loop_expr);
        let program = program!(main, helper);

        let typed_program = program.with_arg_types();
        let (program_with_ctx, _cache) = typed_program.add_context();
        let loop_expr = program_with_ctx
            .entry
            .func_body()
            .expect("main should have a body")
            .clone();
        let Expr::DoWhile(_inputs, body) = loop_expr.as_ref() else {
            panic!("expected loop_invariant call witness main body to be a DoWhile")
        };
        let loop_expr_with_ctx = loop_expr.to_string();
        let body_with_ctx = body.to_string();
        let call_inv_expr = find_named_call(body, "inv_helper")
            .expect("loop_invariant call witness body should contain a call to inv_helper")
            .clone();
        let Expr::Call(_, call_arg) = call_inv_expr.as_ref() else {
            panic!("named call witness should be a Call expression")
        };
        let call_inv = call_inv_expr.to_string();
        let call_arg = call_arg.to_string();

        (
            program,
            loop_expr_with_ctx,
            body_with_ctx,
            call_inv,
            call_arg,
        )
    }

    #[test]
    fn native_feature_path_matches_text_backend_for_loop_invariant_case() {
        let _guard = test_lock::lock();
        let (loop_expr, inputs, body, inv, single_inv, concat_inv, empty_inv) =
            support_witness_terms();
        let initialization = format!(
            "(let __rlcr_loop {loop_expr})\n(let __rlcr_inputs {inputs})\n(let __rlcr_body {body})\n(let __rlcr_inv {inv})\n(let __rlcr_single_inv {single_inv})\n(let __rlcr_concat_inv {concat_inv})\n(let __rlcr_empty_inv {empty_inv})"
        );
        let schedule = format!(
            "{}\n(check (is-inv-Expr __rlcr_body __rlcr_inv))\n(check (is-inv-Expr __rlcr_body __rlcr_single_inv))\n(check (is-inv-Expr __rlcr_body __rlcr_concat_inv))\n(check (is-inv-Expr __rlcr_body __rlcr_empty_inv))\n(check (= (to-hoist __rlcr_inputs __rlcr_body) __rlcr_inv))\n",
            helper_schedule()
        );

        run_text_check(&crate::prologue_egglog_text(), &initialization, &schedule);
        run_native_check(
            &crate::feature_execution_prologue(true, None),
            &initialization,
            &schedule,
            None,
        );
    }

    #[test]
    fn native_feature_path_matches_text_backend_for_loop_invariant_call_case() {
        let _guard = test_lock::lock();
        let (program, loop_expr, body, call_inv, call_arg) = call_witness_program_terms();
        let schedule = format!(
            "{}\n(check (BodyContainsExpr __rlcr_body __rlcr_call_inv))\n(check (ExprIsPure __rlcr_call_inv))\n(check (is-inv-Expr __rlcr_body __rlcr_call_arg))\n(check (is-inv-Expr __rlcr_body __rlcr_call_inv))\n",
            helper_schedule()
        );
        let (program_initialization, schedule) = build_feature_program_parts(&program, &schedule);
        let initialization = format!(
            "{program_initialization}\n(let __rlcr_loop {loop_expr})\n(let __rlcr_body {body})\n(let __rlcr_call_inv {call_inv})\n(let __rlcr_call_arg {call_arg})"
        );

        run_text_check(&crate::prologue_egglog_text(), &initialization, &schedule);
        run_native_check(
            &crate::feature_execution_prologue(true, None),
            &initialization,
            &schedule,
            None,
        );
    }

    #[test]
    fn native_feature_path_matches_text_backend_for_loop_invariant_top_case() {
        let _guard = test_lock::lock();
        let (loop_expr, inputs, body, top_inv) = top_witness_terms();
        let initialization = format!(
            "(let __rlcr_loop {loop_expr})\n(let __rlcr_inputs {inputs})\n(let __rlcr_body {body})\n(let __rlcr_top_inv {top_inv})"
        );
        let schedule = format!(
            "{}\n(check (is-inv-Expr __rlcr_body __rlcr_top_inv))\n(check (= (to-hoist __rlcr_inputs __rlcr_body) __rlcr_top_inv))\n",
            helper_schedule()
        );

        run_text_check(&crate::prologue_egglog_text(), &initialization, &schedule);
        run_native_check(
            &crate::feature_execution_prologue(true, None),
            &initialization,
            &schedule,
            None,
        );
    }

    #[test]
    fn native_feature_path_matches_text_backend_for_loop_invariant_if_case() {
        let _guard = test_lock::lock();
        let (loop_expr, inputs, body, if_inv) = if_witness_terms();
        let initialization = format!(
            "(let __rlcr_loop {loop_expr})\n(let __rlcr_inputs {inputs})\n(let __rlcr_body {body})\n(let __rlcr_if_inv {if_inv})"
        );
        let schedule = format!(
            "{}\n(check (is-inv-Expr __rlcr_body __rlcr_if_inv))\n",
            helper_schedule()
        );

        run_text_check(&crate::prologue_egglog_text(), &initialization, &schedule);
        run_native_check(
            &crate::feature_execution_prologue(true, None),
            &initialization,
            &schedule,
            None,
        );
    }

    #[test]
    fn native_feature_path_matches_text_backend_for_loop_invariant_dowhile_case() {
        let _guard = test_lock::lock();
        let (loop_expr, inputs, body, dowhile_inv) = dowhile_witness_terms();
        let initialization = format!(
            "(let __rlcr_loop {loop_expr})\n(let __rlcr_inputs {inputs})\n(let __rlcr_body {body})\n(let __rlcr_dowhile_inv {dowhile_inv})"
        );
        let schedule = format!(
            "{}\n(check (ExprIsPure __rlcr_dowhile_inv))\n(check (is-inv-Expr __rlcr_body __rlcr_dowhile_inv))\n",
            helper_schedule()
        );

        run_text_check(&crate::prologue_egglog_text(), &initialization, &schedule);
        run_native_check(
            &crate::feature_execution_prologue(true, None),
            &initialization,
            &schedule,
            None,
        );
    }
}
