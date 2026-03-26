pub(crate) fn fragment() -> String {
    let mut out = String::new();
    out.push_str(GENERATED_MARKER);
    out.push_str(HACKERS_DELIGHT);
    out.push('\n');
    out
}

#[cfg(feature = "eggplant")]
pub(crate) fn native_fragment() -> String {
    let mut out = String::new();
    out.push_str(GENERATED_MARKER);
    out.push_str(HACKERS_DELIGHT_SUPPORT);
    out.push('\n');
    out
}

const GENERATED_MARKER: &str =
    "; (Generated from eggplant Rust: src/eggplant_backend/hackers_delight.rs)\n";
const HACKERS_DELIGHT_SUPPORT: &str = r#";; Hacker's delight optimizations

;; A simple analysis to identify loops that run exactly #popcount times

;; IsIsEven e x => e is a boolean expression that checks whether x is an even number
(relation IsIsEven (Expr Expr))

;; NTZIterations lp n pos => loop lp runs exactly number_of_trailing_zeros(n) times at index pos
(relation NTZIterations (Expr Expr i64))

;; Try to do a state-edge-passthrough for loops
;; NLZIterations guarantees termination for non-zero values
;; lowbit(0) is undefined behavior

(constructor DummyLoopContext (Expr Expr Expr) Assumption)"#;
const HACKERS_DELIGHT: &str = r#";; Hacker's delight optimizations

(ruleset hacker)

;; A simple analysis to identify loops that run exactly #popcount times

;; IsIsEven e x => e is a boolean expression that checks whether x is an even number
(relation IsIsEven (Expr Expr))

(rule (
    (= two (Const (Int 2) ty ctx))
    (= e (Bop (Eq) x (Bop (Mul) (Bop (Div) x two) two)))
) (
    (IsIsEven e x)
) :ruleset hacker)

;; NTZIterations lp n pos => loop lp runs exactly number_of_trailing_zeros(n) times at index pos
(relation NTZIterations (Expr Expr i64))

(rule (
    ;; Grab the outer if
    (= outerif (If cond inputs evenbr oddbr))
    ;; There exists an argument n
    (= n (Get inputs i))
    ;; The condition is on the parity of n
    (IsIsEven cond n)
    ;; In the even/true branch, there is a loop
    (= evenbr (DoWhile lp_inputs lp_pred_outputs))
    ;; n is passed into to the loop
    (= (Get lp_inputs j) (Get (Arg _ty1 _ctx1) i))
    ;; the loop continues as long as n / 2 is even
    (= two (Const (Int 2) _ty2 _ctx2))
    (= nd2 (Bop (Div) (Get (Arg _ty3 _ctx3) j) two))
    (IsIsEven (Get lp_pred_outputs 0) nd2)
    ;; n is divided by 2 every loop
    (= nd2 (Get lp_pred_outputs (+ j 1)))
    ;; In the odd/false branch, we look for an n
    (= (Get (Arg _ty4 _ctx4) i) (Get oddbr j))
) (
    (NTZIterations outerif n j)
) :ruleset hacker)

;; Identify and optimize lowbit

(rule (
    (NTZIterations outerif n i)
    (= outerif (If cond inputs evenbr oddbr))
    ;; In the even branch, it returns a value that doubles every iter
    (= evenbr (DoWhile lp_inputs lp_pred_outputs))
    (= (Const (Int 1) _ty1 _ctx1) (Get lp_inputs j))
    (= two (Const (Int 2) _ty2 _ctx2))
    (= (Bop (Mul) (Get (Arg _ty3 _ctx3) j) two) (Get lp_pred_outputs (+ j 1)))    
    ;; In the odd branch, it returns an 1
    (= (Const (Int 1) _ty0 _ctx0) (Get oddbr j))
) (
    (let lowbitn (Bop (Bitand) n (Uop (Neg) n)))
    (union (Get outerif j) lowbitn)
    (union (Get outerif i) (Bop (Div) n lowbitn))
) :ruleset hacker)

;; Try to do a state-edge-passthrough for loops
;; NLZIterations guarantees termination for non-zero values
;; lowbit(0) is undefined behavior

(constructor DummyLoopContext (Expr Expr Expr) Assumption)

(rule (
    (NTZIterations anyif n i)
    (= anyif (If cond inputs thenbr elsebr))
    (= thenbr (DoWhile lpinputs pred_outputs))
    (= (Get pred_outputs (+ j 1)) (Get (Arg arg_ty then_ctx) j))
    (HasType (Get pred_outputs (+ j 1)) (Base (StateT)))
) (
    (let newlpinputs (TupleRemoveAt lpinputs j))
    (let newpred_outputs (TupleRemoveAt pred_outputs (+ j 1)))
    
    (let newlpctx (DummyLoopContext newlpinputs newpred_outputs pred_outputs))

    (let newbody (DropAt newlpctx j newpred_outputs))

    (union newlpctx (InLoop newlpinputs newbody))

    (let newlp (DoWhile newlpinputs newbody))
    (let oldlp (TupleRemoveAt thenbr j))

    (union newlp oldlp)

    (union (Get thenbr j) (Get lpinputs j))

) :ruleset hacker)"#;

#[cfg(feature = "eggplant")]
pub(crate) mod native {
    use super::super::schema_dsl;
    use crate::eggplant_backend::peepholes::native::PeepholeTx;
    use crate::eggplant_backend::schema_dsl::{IsIsEvenPRRuleCtx, NTZIterationsPRRuleCtx};
    use eggplant::prelude::{
        AsHandle, BaseVar, Insertable, PEq, PatRecSgl, RuleRunnerSgl, RuleSetId,
    };

    #[eggplant::pat_vars]
    struct IsEvenPat<PR: PatRecSgl> {
        x: schema_dsl::Expr,
        e: schema_dsl::Bop,
    }

    fn int_const<PR: PatRecSgl>(
        value: i64,
    ) -> (
        schema_dsl::Expr<PR, schema_dsl::ConstTy>,
        eggplant::wrap::EqConstraint<i64, i64>,
    ) {
        let int_expr = schema_dsl::Int::query();
        let value_matches = int_expr.handle_value().eq(&value);
        let const_expr = schema_dsl::Const::query(
            &int_expr,
            &schema_dsl::Type::query_leaf(),
            &schema_dsl::Assumption::query_leaf(),
        );
        (const_expr, value_matches)
    }

    fn arg_get<PR: PatRecSgl>() -> schema_dsl::Get<PR> {
        schema_dsl::Get::query(&schema_dsl::Arg::query(
            &schema_dsl::Type::query_leaf(),
            &schema_dsl::Assumption::query_leaf(),
        ))
    }

    fn state_type<PR: PatRecSgl>() -> schema_dsl::Type<PR, schema_dsl::BaseTy> {
        schema_dsl::Base::query(&schema_dsl::StateT::query())
    }

    fn is_even_pat<PR: PatRecSgl>() -> IsEvenPat<PR> {
        let x = schema_dsl::Expr::query_leaf();
        let (two, two_matches_value) = int_const(2);
        let div = schema_dsl::Bop::query(&schema_dsl::Div::query(), &x, &two);
        let mul = schema_dsl::Bop::query(&schema_dsl::Mul::query(), &div, &two);
        let e = schema_dsl::Bop::query(&schema_dsl::Eq::query(), &x, &mul);

        IsEvenPat::new(x, e).assert(two_matches_value)
    }

    #[eggplant::pat_vars]
    struct NtzIterationsPat<PR: PatRecSgl> {
        outerif: schema_dsl::If,
        n: schema_dsl::Get,
        lp_input_j: schema_dsl::Get,
        arg_i: schema_dsl::Get,
        two: schema_dsl::Const,
        cond_is_even: schema_dsl::IsIsEven,
        pred_is_even: schema_dsl::IsIsEven,
    }

    fn ntz_iterations_pat<PR: PatRecSgl>() -> NtzIterationsPat<PR> {
        let cond = schema_dsl::Expr::query_leaf();
        let inputs = schema_dsl::Expr::query_leaf();
        let evenbr = schema_dsl::Expr::query_leaf();
        let oddbr = schema_dsl::Expr::query_leaf();
        let outerif = schema_dsl::If::query(&cond, &inputs, &evenbr, &oddbr);

        let n = schema_dsl::Get::query(&inputs);
        let cond_is_even = schema_dsl::IsIsEven::query_fields(&cond, &n);

        let lp_inputs = schema_dsl::Expr::query_leaf();
        let lp_pred_outputs = schema_dsl::Expr::query_leaf();
        let even_loop = schema_dsl::DoWhile::query(&lp_inputs, &lp_pred_outputs);
        let even_branch_matches = evenbr.handle().eq(&even_loop.handle());

        let lp_input_j = schema_dsl::Get::query(&lp_inputs);
        let arg_i = arg_get();
        let same_outer_index = n.handle_index().eq(&arg_i.handle_index());
        let same_loop_input_value = lp_input_j.handle().eq(&arg_i.handle());

        let (two, two_matches_value) = int_const(2);

        let arg_j = arg_get();
        let same_loop_index = lp_input_j.handle_index().eq(&arg_j.handle_index());
        let nd2 = schema_dsl::Bop::query(&schema_dsl::Div::query(), &arg_j, &two);
        let pred0 = schema_dsl::Get::query(&lp_pred_outputs);
        let pred0_is_first = pred0.handle_index().eq(&(&0_i64).as_handle());
        let pred_is_even = schema_dsl::IsIsEven::query_fields(&pred0, &nd2);
        let pred_next = schema_dsl::Get::query(&lp_pred_outputs);
        let pred_next_matches = pred_next.handle().eq(&nd2.handle());
        let pred_next_index_matches = pred_next
            .handle_index()
            .eq(&(lp_input_j.handle_index() + (&1_i64).as_handle()));

        let odd_j = schema_dsl::Get::query(&oddbr);
        let odd_index_matches = odd_j.handle_index().eq(&lp_input_j.handle_index());
        let odd_matches_outer = odd_j.handle().eq(&arg_i.handle());

        NtzIterationsPat::new(outerif, n, lp_input_j, arg_i, two, cond_is_even, pred_is_even)
            .assert(even_branch_matches)
            .assert(same_outer_index)
            .assert(same_loop_input_value)
            .assert(same_loop_index)
            .assert(two_matches_value)
            .assert(pred0_is_first)
            .assert(pred_next_matches)
            .assert(pred_next_index_matches)
            .assert(odd_index_matches)
            .assert(odd_matches_outer)
    }

    #[eggplant::pat_vars]
    struct LowbitPat<PR: PatRecSgl> {
        n: schema_dsl::Expr,
        outer_i: schema_dsl::Get,
        outer_j: schema_dsl::Get,
        lp_inputs: schema_dsl::Expr,
        one: schema_dsl::Const,
        two: schema_dsl::Const,
        one_else: schema_dsl::Const,
        ntz: schema_dsl::NTZIterations,
    }

    fn lowbit_pat<PR: PatRecSgl>() -> LowbitPat<PR> {
        let cond = schema_dsl::Expr::query_leaf();
        let inputs = schema_dsl::Expr::query_leaf();
        let evenbr = schema_dsl::Expr::query_leaf();
        let oddbr = schema_dsl::Expr::query_leaf();
        let outerif = schema_dsl::If::query(&cond, &inputs, &evenbr, &oddbr);

        let n = schema_dsl::Expr::query_leaf();
        let outer_i = schema_dsl::Get::query(&outerif);
        let ntz_index = BaseVar::<i64, PR>::query_named("ntz_index");
        let ntz = schema_dsl::NTZIterations::query_fields(&outerif, &n, &ntz_index);
        let outer_i_index_matches = outer_i.handle_index().eq(&ntz.handle_index());

        let lp_inputs = schema_dsl::Expr::query_leaf();
        let lp_pred_outputs = schema_dsl::Expr::query_leaf();
        let even_loop = schema_dsl::DoWhile::query(&lp_inputs, &lp_pred_outputs);
        let even_branch_matches = evenbr.handle().eq(&even_loop.handle());

        let lp_input_j = schema_dsl::Get::query(&lp_inputs);
        let (one, one_matches_value) = int_const(1);
        let loop_input_is_one = lp_input_j.handle().eq(&one.handle());

        let (two, two_matches_value) = int_const(2);
        let arg_j = arg_get();
        let same_loop_index = lp_input_j.handle_index().eq(&arg_j.handle_index());
        let body_next = schema_dsl::Get::query(&lp_pred_outputs);
        let doubled = schema_dsl::Bop::query(&schema_dsl::Mul::query(), &arg_j, &two);
        let body_next_matches = body_next.handle().eq(&doubled.handle());
        let body_next_index_matches = body_next
            .handle_index()
            .eq(&(lp_input_j.handle_index() + (&1_i64).as_handle()));

        let odd_j = schema_dsl::Get::query(&oddbr);
        let odd_index_matches = odd_j.handle_index().eq(&lp_input_j.handle_index());
        let (one_else, one_else_matches_value) = int_const(1);
        let odd_is_one = odd_j.handle().eq(&one_else.handle());

        let outer_j = schema_dsl::Get::query(&outerif);
        let outer_j_index_matches = outer_j.handle_index().eq(&lp_input_j.handle_index());

        LowbitPat::new(n, outer_i, outer_j, lp_inputs, one, two, one_else, ntz)
            .assert(outer_i_index_matches)
            .assert(even_branch_matches)
            .assert(loop_input_is_one)
            .assert(one_matches_value)
            .assert(same_loop_index)
            .assert(body_next_matches)
            .assert(body_next_index_matches)
            .assert(odd_index_matches)
            .assert(odd_is_one)
            .assert(two_matches_value)
            .assert(one_else_matches_value)
            .assert(outer_j_index_matches)
    }

    #[eggplant::pat_vars]
    struct HackerLoopStateEdgePat<PR: PatRecSgl> {
        n: schema_dsl::Expr,
        thenbr: schema_dsl::Expr,
        lpinputs: schema_dsl::Expr,
        pred_outputs: schema_dsl::Expr,
        arg_j: schema_dsl::Get,
        has_state_type: schema_dsl::HasType,
        ntz: schema_dsl::NTZIterations,
    }

    fn hacker_loop_state_edge_pat<PR: PatRecSgl>() -> HackerLoopStateEdgePat<PR> {
        let cond = schema_dsl::Expr::query_leaf();
        let inputs = schema_dsl::Expr::query_leaf();
        let thenbr = schema_dsl::Expr::query_leaf();
        let elsebr = schema_dsl::Expr::query_leaf();
        let anyif = schema_dsl::If::query(&cond, &inputs, &thenbr, &elsebr);
        let n = schema_dsl::Expr::query_leaf();
        let outer_i = schema_dsl::Get::query(&anyif);
        let ntz_index = BaseVar::<i64, PR>::query_named("ntz_index");
        let ntz = schema_dsl::NTZIterations::query_fields(&anyif, &n, &ntz_index);
        let outer_i_index_matches = outer_i.handle_index().eq(&ntz.handle_index());

        let lpinputs = schema_dsl::Expr::query_leaf();
        let pred_outputs = schema_dsl::Expr::query_leaf();
        let then_loop = schema_dsl::DoWhile::query(&lpinputs, &pred_outputs);
        let then_branch_matches = thenbr.handle().eq(&then_loop.handle());

        let arg_j = arg_get();
        let pred_next = schema_dsl::Get::query(&pred_outputs);
        let pred_next_matches = pred_next.handle().eq(&arg_j.handle());
        let pred_next_index_matches = pred_next
            .handle_index()
            .eq(&(arg_j.handle_index() + (&1_i64).as_handle()));
        let has_state_type = schema_dsl::HasType::query_fields(&pred_next, &state_type::<PR>());

        HackerLoopStateEdgePat::new(n, thenbr, lpinputs, pred_outputs, arg_j, has_state_type, ntz)
            .assert(outer_i_index_matches)
            .assert(then_branch_matches)
            .assert(pred_next_matches)
            .assert(pred_next_index_matches)
    }

    pub(crate) fn register_native_rules() -> RuleSetId {
        let ruleset = PeepholeTx::new_ruleset("hacker");

        PeepholeTx::add_rule("hacker_is_even", ruleset, is_even_pat, |ctx, pat| {
            ctx.insert_is_is_even(pat.e, pat.x);
        });

        PeepholeTx::add_rule(
            "hacker_ntz_iterations",
            ruleset,
            ntz_iterations_pat,
            |ctx, pat| {
                ctx.insert_ntz_iterations(pat.outerif, pat.n, ctx.devalue(pat.lp_input_j.index));
            },
        );

        PeepholeTx::add_rule("hacker_lowbit", ruleset, lowbit_pat, |ctx, pat| {
            let neg_op =
                eggplant::wrap::Value::<schema_dsl::UnaryOp>::new((ctx).insert("Neg", &[]));
            let neg_n = eggplant::wrap::Value::<schema_dsl::Expr>::new(
                (&ctx).insert("Uop", &[neg_op.val, pat.n.to_value(&ctx).val]),
            );
            let bitand =
                eggplant::wrap::Value::<schema_dsl::BinaryOp>::new((ctx).insert("Bitand", &[]));
            let lowbit = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                "Bop",
                &[
                    bitand.val,
                    pat.n.to_value(&ctx).val,
                    neg_n.to_value(&ctx).val,
                ],
            ));
            let div = eggplant::wrap::Value::<schema_dsl::BinaryOp>::new((ctx).insert("Div", &[]));
            let ntz = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                "Bop",
                &[div.val, pat.n.to_value(&ctx).val, lowbit.to_value(&ctx).val],
            ));

            ctx.union(pat.outer_j, lowbit);
            ctx.union(pat.outer_i, ntz);
        });

        PeepholeTx::add_rule(
            "hacker_loop_state_edge",
            ruleset,
            hacker_loop_state_edge_pat,
            |ctx, pat| {
                let j = ctx.devalue(pat.arg_j.index);
                let j_val = j.to_value(&ctx).val;
                let j_plus_one = (j + 1).to_value(&ctx).val;

                let new_lp_inputs = eggplant::wrap::Value::<schema_dsl::Expr>::new(
                    (&ctx).insert("TupleRemoveAt", &[pat.lpinputs.to_value(&ctx).val, j_val]),
                );
                let new_pred_outputs =
                    eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                        "TupleRemoveAt",
                        &[pat.pred_outputs.to_value(&ctx).val, j_plus_one],
                    ));
                let new_loop_ctx =
                    eggplant::wrap::Value::<schema_dsl::Assumption>::new((&ctx).insert(
                        "DummyLoopContext",
                        &[
                            new_lp_inputs.to_value(&ctx).val,
                            new_pred_outputs.to_value(&ctx).val,
                            pat.pred_outputs.to_value(&ctx).val,
                        ],
                    ));
                let new_body = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "DropAt",
                    &[
                        new_loop_ctx.to_value(&ctx).val,
                        j_val,
                        new_pred_outputs.to_value(&ctx).val,
                    ],
                ));
                let loop_ctx = eggplant::wrap::Value::<schema_dsl::Assumption>::new((&ctx).insert(
                    "InLoop",
                    &[
                        new_lp_inputs.to_value(&ctx).val,
                        new_body.to_value(&ctx).val,
                    ],
                ));
                ctx.union(new_loop_ctx, loop_ctx);

                let new_loop = eggplant::wrap::Value::<schema_dsl::Expr>::new((&ctx).insert(
                    "DoWhile",
                    &[
                        new_lp_inputs.to_value(&ctx).val,
                        new_body.to_value(&ctx).val,
                    ],
                ));
                let old_loop = eggplant::wrap::Value::<schema_dsl::Expr>::new(
                    (&ctx).insert("TupleRemoveAt", &[pat.thenbr.to_value(&ctx).val, j_val]),
                );
                ctx.union(new_loop, old_loop);

                let then_state = eggplant::wrap::Value::<schema_dsl::Expr>::new(
                    (&ctx).insert("Get", &[pat.thenbr.to_value(&ctx).val, j_val]),
                );
                let input_state = eggplant::wrap::Value::<schema_dsl::Expr>::new(
                    (&ctx).insert("Get", &[pat.lpinputs.to_value(&ctx).val, j_val]),
                );
                ctx.union(then_state, input_state);
            },
        );

        ruleset
    }
}
