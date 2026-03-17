pub(crate) fn fragment() -> String {
    let mut out = String::new();
    out.push_str(GENERATED_MARKER);
    out.push_str(PEEPHOLES);
    out.push('\n');
    out
}

const GENERATED_MARKER: &str =
    "; (Generated from eggplant Rust: src/eggplant_backend/peepholes.rs)\n";
const PEEPHOLES: &str = r#"; Simple rewrites that don't do a ton with control flow.

(ruleset peepholes)

(rewrite (Bop (Mul) (Const (Int 0) ty ctx) e) (Const (Int 0) ty ctx) :ruleset peepholes)
(rewrite (Bop (Mul) e (Const (Int 0) ty ctx)) (Const (Int 0) ty ctx) :ruleset peepholes)
(rewrite (Bop (Mul) (Const (Int 1) ty ctx) e) e :ruleset peepholes)
(rewrite (Bop (Mul) e (Const (Int 1) ty ctx)) e :ruleset peepholes)
(rewrite (Bop (Add) (Const (Int 0) ty ctx) e) e :ruleset peepholes)
(rewrite (Bop (Add) e (Const (Int 0) ty ctx) ) e :ruleset peepholes)

(rewrite (Bop (Mul) (Const (Int j) ty ctx) (Const (Int i) ty ctx)) (Const (Int (* i j)) ty ctx) :ruleset peepholes)
(rewrite (Bop (Add) (Const (Int j) ty ctx) (Const (Int i) ty ctx)) (Const (Int (+ i j)) ty ctx) :ruleset peepholes)

(rewrite (Bop (And) (Const (Bool true) ty ctx) e) e :ruleset peepholes)
(rewrite (Bop (And) e (Const (Bool true) ty ctx)) e :ruleset peepholes)
(rewrite (Bop (And) (Const (Bool false) ty ctx) e) (Const (Bool false) ty ctx) :ruleset peepholes)
(rewrite (Bop (And) e (Const (Bool false) ty ctx)) (Const (Bool false) ty ctx) :ruleset peepholes)
(rewrite (Bop (Or) (Const (Bool false) ty ctx) e) e :ruleset peepholes)
(rewrite (Bop (Or) e (Const (Bool false) ty ctx)) e :ruleset peepholes)
(rewrite (Bop (Or) (Const (Bool true) ty ctx) e) (Const (Bool true) ty ctx) :ruleset peepholes)
(rewrite (Bop (Or) e (Const (Bool true) ty ctx)) (Const (Bool true) ty ctx) :ruleset peepholes)

(rule (
        (= expr (Bop (Sub) x x))
        (HasArgType expr ty)
        (ContextOf expr ctx)
      )
      ((union expr (Const (Int 0) ty ctx)))
      :ruleset peepholes)

; (x - y) + z => x + (z - y)
(rewrite (Bop (Add) (Bop (Sub) x y) z) (Bop (Add) x (Bop (Sub) z y)) :ruleset peepholes)

; (a + b) - c => a + (b - c)
(rewrite (Bop (Sub) (Bop (Add) a b) c) (Bop (Add) a (Bop (Sub) b c)) :ruleset peepholes)

; (a * x) + a => a * (x + 1)
(rule (
        (= expr (Bop (Add) (Bop (Mul) a x) a))
        (HasArgType expr ty)
        (ContextOf expr ctx)
      )
      ((union expr (Bop (Mul) a (Bop (Add) x (Const (Int 1) ty ctx)))))
      :ruleset peepholes)

(rewrite (Top (Select) pred x x) x :ruleset peepholes)

; constant fold `(x + const1) + const2` even when x is not constant
(rewrite (Bop (Add) (Bop (Add) x (Const (Int i) ty ctx)) (Const (Int j) ty ctx))
         (Bop (Add) x (Const (Int (+ i j)) ty ctx))
         :ruleset peepholes)

; ptradd(ptradd(p, x), y) => ptradd(p, x + y)
(rewrite (Bop (PtrAdd) (Bop (PtrAdd) p x) y)
         (Bop (PtrAdd) p (Bop (Add) x y))
         :ruleset peepholes)"#;

#[cfg(feature = "eggplant")]
pub(crate) mod native {
    use eggplant::{prelude::*, tx_rx_vt_pr};

    #[eggplant::dsl]
    pub enum PeepholeExpr {
        PIntConst {
            num: i64,
        },
        PAdd {
            l: PeepholeExpr,
            r: PeepholeExpr,
        },
        PMul {
            l: PeepholeExpr,
            r: PeepholeExpr,
        },
        PSelect {
            pred: PeepholeExpr,
            thn: PeepholeExpr,
            els: PeepholeExpr,
        },
    }

    tx_rx_vt_pr!(PeepholeTx, PeepholePatRec);

    #[allow(dead_code)]
    pub(crate) fn register_native_rules(ruleset_name: &'static str) -> RuleSetId {
        let ruleset = PeepholeTx::new_ruleset(ruleset_name);

        PeepholeTx::add_rule(
            "add_zero_lhs",
            ruleset,
            || {
                let x = PeepholeExpr::query_leaf();
                let z = PIntConst::query().num(&0);
                let add = PAdd::query(&z, &x);
                #[eggplant::pat_vars_catch]
                struct AddZeroLhsPat {
                    x: PeepholeExpr,
                    z: PIntConst,
                    add: PAdd,
                }
            },
            |ctx, pat| {
                let _ = ctx.devalue(pat.z.num);
                ctx.union(pat.add, pat.x);
            },
        );

        PeepholeTx::add_rule(
            "mul_one_rhs",
            ruleset,
            || {
                let x = PeepholeExpr::query_leaf();
                let one = PIntConst::query().num(&1);
                let mul = PMul::query(&x, &one);
                #[eggplant::pat_vars_catch]
                struct MulOneRhsPat {
                    x: PeepholeExpr,
                    one: PIntConst,
                    mul: PMul,
                }
            },
            |ctx, pat| {
                let _ = ctx.devalue(pat.one.num);
                ctx.union(pat.mul, pat.x);
            },
        );

        PeepholeTx::add_rule(
            "select_same_arms",
            ruleset,
            || {
                let pred = PeepholeExpr::query_leaf();
                let x = PeepholeExpr::query_leaf();
                let select = PSelect::query(&pred, &x, &x);
                #[eggplant::pat_vars_catch]
                struct SelectSamePat {
                    pred: PeepholeExpr,
                    x: PeepholeExpr,
                    select: PSelect,
                }
            },
            |ctx, pat| {
                let _ = pat.pred;
                ctx.union(pat.select, pat.x);
            },
        );

        ruleset
    }
}

#[cfg(all(test, feature = "eggplant"))]
mod native_tests {
    use super::native::{
        register_native_rules, PAdd, PIntConst, PMul, PSelect, PeepholeExpr, PeepholeTx,
    };
    use eggplant::prelude::{Commit, RuleRunnerSgl, RunConfig, TxSgl};

    #[test]
    fn native_peepholes_run_without_text_prologue() {
        let ruleset = register_native_rules("native_peepholes_round0");

        let arithmetic: PeepholeExpr<PeepholeTx, _> = PAdd::new(
            &PIntConst::new(0),
            &PMul::new(&PIntConst::new(7), &PIntConst::new(1)),
        );
        arithmetic.commit();
        let arithmetic_expected: PeepholeExpr<PeepholeTx, _> = PIntConst::new(7);
        arithmetic_expected.commit();

        let select_expr: PeepholeExpr<PeepholeTx, _> =
            PSelect::new(&PIntConst::new(1), &PIntConst::new(9), &PIntConst::new(9));
        select_expr.commit();
        let select_expected: PeepholeExpr<PeepholeTx, _> = PIntConst::new(9);
        select_expected.commit();

        PeepholeTx::run_ruleset(ruleset, RunConfig::Sat);

        assert_eq!(
            PeepholeTx::canonical_raw(&arithmetic),
            PeepholeTx::canonical_raw(&arithmetic_expected),
            "typed arithmetic peepholes should simplify without the text prologue",
        );
        assert_eq!(
            PeepholeTx::canonical_raw(&select_expr),
            PeepholeTx::canonical_raw(&select_expected),
            "typed select peephole should simplify without the text prologue",
        );
    }
}
