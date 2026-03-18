pub(crate) fn fragment() -> String {
    let mut out = String::new();
    out.push_str(GENERATED_MARKER);
    out.push_str(SWITCH_REWRITES);
    out.push('\n');
    out
}

const GENERATED_MARKER: &str =
    "; (Generated from eggplant Rust: src/eggplant_backend/switch_rewrites.rs)\n";
const SWITCH_REWRITES: &str = r#"(ruleset switch_rewrite)
(ruleset always-switch-rewrite)

; if a < b then a else b ~~> (min a b)
(rule (
       (= pred (Bop (LessThan) a b))
       (= if_e (If pred inputs thn els))
       ; a is an input to the if region
       (= a (Get inputs i))
       ; b is an input to the if region
       (= b (Get inputs j))
       ; if a < b then a else b
       (= (Get thn k) (Get (Arg ty (InIf true pred inputs)) i))
       (= (Get els k) (Get (Arg ty (InIf false pred inputs)) j))
      )
      ((union (Get if_e k) (Bop (Smin) a b)))
      :ruleset switch_rewrite)

; if a < b then b else a ~~> (max a b)
(rule (
       (= pred (Bop (LessThan) a b))
       (= if_e (If pred inputs thn els))
       ; a is an input to the if region
       (= a (Get inputs i))
       ; b is an input to the if region
       (= b (Get inputs j))
       ; if a < b then b else a
       (= (Get thn k) (Get (Arg ty (InIf true pred inputs)) j))
       (= (Get els k) (Get (Arg ty (InIf false pred inputs)) i))
      )
      ((union (Get if_e k) (Bop (Smax) a b)))
      :ruleset switch_rewrite) 

; if pred then a else b ~~> (select pred a b)
; where a and b are inputs to the region
(rule (
       (= if_e (If pred inputs thn els))
       (= a (Get inputs i))
       (= b (Get inputs j))

       ; if pred then a else b
       (= (Get thn k) (Get (Arg ty (InIf true pred inputs)) i))
       (= (Get els k) (Get (Arg ty (InIf false pred inputs)) j))

       ; If i = j, then the arg is just passed through the if, and we
       ; don't need a select. This will get handled by the passthrough rules.
       (!= i j)
       )
       (
       (union (Get if_e k) (Top (Select) pred a b))
       )
       :ruleset switch_rewrite)

(rule (
       (= if_e (If pred inputs thn els))
       (ContextOf if_e ctx)
       (HasArgType if_e ty)
       (= (Get thn i) (Const x _ty (InIf true pred inputs)))
       (= (Get els i) (Const y _ty (InIf false pred inputs)))
      )
      ((union (Get if_e i) (Top (Select) pred (Const x ty ctx) (Const y ty ctx))))
      :ruleset switch_rewrite)

; if pred then A else Const -> select pred A Const
; where A is an input to the region
(rule (
       (= if_e (If pred inputs thn els))
       (ContextOf if_e ctx)
       (HasArgType if_e ty)

       ; input to the if
       (= a (Get inputs i))
       (= (Get thn k) (Get (Arg _ty (InIf true pred inputs)) i))

       (= els_out (Get els k))
       (= (IntB y) (lo-bound els_out))
       (= (IntB y) (hi-bound els_out))
       )
       (
       (union (Get if_e k) (Top (Select) pred a (Const (Int y) ty ctx)))
       )
       :ruleset switch_rewrite
)

; if pred then Const else B -> select pred Const B
; where B is an input to the region
(rule (
       (= if_e (If pred inputs thn els))
       (ContextOf if_e ctx)
       (HasArgType if_e ty)

       (= thn_out (Get thn k))
       (= (IntB y) (lo-bound thn_out))
       (= (IntB y) (hi-bound thn_out))

       ; input to the if
       (= b (Get inputs i))
       (= (Get els k) (Get (Arg _ty (InIf false pred inputs)) i))
      )
      (
       (union (Get if_e k) (Top (Select) pred (Const (Int y) ty ctx) b))
      )
      :ruleset switch_rewrite
)

; if (a and b) X Y ~~> if a (if b X Y) Y
(rule ((= lhs (If (Bop (And) a b) ins X Y))
       (HasType ins (TupleT ins_ty))
       (= len (tuple-length ins))
       
       ;; For early returns, Y might be fairly large
       ;; limit the size we match on here
       (< (Expr-size Y) 100))

      ((let outer_ins (Concat (Single b) ins))
       (let outer_ins_ty (TupleT (TCons (BoolT) ins_ty)))

       (let inner_pred    (Get      (Arg outer_ins_ty (InIf true  a outer_ins)) 0))
       (let sub_arg_true  (SubTuple (Arg outer_ins_ty (InIf true  a outer_ins)) 1 len))
       (let sub_arg_false (SubTuple (Arg outer_ins_ty (InIf false a outer_ins)) 1 len))

       (let inner_Y (AddContext (InIf false inner_pred sub_arg_true) Y))
       (let outer_Y (Subst      (InIf false a          outer_ins) sub_arg_false Y))

       (let inner (If inner_pred sub_arg_true X inner_Y))
       (union lhs (If a          outer_ins    inner   outer_Y)))

       :ruleset switch_rewrite)

; if (a or b) X Y ~~> if a X (if b X Y)
(rule ((= lhs (If (Bop (Or) a b) ins X Y))
       (HasType ins (TupleT ins_ty))
       (= len (tuple-length ins))
       
       ;; limit the size, since X and Y get new contexts
       (< (Expr-size X) 100)
       (< (Expr-size Y) 100)
       )

      ((let outer_ins (Concat (Single b) ins))
       (let outer_ins_ty (TupleT (TCons (BoolT) ins_ty)))

       (let inner_pred    (Get      (Arg outer_ins_ty (InIf false a outer_ins)) 0))
       (let sub_arg_true  (SubTuple (Arg outer_ins_ty (InIf true  a outer_ins)) 1 len))
       (let sub_arg_false (SubTuple (Arg outer_ins_ty (InIf false a outer_ins)) 1 len))

       (let outer_X (Subst      (InIf true  a          outer_ins) sub_arg_true X))
       (let inner_X (AddContext (InIf true  inner_pred sub_arg_false) X))
       (let inner_Y (AddContext (InIf false inner_pred sub_arg_false) Y))

       (let inner (If inner_pred sub_arg_false inner_X inner_Y))
       (union lhs (If a          outer_ins     outer_X inner  )))

       :ruleset switch_rewrite)"#;

#[cfg(feature = "eggplant")]
pub(crate) mod native {
    use eggplant::dashmap;
    use eggplant::egglog;
    use super::super::native_rule_helpers::{
        bool_expr, call_expr, i64_expr, insert_call, node_expr, var_expr, EqCallConstraint,
        FactConstraint,
    };
    use super::super::schema_dsl;
    use crate::eggplant_backend::peepholes::native::{PeepholePatRec, PeepholeTx};
    use eggplant::prelude::{Insertable, PatRecSgl, RuleRunnerSgl, RuleSetId};

    #[eggplant::dsl]
    enum Bound {
        IntB { value: i64 },
        UnknownB {},
    }

    pub(crate) fn register_native_rules() -> RuleSetId {
        let ruleset = PeepholeTx::new_ruleset("switch_rewrite");
        let _always_ruleset = PeepholeTx::new_ruleset("always-switch-rewrite");

        PeepholeTx::add_rule(
            "switch_min",
            ruleset,
            || {
                let a = schema_dsl::Expr::query_leaf();
                let b = schema_dsl::Expr::query_leaf();
                let pred = schema_dsl::Bop::query(&schema_dsl::LessThan::query(), &a, &b);
                let inputs = schema_dsl::Expr::query_leaf();
                let thn = schema_dsl::Expr::query_leaf();
                let els = schema_dsl::Expr::query_leaf();
                let if_e = schema_dsl::If::query(&pred, &inputs, &thn, &els);
                let if_out: schema_dsl::Expr<PeepholePatRec, _> = schema_dsl::Expr::query_leaf();
                let thn_out: schema_dsl::Expr<PeepholePatRec, _> = schema_dsl::Expr::query_leaf();
                let els_out: schema_dsl::Expr<PeepholePatRec, _> = schema_dsl::Expr::query_leaf();
                let ty: schema_dsl::Type<PeepholePatRec, _> = schema_dsl::Type::query_leaf();
                let thn_ctx: schema_dsl::Assumption<PeepholePatRec, _> =
                    schema_dsl::Assumption::query_leaf();
                let els_ctx: schema_dsl::Assumption<PeepholePatRec, _> =
                    schema_dsl::Assumption::query_leaf();

                PeepholePatRec::on_new_constraint(EqCallConstraint::new(
                    node_expr(&if_out),
                    "Get",
                    vec![node_expr(&if_e), var_expr("k")],
                ));
                PeepholePatRec::on_new_constraint(EqCallConstraint::new(
                    node_expr(&a),
                    "Get",
                    vec![node_expr(&inputs), var_expr("i")],
                ));
                PeepholePatRec::on_new_constraint(EqCallConstraint::new(
                    node_expr(&b),
                    "Get",
                    vec![node_expr(&inputs), var_expr("j")],
                ));
                PeepholePatRec::on_new_constraint(EqCallConstraint::new(
                    node_expr(&thn_out),
                    "Get",
                    vec![node_expr(&thn), var_expr("k")],
                ));
                PeepholePatRec::on_new_constraint(EqCallConstraint::new(
                    node_expr(&els_out),
                    "Get",
                    vec![node_expr(&els), var_expr("k")],
                ));
                PeepholePatRec::on_new_constraint(EqCallConstraint::new(
                    node_expr(&thn_ctx),
                    "InIf",
                    vec![bool_expr(true), node_expr(&pred), node_expr(&inputs)],
                ));
                PeepholePatRec::on_new_constraint(EqCallConstraint::new(
                    node_expr(&els_ctx),
                    "InIf",
                    vec![bool_expr(false), node_expr(&pred), node_expr(&inputs)],
                ));
                PeepholePatRec::on_new_constraint(EqCallConstraint::new(
                    node_expr(&thn_out),
                    "Get",
                    vec![
                        call_expr("Arg", vec![node_expr(&ty), node_expr(&thn_ctx)]),
                        var_expr("i"),
                    ],
                ));
                PeepholePatRec::on_new_constraint(EqCallConstraint::new(
                    node_expr(&els_out),
                    "Get",
                    vec![
                        call_expr("Arg", vec![node_expr(&ty), node_expr(&els_ctx)]),
                        var_expr("j"),
                    ],
                ));

                #[eggplant::pat_vars_catch]
                struct SwitchMinPat {
                    a: schema_dsl::Expr,
                    b: schema_dsl::Expr,
                    if_out: schema_dsl::Expr,
                }
            },
            |ctx, pat| {
                let op = insert_call::<schema_dsl::BinaryOp>(ctx, "Smin", &[]);
                let rhs = insert_call::<schema_dsl::Expr>(
                    ctx,
                    "Bop",
                    &[op.0.val, pat.a.val, pat.b.val],
                );
                ctx.union(pat.if_out, rhs);
            },
        );

        PeepholeTx::add_rule(
            "switch_max",
            ruleset,
            || {
                let a = schema_dsl::Expr::query_leaf();
                let b = schema_dsl::Expr::query_leaf();
                let pred = schema_dsl::Bop::query(&schema_dsl::LessThan::query(), &a, &b);
                let inputs = schema_dsl::Expr::query_leaf();
                let thn = schema_dsl::Expr::query_leaf();
                let els = schema_dsl::Expr::query_leaf();
                let if_e = schema_dsl::If::query(&pred, &inputs, &thn, &els);
                let if_out: schema_dsl::Expr<PeepholePatRec, _> = schema_dsl::Expr::query_leaf();
                let thn_out: schema_dsl::Expr<PeepholePatRec, _> = schema_dsl::Expr::query_leaf();
                let els_out: schema_dsl::Expr<PeepholePatRec, _> = schema_dsl::Expr::query_leaf();
                let ty: schema_dsl::Type<PeepholePatRec, _> = schema_dsl::Type::query_leaf();
                let thn_ctx: schema_dsl::Assumption<PeepholePatRec, _> =
                    schema_dsl::Assumption::query_leaf();
                let els_ctx: schema_dsl::Assumption<PeepholePatRec, _> =
                    schema_dsl::Assumption::query_leaf();

                PeepholePatRec::on_new_constraint(EqCallConstraint::new(
                    node_expr(&if_out),
                    "Get",
                    vec![node_expr(&if_e), var_expr("k")],
                ));
                PeepholePatRec::on_new_constraint(EqCallConstraint::new(
                    node_expr(&a),
                    "Get",
                    vec![node_expr(&inputs), var_expr("i")],
                ));
                PeepholePatRec::on_new_constraint(EqCallConstraint::new(
                    node_expr(&b),
                    "Get",
                    vec![node_expr(&inputs), var_expr("j")],
                ));
                PeepholePatRec::on_new_constraint(EqCallConstraint::new(
                    node_expr(&thn_out),
                    "Get",
                    vec![node_expr(&thn), var_expr("k")],
                ));
                PeepholePatRec::on_new_constraint(EqCallConstraint::new(
                    node_expr(&els_out),
                    "Get",
                    vec![node_expr(&els), var_expr("k")],
                ));
                PeepholePatRec::on_new_constraint(EqCallConstraint::new(
                    node_expr(&thn_ctx),
                    "InIf",
                    vec![bool_expr(true), node_expr(&pred), node_expr(&inputs)],
                ));
                PeepholePatRec::on_new_constraint(EqCallConstraint::new(
                    node_expr(&els_ctx),
                    "InIf",
                    vec![bool_expr(false), node_expr(&pred), node_expr(&inputs)],
                ));
                PeepholePatRec::on_new_constraint(EqCallConstraint::new(
                    node_expr(&thn_out),
                    "Get",
                    vec![
                        call_expr("Arg", vec![node_expr(&ty), node_expr(&thn_ctx)]),
                        var_expr("j"),
                    ],
                ));
                PeepholePatRec::on_new_constraint(EqCallConstraint::new(
                    node_expr(&els_out),
                    "Get",
                    vec![
                        call_expr("Arg", vec![node_expr(&ty), node_expr(&els_ctx)]),
                        var_expr("i"),
                    ],
                ));

                #[eggplant::pat_vars_catch]
                struct SwitchMaxPat {
                    a: schema_dsl::Expr,
                    b: schema_dsl::Expr,
                    if_out: schema_dsl::Expr,
                }
            },
            |ctx, pat| {
                let op = insert_call::<schema_dsl::BinaryOp>(ctx, "Smax", &[]);
                let rhs = insert_call::<schema_dsl::Expr>(
                    ctx,
                    "Bop",
                    &[op.0.val, pat.a.val, pat.b.val],
                );
                ctx.union(pat.if_out, rhs);
            },
        );

        PeepholeTx::add_rule(
            "switch_select_from_if",
            ruleset,
            || {
                let pred = schema_dsl::Expr::query_leaf();
                let inputs = schema_dsl::Expr::query_leaf();
                let thn = schema_dsl::Expr::query_leaf();
                let els = schema_dsl::Expr::query_leaf();
                let if_e = schema_dsl::If::query(&pred, &inputs, &thn, &els);
                let if_out: schema_dsl::Expr<PeepholePatRec, _> = schema_dsl::Expr::query_leaf();
                let a = schema_dsl::Expr::query_leaf();
                let b = schema_dsl::Expr::query_leaf();
                let thn_out: schema_dsl::Expr<PeepholePatRec, _> = schema_dsl::Expr::query_leaf();
                let els_out: schema_dsl::Expr<PeepholePatRec, _> = schema_dsl::Expr::query_leaf();
                let ty: schema_dsl::Type<PeepholePatRec, _> = schema_dsl::Type::query_leaf();
                let thn_ctx: schema_dsl::Assumption<PeepholePatRec, _> =
                    schema_dsl::Assumption::query_leaf();
                let els_ctx: schema_dsl::Assumption<PeepholePatRec, _> =
                    schema_dsl::Assumption::query_leaf();

                PeepholePatRec::on_new_constraint(EqCallConstraint::new(
                    node_expr(&if_out),
                    "Get",
                    vec![node_expr(&if_e), var_expr("k")],
                ));
                PeepholePatRec::on_new_constraint(EqCallConstraint::new(
                    node_expr(&a),
                    "Get",
                    vec![node_expr(&inputs), var_expr("i")],
                ));
                PeepholePatRec::on_new_constraint(EqCallConstraint::new(
                    node_expr(&b),
                    "Get",
                    vec![node_expr(&inputs), var_expr("j")],
                ));
                PeepholePatRec::on_new_constraint(EqCallConstraint::new(
                    node_expr(&thn_out),
                    "Get",
                    vec![node_expr(&thn), var_expr("k")],
                ));
                PeepholePatRec::on_new_constraint(EqCallConstraint::new(
                    node_expr(&els_out),
                    "Get",
                    vec![node_expr(&els), var_expr("k")],
                ));
                PeepholePatRec::on_new_constraint(EqCallConstraint::new(
                    node_expr(&thn_ctx),
                    "InIf",
                    vec![bool_expr(true), node_expr(&pred), node_expr(&inputs)],
                ));
                PeepholePatRec::on_new_constraint(EqCallConstraint::new(
                    node_expr(&els_ctx),
                    "InIf",
                    vec![bool_expr(false), node_expr(&pred), node_expr(&inputs)],
                ));
                PeepholePatRec::on_new_constraint(EqCallConstraint::new(
                    node_expr(&thn_out),
                    "Get",
                    vec![
                        call_expr("Arg", vec![node_expr(&ty), node_expr(&thn_ctx)]),
                        var_expr("i"),
                    ],
                ));
                PeepholePatRec::on_new_constraint(EqCallConstraint::new(
                    node_expr(&els_out),
                    "Get",
                    vec![
                        call_expr("Arg", vec![node_expr(&ty), node_expr(&els_ctx)]),
                        var_expr("j"),
                    ],
                ));
                PeepholePatRec::on_new_constraint(FactConstraint::new(
                    "!=",
                    vec![var_expr("i"), var_expr("j")],
                ));

                #[eggplant::pat_vars_catch]
                struct SwitchSelectPat {
                    pred: schema_dsl::Expr,
                    a: schema_dsl::Expr,
                    b: schema_dsl::Expr,
                    if_out: schema_dsl::Expr,
                }
            },
            |ctx, pat| {
                let op = insert_call::<schema_dsl::TernaryOp>(ctx, "Select", &[]);
                let rhs = insert_call::<schema_dsl::Expr>(
                    ctx,
                    "Top",
                    &[op.0.val, pat.pred.val, pat.a.val, pat.b.val],
                );
                ctx.union(pat.if_out, rhs);
            },
        );

        PeepholeTx::add_rule(
            "switch_select_from_const_branches",
            ruleset,
            || {
                let pred = schema_dsl::Expr::query_leaf();
                let inputs = schema_dsl::Expr::query_leaf();
                let thn = schema_dsl::Expr::query_leaf();
                let els = schema_dsl::Expr::query_leaf();
                let if_e = schema_dsl::If::query(&pred, &inputs, &thn, &els);
                let if_out: schema_dsl::Expr<PeepholePatRec, _> = schema_dsl::Expr::query_leaf();
                let ctx: schema_dsl::Assumption<PeepholePatRec, _> =
                    schema_dsl::Assumption::query_leaf();
                let ty: schema_dsl::Type<PeepholePatRec, _> = schema_dsl::Type::query_leaf();
                let x: schema_dsl::Constant<PeepholePatRec, _> = schema_dsl::Constant::query_leaf();
                let y: schema_dsl::Constant<PeepholePatRec, _> = schema_dsl::Constant::query_leaf();
                let branch_ty: schema_dsl::Type<PeepholePatRec, _> = schema_dsl::Type::query_leaf();
                let thn_ctx: schema_dsl::Assumption<PeepholePatRec, _> =
                    schema_dsl::Assumption::query_leaf();
                let els_ctx: schema_dsl::Assumption<PeepholePatRec, _> =
                    schema_dsl::Assumption::query_leaf();
                let thn_const = schema_dsl::Const::query(&x, &branch_ty, &thn_ctx);
                let els_const = schema_dsl::Const::query(&y, &branch_ty, &els_ctx);

                PeepholePatRec::on_new_constraint(FactConstraint::new(
                    "ContextOf",
                    vec![node_expr(&if_e), node_expr(&ctx)],
                ));
                PeepholePatRec::on_new_constraint(FactConstraint::new(
                    "HasArgType",
                    vec![node_expr(&if_e), node_expr(&ty)],
                ));
                PeepholePatRec::on_new_constraint(EqCallConstraint::new(
                    node_expr(&if_out),
                    "Get",
                    vec![node_expr(&if_e), var_expr("i")],
                ));
                PeepholePatRec::on_new_constraint(EqCallConstraint::new(
                    node_expr(&thn_const),
                    "Get",
                    vec![node_expr(&thn), var_expr("i")],
                ));
                PeepholePatRec::on_new_constraint(EqCallConstraint::new(
                    node_expr(&els_const),
                    "Get",
                    vec![node_expr(&els), var_expr("i")],
                ));
                PeepholePatRec::on_new_constraint(EqCallConstraint::new(
                    node_expr(&thn_ctx),
                    "InIf",
                    vec![bool_expr(true), node_expr(&pred), node_expr(&inputs)],
                ));
                PeepholePatRec::on_new_constraint(EqCallConstraint::new(
                    node_expr(&els_ctx),
                    "InIf",
                    vec![bool_expr(false), node_expr(&pred), node_expr(&inputs)],
                ));

                #[eggplant::pat_vars_catch]
                struct SwitchSelectConstPat {
                    pred: schema_dsl::Expr,
                    ctx: schema_dsl::Assumption,
                    ty: schema_dsl::Type,
                    x: schema_dsl::Constant,
                    y: schema_dsl::Constant,
                    if_out: schema_dsl::Expr,
                }
            },
            |ctx, pat| {
                let lhs = insert_call::<schema_dsl::Expr>(
                    ctx,
                    "Const",
                    &[pat.x.val, pat.ty.val, pat.ctx.val],
                );
                let rhs = insert_call::<schema_dsl::Expr>(
                    ctx,
                    "Const",
                    &[pat.y.val, pat.ty.val, pat.ctx.val],
                );
                let op = insert_call::<schema_dsl::TernaryOp>(ctx, "Select", &[]);
                let result = insert_call::<schema_dsl::Expr>(
                    ctx,
                    "Top",
                    &[op.0.val, pat.pred.val, lhs.0.val, rhs.0.val],
                );
                ctx.union(pat.if_out, result);
            },
        );

        PeepholeTx::add_rule(
            "switch_select_with_else_const",
            ruleset,
            || {
                let pred = schema_dsl::Expr::query_leaf();
                let inputs = schema_dsl::Expr::query_leaf();
                let thn = schema_dsl::Expr::query_leaf();
                let els = schema_dsl::Expr::query_leaf();
                let if_e = schema_dsl::If::query(&pred, &inputs, &thn, &els);
                let if_out: schema_dsl::Expr<PeepholePatRec, _> = schema_dsl::Expr::query_leaf();
                let a = schema_dsl::Expr::query_leaf();
                let ctx: schema_dsl::Assumption<PeepholePatRec, _> =
                    schema_dsl::Assumption::query_leaf();
                let ty: schema_dsl::Type<PeepholePatRec, _> = schema_dsl::Type::query_leaf();
                let thn_out: schema_dsl::Expr<PeepholePatRec, _> = schema_dsl::Expr::query_leaf();
                let els_out: schema_dsl::Expr<PeepholePatRec, _> = schema_dsl::Expr::query_leaf();
                let thn_ctx: schema_dsl::Assumption<PeepholePatRec, _> =
                    schema_dsl::Assumption::query_leaf();
                let y = IntB::query();

                PeepholePatRec::on_new_constraint(FactConstraint::new(
                    "ContextOf",
                    vec![node_expr(&if_e), node_expr(&ctx)],
                ));
                PeepholePatRec::on_new_constraint(FactConstraint::new(
                    "HasArgType",
                    vec![node_expr(&if_e), node_expr(&ty)],
                ));
                PeepholePatRec::on_new_constraint(EqCallConstraint::new(
                    node_expr(&if_out),
                    "Get",
                    vec![node_expr(&if_e), var_expr("k")],
                ));
                PeepholePatRec::on_new_constraint(EqCallConstraint::new(
                    node_expr(&a),
                    "Get",
                    vec![node_expr(&inputs), var_expr("i")],
                ));
                PeepholePatRec::on_new_constraint(EqCallConstraint::new(
                    node_expr(&thn_out),
                    "Get",
                    vec![node_expr(&thn), var_expr("k")],
                ));
                PeepholePatRec::on_new_constraint(EqCallConstraint::new(
                    node_expr(&thn_ctx),
                    "InIf",
                    vec![bool_expr(true), node_expr(&pred), node_expr(&inputs)],
                ));
                PeepholePatRec::on_new_constraint(EqCallConstraint::new(
                    node_expr(&thn_out),
                    "Get",
                    vec![
                        call_expr("Arg", vec![var_expr("_ty"), node_expr(&thn_ctx)]),
                        var_expr("i"),
                    ],
                ));
                PeepholePatRec::on_new_constraint(EqCallConstraint::new(
                    node_expr(&els_out),
                    "Get",
                    vec![node_expr(&els), var_expr("k")],
                ));
                PeepholePatRec::on_new_constraint(EqCallConstraint::new(
                    node_expr(&y),
                    "lo-bound",
                    vec![node_expr(&els_out)],
                ));
                PeepholePatRec::on_new_constraint(EqCallConstraint::new(
                    node_expr(&y),
                    "hi-bound",
                    vec![node_expr(&els_out)],
                ));

                #[eggplant::pat_vars_catch]
                struct SwitchSelectElseConstPat {
                    pred: schema_dsl::Expr,
                    a: schema_dsl::Expr,
                    ctx: schema_dsl::Assumption,
                    ty: schema_dsl::Type,
                    y: IntB,
                    if_out: schema_dsl::Expr,
                }
            },
            |ctx, pat| {
                let constant =
                    insert_call::<schema_dsl::Constant>(ctx, "Int", &[pat.y.value.val]);
                let rhs_const = insert_call::<schema_dsl::Expr>(
                    ctx,
                    "Const",
                    &[constant.0.val, pat.ty.val, pat.ctx.val],
                );
                let op = insert_call::<schema_dsl::TernaryOp>(ctx, "Select", &[]);
                let rhs = insert_call::<schema_dsl::Expr>(
                    ctx,
                    "Top",
                    &[op.0.val, pat.pred.val, pat.a.val, rhs_const.0.val],
                );
                ctx.union(pat.if_out, rhs);
            },
        );

        PeepholeTx::add_rule(
            "switch_select_with_then_const",
            ruleset,
            || {
                let pred = schema_dsl::Expr::query_leaf();
                let inputs = schema_dsl::Expr::query_leaf();
                let thn = schema_dsl::Expr::query_leaf();
                let els = schema_dsl::Expr::query_leaf();
                let if_e = schema_dsl::If::query(&pred, &inputs, &thn, &els);
                let if_out: schema_dsl::Expr<PeepholePatRec, _> = schema_dsl::Expr::query_leaf();
                let b = schema_dsl::Expr::query_leaf();
                let ctx: schema_dsl::Assumption<PeepholePatRec, _> =
                    schema_dsl::Assumption::query_leaf();
                let ty: schema_dsl::Type<PeepholePatRec, _> = schema_dsl::Type::query_leaf();
                let thn_out: schema_dsl::Expr<PeepholePatRec, _> = schema_dsl::Expr::query_leaf();
                let els_out: schema_dsl::Expr<PeepholePatRec, _> = schema_dsl::Expr::query_leaf();
                let els_ctx: schema_dsl::Assumption<PeepholePatRec, _> =
                    schema_dsl::Assumption::query_leaf();
                let y = IntB::query();

                PeepholePatRec::on_new_constraint(FactConstraint::new(
                    "ContextOf",
                    vec![node_expr(&if_e), node_expr(&ctx)],
                ));
                PeepholePatRec::on_new_constraint(FactConstraint::new(
                    "HasArgType",
                    vec![node_expr(&if_e), node_expr(&ty)],
                ));
                PeepholePatRec::on_new_constraint(EqCallConstraint::new(
                    node_expr(&if_out),
                    "Get",
                    vec![node_expr(&if_e), var_expr("k")],
                ));
                PeepholePatRec::on_new_constraint(EqCallConstraint::new(
                    node_expr(&thn_out),
                    "Get",
                    vec![node_expr(&thn), var_expr("k")],
                ));
                PeepholePatRec::on_new_constraint(EqCallConstraint::new(
                    node_expr(&y),
                    "lo-bound",
                    vec![node_expr(&thn_out)],
                ));
                PeepholePatRec::on_new_constraint(EqCallConstraint::new(
                    node_expr(&y),
                    "hi-bound",
                    vec![node_expr(&thn_out)],
                ));
                PeepholePatRec::on_new_constraint(EqCallConstraint::new(
                    node_expr(&b),
                    "Get",
                    vec![node_expr(&inputs), var_expr("i")],
                ));
                PeepholePatRec::on_new_constraint(EqCallConstraint::new(
                    node_expr(&els_ctx),
                    "InIf",
                    vec![bool_expr(false), node_expr(&pred), node_expr(&inputs)],
                ));
                PeepholePatRec::on_new_constraint(EqCallConstraint::new(
                    node_expr(&els_out),
                    "Get",
                    vec![node_expr(&els), var_expr("k")],
                ));
                PeepholePatRec::on_new_constraint(EqCallConstraint::new(
                    node_expr(&els_out),
                    "Get",
                    vec![
                        call_expr("Arg", vec![var_expr("_ty"), node_expr(&els_ctx)]),
                        var_expr("i"),
                    ],
                ));

                #[eggplant::pat_vars_catch]
                struct SwitchSelectThenConstPat {
                    pred: schema_dsl::Expr,
                    b: schema_dsl::Expr,
                    ctx: schema_dsl::Assumption,
                    ty: schema_dsl::Type,
                    y: IntB,
                    if_out: schema_dsl::Expr,
                }
            },
            |ctx, pat| {
                let constant =
                    insert_call::<schema_dsl::Constant>(ctx, "Int", &[pat.y.value.val]);
                let lhs_const = insert_call::<schema_dsl::Expr>(
                    ctx,
                    "Const",
                    &[constant.0.val, pat.ty.val, pat.ctx.val],
                );
                let op = insert_call::<schema_dsl::TernaryOp>(ctx, "Select", &[]);
                let rhs = insert_call::<schema_dsl::Expr>(
                    ctx,
                    "Top",
                    &[op.0.val, pat.pred.val, lhs_const.0.val, pat.b.val],
                );
                ctx.union(pat.if_out, rhs);
            },
        );

        PeepholeTx::add_rule(
            "switch_and_reassociate",
            ruleset,
            || {
                let a = schema_dsl::Expr::query_leaf();
                let b = schema_dsl::Expr::query_leaf();
                let ins = schema_dsl::Expr::query_leaf();
                let x = schema_dsl::Expr::query_leaf();
                let y = schema_dsl::Expr::query_leaf();
                let lhs = schema_dsl::If::query(
                    &schema_dsl::Bop::query(&schema_dsl::And::query(), &a, &b),
                    &ins,
                    &x,
                    &y,
                );
                let ins_ty: schema_dsl::TypeList<PeepholePatRec, _> =
                    schema_dsl::TypeList::query_leaf();

                PeepholePatRec::on_new_constraint(FactConstraint::new(
                    "HasType",
                    vec![node_expr(&ins), call_expr("TupleT", vec![node_expr(&ins_ty)])],
                ));
                PeepholePatRec::on_new_constraint(EqCallConstraint::new(
                    var_expr("len"),
                    "tuple-length",
                    vec![node_expr(&ins)],
                ));
                PeepholePatRec::on_new_constraint(FactConstraint::new(
                    "<",
                    vec![call_expr("Expr-size", vec![node_expr(&y)]), i64_expr(100)],
                ));

                #[eggplant::pat_vars_catch]
                struct SwitchAndPat {
                    lhs: schema_dsl::If,
                    a: schema_dsl::Expr,
                    b: schema_dsl::Expr,
                    ins: schema_dsl::Expr,
                    x: schema_dsl::Expr,
                    y: schema_dsl::Expr,
                    ins_ty: schema_dsl::TypeList,
                }
            },
            |ctx, pat| {
                let len = insert_call::<i64>(ctx, "tuple-length", &[pat.ins.val]);
                let single_b = insert_call::<schema_dsl::Expr>(ctx, "Single", &[pat.b.val]);
                let outer_ins = insert_call::<schema_dsl::Expr>(
                    ctx,
                    "Concat",
                    &[single_b.0.val, pat.ins.val],
                );
                let bool_ty = insert_call::<schema_dsl::BaseType>(ctx, "BoolT", &[]);
                let outer_ins_ty_list = insert_call::<schema_dsl::TypeList>(
                    ctx,
                    "TCons",
                    &[bool_ty.0.val, pat.ins_ty.val],
                );
                let outer_ins_ty = insert_call::<schema_dsl::Type>(
                    ctx,
                    "TupleT",
                    &[outer_ins_ty_list.0.val],
                );
                let if_true = insert_call::<schema_dsl::Assumption>(
                    ctx,
                    "InIf",
                    &[true.to_value(ctx).val, pat.a.val, outer_ins.0.val],
                );
                let if_false = insert_call::<schema_dsl::Assumption>(
                    ctx,
                    "InIf",
                    &[false.to_value(ctx).val, pat.a.val, outer_ins.0.val],
                );
                let arg_true = insert_call::<schema_dsl::Expr>(
                    ctx,
                    "Arg",
                    &[outer_ins_ty.0.val, if_true.0.val],
                );
                let arg_false = insert_call::<schema_dsl::Expr>(
                    ctx,
                    "Arg",
                    &[outer_ins_ty.0.val, if_false.0.val],
                );
                let inner_pred = insert_call::<schema_dsl::Expr>(
                    ctx,
                    "Get",
                    &[arg_true.0.val, 0_i64.to_value(ctx).val],
                );
                let sub_arg_true = insert_call::<schema_dsl::Expr>(
                    ctx,
                    "SubTuple",
                    &[arg_true.0.val, 1_i64.to_value(ctx).val, len.0.val],
                );
                let sub_arg_false = insert_call::<schema_dsl::Expr>(
                    ctx,
                    "SubTuple",
                    &[arg_false.0.val, 1_i64.to_value(ctx).val, len.0.val],
                );
                let inner_false_ctx = insert_call::<schema_dsl::Assumption>(
                    ctx,
                    "InIf",
                    &[false.to_value(ctx).val, inner_pred.0.val, sub_arg_true.0.val],
                );
                let inner_y = insert_call::<schema_dsl::Expr>(
                    ctx,
                    "AddContext",
                    &[inner_false_ctx.0.val, pat.y.val],
                );
                let outer_y = insert_call::<schema_dsl::Expr>(
                    ctx,
                    "Subst",
                    &[if_false.0.val, sub_arg_false.0.val, pat.y.val],
                );
                let inner = insert_call::<schema_dsl::Expr>(
                    ctx,
                    "If",
                    &[inner_pred.0.val, sub_arg_true.0.val, pat.x.val, inner_y.0.val],
                );
                let outer = insert_call::<schema_dsl::Expr>(
                    ctx,
                    "If",
                    &[pat.a.val, outer_ins.0.val, inner.0.val, outer_y.0.val],
                );
                ctx.union(pat.lhs, outer);
            },
        );

        PeepholeTx::add_rule(
            "switch_or_reassociate",
            ruleset,
            || {
                let a = schema_dsl::Expr::query_leaf();
                let b = schema_dsl::Expr::query_leaf();
                let ins = schema_dsl::Expr::query_leaf();
                let x = schema_dsl::Expr::query_leaf();
                let y = schema_dsl::Expr::query_leaf();
                let lhs = schema_dsl::If::query(
                    &schema_dsl::Bop::query(&schema_dsl::Or::query(), &a, &b),
                    &ins,
                    &x,
                    &y,
                );
                let ins_ty: schema_dsl::TypeList<PeepholePatRec, _> =
                    schema_dsl::TypeList::query_leaf();

                PeepholePatRec::on_new_constraint(FactConstraint::new(
                    "HasType",
                    vec![node_expr(&ins), call_expr("TupleT", vec![node_expr(&ins_ty)])],
                ));
                PeepholePatRec::on_new_constraint(EqCallConstraint::new(
                    var_expr("len"),
                    "tuple-length",
                    vec![node_expr(&ins)],
                ));
                PeepholePatRec::on_new_constraint(FactConstraint::new(
                    "<",
                    vec![call_expr("Expr-size", vec![node_expr(&x)]), i64_expr(100)],
                ));
                PeepholePatRec::on_new_constraint(FactConstraint::new(
                    "<",
                    vec![call_expr("Expr-size", vec![node_expr(&y)]), i64_expr(100)],
                ));

                #[eggplant::pat_vars_catch]
                struct SwitchOrPat {
                    lhs: schema_dsl::If,
                    a: schema_dsl::Expr,
                    b: schema_dsl::Expr,
                    ins: schema_dsl::Expr,
                    x: schema_dsl::Expr,
                    y: schema_dsl::Expr,
                    ins_ty: schema_dsl::TypeList,
                }
            },
            |ctx, pat| {
                let len = insert_call::<i64>(ctx, "tuple-length", &[pat.ins.val]);
                let single_b = insert_call::<schema_dsl::Expr>(ctx, "Single", &[pat.b.val]);
                let outer_ins = insert_call::<schema_dsl::Expr>(
                    ctx,
                    "Concat",
                    &[single_b.0.val, pat.ins.val],
                );
                let bool_ty = insert_call::<schema_dsl::BaseType>(ctx, "BoolT", &[]);
                let outer_ins_ty_list = insert_call::<schema_dsl::TypeList>(
                    ctx,
                    "TCons",
                    &[bool_ty.0.val, pat.ins_ty.val],
                );
                let outer_ins_ty = insert_call::<schema_dsl::Type>(
                    ctx,
                    "TupleT",
                    &[outer_ins_ty_list.0.val],
                );
                let if_true = insert_call::<schema_dsl::Assumption>(
                    ctx,
                    "InIf",
                    &[true.to_value(ctx).val, pat.a.val, outer_ins.0.val],
                );
                let if_false = insert_call::<schema_dsl::Assumption>(
                    ctx,
                    "InIf",
                    &[false.to_value(ctx).val, pat.a.val, outer_ins.0.val],
                );
                let arg_true = insert_call::<schema_dsl::Expr>(
                    ctx,
                    "Arg",
                    &[outer_ins_ty.0.val, if_true.0.val],
                );
                let arg_false = insert_call::<schema_dsl::Expr>(
                    ctx,
                    "Arg",
                    &[outer_ins_ty.0.val, if_false.0.val],
                );
                let inner_pred = insert_call::<schema_dsl::Expr>(
                    ctx,
                    "Get",
                    &[arg_false.0.val, 0_i64.to_value(ctx).val],
                );
                let sub_arg_true = insert_call::<schema_dsl::Expr>(
                    ctx,
                    "SubTuple",
                    &[arg_true.0.val, 1_i64.to_value(ctx).val, len.0.val],
                );
                let sub_arg_false = insert_call::<schema_dsl::Expr>(
                    ctx,
                    "SubTuple",
                    &[arg_false.0.val, 1_i64.to_value(ctx).val, len.0.val],
                );
                let outer_x = insert_call::<schema_dsl::Expr>(
                    ctx,
                    "Subst",
                    &[if_true.0.val, sub_arg_true.0.val, pat.x.val],
                );
                let inner_true_ctx = insert_call::<schema_dsl::Assumption>(
                    ctx,
                    "InIf",
                    &[true.to_value(ctx).val, inner_pred.0.val, sub_arg_false.0.val],
                );
                let inner_false_ctx = insert_call::<schema_dsl::Assumption>(
                    ctx,
                    "InIf",
                    &[false.to_value(ctx).val, inner_pred.0.val, sub_arg_false.0.val],
                );
                let inner_x = insert_call::<schema_dsl::Expr>(
                    ctx,
                    "AddContext",
                    &[inner_true_ctx.0.val, pat.x.val],
                );
                let inner_y = insert_call::<schema_dsl::Expr>(
                    ctx,
                    "AddContext",
                    &[inner_false_ctx.0.val, pat.y.val],
                );
                let inner = insert_call::<schema_dsl::Expr>(
                    ctx,
                    "If",
                    &[
                        inner_pred.0.val,
                        sub_arg_false.0.val,
                        inner_x.0.val,
                        inner_y.0.val,
                    ],
                );
                let outer = insert_call::<schema_dsl::Expr>(
                    ctx,
                    "If",
                    &[pat.a.val, outer_ins.0.val, outer_x.0.val, inner.0.val],
                );
                ctx.union(pat.lhs, outer);
            },
        );

        ruleset
    }
}

#[cfg(all(test, feature = "eggplant"))]
mod native_tests {
    use crate::eggplant_backend::test_lock;
    use egglog::{ast::Expr as EgglogExpr, EGraph, TermDag};

    fn eval_and_extract_expr(prologue: &str, expr: &str, schedule: &str) -> String {
        let binding = "__switch_rewrite_expr";
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
        let binding = "__switch_rewrite_native_expr";
        let initialization = format!("(let {binding} {expr})");
        use eggplant::egglog::ast::Expr as NativeEgglogExpr;

        crate::with_native_rules_egraph(prologue, &initialization, schedule, ablate, |egraph| {
            let (sort, value) = egraph.eval_expr(&NativeEgglogExpr::Var(
                eggplant::egglog::ast::Span::Panic,
                binding.into(),
            ))?;
            let (termdag, extracted, _) = egraph.extract_value(&sort, value)?;
            Ok(termdag.to_string(&extracted))
        })
        .unwrap()
    }

    #[test]
    fn native_feature_path_matches_text_backend_for_switch_min_case() {
        let _guard = test_lock::lock();

        let tuple_ty = "(TupleT (TCons (IntT) (TCons (IntT) (TNil))))";
        let left = "(Const (Int 3) (Base (IntT)) (InFunc \"RLCR\"))";
        let right = "(Const (Int 5) (Base (IntT)) (InFunc \"RLCR\"))";
        let pred = format!("(Bop (LessThan) {left} {right})");
        let inputs = format!("(Concat (Single {left}) (Single {right}))");
        let then_branch =
            format!("(Single (Get (Arg {tuple_ty} (InIf true {pred} {inputs})) 0))");
        let else_branch =
            format!("(Single (Get (Arg {tuple_ty} (InIf false {pred} {inputs})) 1))");
        let expr = format!("(Get (If {pred} {inputs} {then_branch} {else_branch}) 0)");
        let schedule = format!(
            "(run-schedule\n{}\n(saturate switch_rewrite)\n)",
            crate::schedule::types_and_indexing()
        );

        let text_extracted =
            eval_and_extract_expr(&crate::prologue_egglog_text(), &expr, &schedule);
        let native_extracted = eval_and_extract_native_expr(
            &crate::feature_execution_prologue(true, None),
            &expr,
            &schedule,
            None,
        );

        assert_eq!(native_extracted, text_extracted);
    }

    #[test]
    fn ablating_switch_rewrite_changes_feature_native_result() {
        let _guard = test_lock::lock();

        let tuple_ty = "(TupleT (TCons (IntT) (TCons (IntT) (TNil))))";
        let left = "(Const (Int 3) (Base (IntT)) (InFunc \"RLCR\"))";
        let right = "(Const (Int 5) (Base (IntT)) (InFunc \"RLCR\"))";
        let pred = format!("(Bop (LessThan) {left} {right})");
        let inputs = format!("(Concat (Single {left}) (Single {right}))");
        let then_branch =
            format!("(Single (Get (Arg {tuple_ty} (InIf true {pred} {inputs})) 0))");
        let else_branch =
            format!("(Single (Get (Arg {tuple_ty} (InIf false {pred} {inputs})) 1))");
        let expr = format!("(Get (If {pred} {inputs} {then_branch} {else_branch}) 0)");
        let schedule = format!(
            "(run-schedule\n{}\n(saturate switch_rewrite)\n)",
            crate::schedule::types_and_indexing()
        );

        let simplified = eval_and_extract_native_expr(
            &crate::feature_execution_prologue(true, None),
            &expr,
            &schedule,
            None,
        );
        let ablated = eval_and_extract_native_expr(
            &crate::feature_execution_prologue(true, Some("switch_rewrite")),
            &expr,
            &crate::ablate_schedule(&schedule, "switch_rewrite"),
            Some("switch_rewrite"),
        );

        assert_ne!(ablated, simplified);
    }
}
