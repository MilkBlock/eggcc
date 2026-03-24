pub(crate) fn fragment() -> String {
    let mut out = String::new();
    out.push_str(GENERATED_MARKER);
    out.push_str(MEM_SIMPLE);
    out.push('\n');
    out
}

#[cfg(feature = "eggplant")]
pub(crate) fn native_fragment() -> String {
    let (support_only, _) = MEM_SIMPLE
        .split_once("\n; A write then a load to different addresses can be swapped\n")
        .expect("mem_simple::native_fragment() expects the generated optimization marker");
    let mut out = String::new();
    out.push_str(GENERATED_MARKER);
    out.push_str(support_only);
    out.push('\n');
    out
}

const GENERATED_MARKER: &str =
    "; (Generated from eggplant Rust: src/eggplant_backend/mem_simple.rs)\n";
const MEM_SIMPLE: &str = r#"
(ruleset mem-simple)

; ============================
; NoAlias analysis
; ============================

(relation NoAlias (Expr Expr))

; Push NoAlias into if
(rule ((= if (If pred inputs thn els))
       (= (Get inputs i) inputs-i)
       (= (Get inputs j) inputs-j)
       (NoAlias inputs-i inputs-j)
       (= arg-i (Get (Arg ty (InIf which pred inputs)) i))
       (= arg-j (Get (Arg ty (InIf which pred inputs)) j)))
      ((NoAlias arg-i arg-j))
      :ruleset mem-simple)

(rule ((Bop (PtrAdd) e i)
       (= (lo_bound i) (IntB lo))
       (> lo 0))
      ((NoAlias e (Bop (PtrAdd) e i)))
      :ruleset mem-simple)

(rule ((Bop (PtrAdd) e i)
       (= (hi_bound i) (IntB hi))
       (< hi 0))
      ((NoAlias e (Bop (PtrAdd) e i)))
      :ruleset mem-simple)

(rule ((= p1 (Bop (PtrAdd) p i))
       (= p2 (Bop (PtrAdd) p (Bop (Add) i diff)))
       (= (lo_bound diff) (IntB lo))
       (> lo 0))
      ((NoAlias p1 p2))
      :ruleset mem-simple)

(rule ((= p1 (Bop (PtrAdd) p i))
       (= p2 (Bop (PtrAdd) p (Bop (Add) i diff)))
       (= (hi_bound diff) (IntB hi))
       (< hi 0))
      ((NoAlias p1 p2))
      :ruleset mem-simple)

(rule ((= p1 (Bop (PtrAdd) p i))
       (= p2 (Bop (PtrAdd) p (Bop (Sub) i diff)))
       (= (lo_bound diff) (IntB lo))
       (> lo 0))
      ((NoAlias p1 p2))
      :ruleset mem-simple)

(rule ((= p1 (Bop (PtrAdd) p i))
       (= p2 (Bop (PtrAdd) p (Bop (Sub) i diff)))
       (= (hi_bound diff) (IntB hi))
       (< hi 0))
      ((NoAlias p1 p2))
      :ruleset mem-simple)

(rule ((NoAlias x y))
      ((NoAlias y x))
      :ruleset mem-simple)

; ============================
; Memory optimizations
; ============================

(relation DidMemOptimization (String))

; A write then a load to different addresses can be swapped
(rule ((NoAlias write-addr load-addr)
       (= write (Top (Write) write-addr write-val state))
       (= load (Bop (Load) load-addr write)))
      ((let new-load (Bop (Load) load-addr state))
       (union
          (Get load 1)
          (Top (Write) write-addr write-val (Get new-load 1)))
       (union (Get load 0) (Get new-load 0))
       (DidMemOptimization "commute write then load")
      )
      :ruleset mem-simple)

; A load then a write to different addresses can be swapped
; Actually, does this break WeaklyLinear if the stored value depends on the
; loaded value? Commenting this out for now.
; (rule ((NoAlias load-addr write-addr)
;        (= load (Bop (Load) load-addr state))
;        (= write (Top (Write) write-addr write-val (Get load 1))))
;       ((let new-write (Top (Write) write-addr write-val state))
;        (let new-load (Bop (Load) load-addr new-write))
;        (union write (Get new-load 1))
;        (union (Get load 0) (Get new-load 0))
;        (DidMemOptimization "commute load then write")
;        )
;       :ruleset mem-simple)

; Two loads to the same address can be compressed
(rule ((= first-load (Bop (Load) addr state))
       (= second-load (Bop (Load) addr first-load)))
      ((union (Get first-load 0) (Get second-load 0))
       (union (Get first-load 1) (Get second-load 1))
       (DidMemOptimization "duplicate load")
       )
      :ruleset mem-simple)

; A write and a load to the same address can be forwarded
(rule ((= write (Top (Write) addr write-val state))
       (= load (Bop (Load) addr write)))
      ((union (Get load 0) write-val)
       (union (Get load 1) write)
       (DidMemOptimization "store forward")
       )
      :ruleset mem-simple)

; Two writes of the same value to the same address can be compressed
(rule ((= first-write (Top (Write) addr write-val state))
       (= second-write (Top (Write) addr write-val first-write)))
      ((union first-write second-write)
       (DidMemOptimization "duplicate write"))
      :ruleset mem-simple)

; A write shadows a previous write to the same address
(rule ((= first-write (Top (Write) addr shadowed-val state))
       (= second-write (Top (Write) addr write-val first-write)))
      ((union second-write (Top (Write) addr write-val state))
       (DidMemOptimization "shadowed write"))
      :ruleset mem-simple)

; A load doesn't change the state
; TODO: why does this break weaklylinear?
; (rule ((= load (Bop (Load) addr state)))
;       ((union (Get load 1) state))
;       :ruleset mem-simple)

; (rule ((DidMemOptimization _))
;       ((panic "DidMemOptimization"))
;       :ruleset mem-simple)"#;

#[cfg(feature = "eggplant")]
pub(crate) mod native {
    use super::super::schema_dsl;
    use crate::eggplant_backend::peepholes::native::PeepholeTx;
    use crate::eggplant_backend::schema_dsl::{BinaryOpRuleCtx, ExprRuleCtx, TernaryOpRuleCtx};
    use eggplant::prelude::{IntoHandleTy, PatRecSgl, RuleRunnerSgl, RuleSetId};

    #[eggplant::pat_vars]
    struct CommuteWriteLoadPat<PR: PatRecSgl> {
        write_addr: schema_dsl::Expr,
        load_addr: schema_dsl::Expr,
        write_val: schema_dsl::Expr,
        state: schema_dsl::Expr,
        write: schema_dsl::Top,
        load: schema_dsl::Bop,
    }

    fn commute_write_load_pat<PR: PatRecSgl>() -> CommuteWriteLoadPat<PR> {
        let write_addr = schema_dsl::Expr::query_leaf();
        let load_addr = schema_dsl::Expr::query_leaf();
        let write_val = schema_dsl::Expr::query_leaf();
        let state = schema_dsl::Expr::query_leaf();
        let write =
            schema_dsl::Top::query(&schema_dsl::Write::query(), &write_addr, &write_val, &state);
        let load = schema_dsl::Bop::query(&schema_dsl::Load::query(), &load_addr, &write);
        let no_alias = eggplant::wrap::FactCallConstraint {
            op: "NoAlias",
            operands: vec![
                write_addr.handle().into_handle_ty(),
                load_addr.handle().into_handle_ty(),
            ],
        };

        CommuteWriteLoadPat::new(write_addr, load_addr, write_val, state, write, load)
            .assert(no_alias)
    }

    #[eggplant::pat_vars]
    struct DuplicateLoadPat<PR: PatRecSgl> {
        first_load: schema_dsl::Bop,
        second_load: schema_dsl::Bop,
    }

    fn duplicate_load_pat<PR: PatRecSgl>() -> DuplicateLoadPat<PR> {
        let addr = schema_dsl::Expr::query_leaf();
        let state = schema_dsl::Expr::query_leaf();
        let first_load = schema_dsl::Bop::query(&schema_dsl::Load::query(), &addr, &state);
        let second_load = schema_dsl::Bop::query(&schema_dsl::Load::query(), &addr, &first_load);

        DuplicateLoadPat::new(first_load, second_load)
    }

    #[eggplant::pat_vars]
    struct StoreForwardPat<PR: PatRecSgl> {
        write_val: schema_dsl::Expr,
        write: schema_dsl::Top,
        load: schema_dsl::Bop,
    }

    fn store_forward_pat<PR: PatRecSgl>() -> StoreForwardPat<PR> {
        let addr = schema_dsl::Expr::query_leaf();
        let write_val = schema_dsl::Expr::query_leaf();
        let state = schema_dsl::Expr::query_leaf();
        let write = schema_dsl::Top::query(&schema_dsl::Write::query(), &addr, &write_val, &state);
        let load = schema_dsl::Bop::query(&schema_dsl::Load::query(), &addr, &write);

        StoreForwardPat::new(write_val, write, load)
    }

    #[eggplant::pat_vars]
    struct DuplicateWritePat<PR: PatRecSgl> {
        first_write: schema_dsl::Top,
        second_write: schema_dsl::Top,
    }

    fn duplicate_write_pat<PR: PatRecSgl>() -> DuplicateWritePat<PR> {
        let addr = schema_dsl::Expr::query_leaf();
        let write_val = schema_dsl::Expr::query_leaf();
        let state = schema_dsl::Expr::query_leaf();
        let first_write =
            schema_dsl::Top::query(&schema_dsl::Write::query(), &addr, &write_val, &state);
        let second_write =
            schema_dsl::Top::query(&schema_dsl::Write::query(), &addr, &write_val, &first_write);

        DuplicateWritePat::new(first_write, second_write)
    }

    #[eggplant::pat_vars]
    struct ShadowedWritePat<PR: PatRecSgl> {
        addr: schema_dsl::Expr,
        write_val: schema_dsl::Expr,
        state: schema_dsl::Expr,
        second_write: schema_dsl::Top,
    }

    fn shadowed_write_pat<PR: PatRecSgl>() -> ShadowedWritePat<PR> {
        let addr = schema_dsl::Expr::query_leaf();
        let shadowed_val = schema_dsl::Expr::query_leaf();
        let write_val = schema_dsl::Expr::query_leaf();
        let state = schema_dsl::Expr::query_leaf();
        let first_write =
            schema_dsl::Top::query(&schema_dsl::Write::query(), &addr, &shadowed_val, &state);
        let second_write =
            schema_dsl::Top::query(&schema_dsl::Write::query(), &addr, &write_val, &first_write);

        ShadowedWritePat::new(addr, write_val, state, second_write)
    }

    pub(crate) fn register_native_rules() -> RuleSetId {
        let ruleset = RuleSetId("mem-simple");

        PeepholeTx::add_rule(
            "mem_simple_commute_write_then_load",
            ruleset,
            commute_write_load_pat,
            |ctx, pat| {
                let new_load = ctx
                    .ctx
                    .insert_bop(ctx.ctx.insert_load(), pat.load_addr, pat.state);
                let new_load_state = ctx.ctx.insert_get(new_load, 1_i64);
                let new_write = ctx.ctx.insert_top(
                    ctx.ctx.insert_write(),
                    pat.write_addr,
                    pat.write_val,
                    new_load_state,
                );
                let load_state = ctx.ctx.insert_get(pat.load, 1_i64);
                let load_value = ctx.ctx.insert_get(pat.load, 0_i64);
                let new_load_value = ctx.ctx.insert_get(new_load, 0_i64);

                ctx.union(load_state, new_write);
                ctx.union(load_value, new_load_value);
            },
        );

        PeepholeTx::add_rule(
            "mem_simple_duplicate_load",
            ruleset,
            duplicate_load_pat,
            |ctx, pat| {
                let first_value = ctx.ctx.insert_get(pat.first_load, 0_i64);
                let second_value = ctx.ctx.insert_get(pat.second_load, 0_i64);
                let first_state = ctx.ctx.insert_get(pat.first_load, 1_i64);
                let second_state = ctx.ctx.insert_get(pat.second_load, 1_i64);

                ctx.union(first_value, second_value);
                ctx.union(first_state, second_state);
            },
        );

        PeepholeTx::add_rule(
            "mem_simple_store_forward",
            ruleset,
            store_forward_pat,
            |ctx, pat| {
                let load_value = ctx.ctx.insert_get(pat.load, 0_i64);
                let load_state = ctx.ctx.insert_get(pat.load, 1_i64);

                ctx.union(load_value, pat.write_val);
                ctx.union(load_state, pat.write);
            },
        );

        PeepholeTx::add_rule(
            "mem_simple_duplicate_write",
            ruleset,
            duplicate_write_pat,
            |ctx, pat| {
                ctx.union(pat.first_write, pat.second_write);
            },
        );

        PeepholeTx::add_rule(
            "mem_simple_shadowed_write",
            ruleset,
            shadowed_write_pat,
            |ctx, pat| {
                let rewritten =
                    ctx.ctx
                        .insert_top(ctx.ctx.insert_write(), pat.addr, pat.write_val, pat.state);

                ctx.union(pat.second_write, rewritten);
            },
        );

        ruleset
    }
}

#[cfg(all(test, feature = "eggplant"))]
mod native_tests {
    use crate::ast::*;
    use crate::eggplant_backend::test_lock;
    use crate::schema::{BaseType, Type};

    fn candidate_parts() -> (String, String, String) {
        let one = int_ty(1, Type::Base(BaseType::IntT));
        let two = int(2).with_arg_types(tuplet!(statet()), Type::Base(intt()));
        let orig_state = get(arg_ty(tuplet!(statet())), 0);
        let ptr_and_state = alloc(0, one, orig_state.clone(), pointert(intt()));
        let ptr = get(ptr_and_state.clone(), 0);
        let state = get(ptr_and_state, 1);
        let write_expr = write(ptr.clone(), two.clone(), state);
        (
            load(ptr, write_expr.clone()).to_string(),
            two.to_string(),
            write_expr.to_string(),
        )
    }

    fn mem_simple_schedule() -> String {
        "(run-schedule mem-simple)".to_string()
    }

    fn text_store_forward_holds(
        prologue: &str,
        expr: &str,
        schedule: &str,
        expected_value: &str,
        expected_state: &str,
    ) {
        let program = format!(
            "{prologue}\n(let __rlcr_load {expr})\n{schedule}\n(check (= (Get __rlcr_load 0) {expected_value}))\n(check (= (Get __rlcr_load 1) {expected_state}))\n"
        );
        let mut egraph = egglog::EGraph::default();
        egraph.parse_and_run_program(None, &program).unwrap();
    }

    fn native_store_forward_holds(
        prologue: &str,
        expr: &str,
        schedule: &str,
        ablate: Option<&str>,
        expected_value: &str,
        expected_state: &str,
    ) -> std::result::Result<(), eggplant::egglog::Error> {
        let initialization = format!("(let __rlcr_load {expr})");
        crate::with_native_rules_egraph(prologue, &initialization, schedule, ablate, |egraph| {
            egraph.parse_and_run_program(
                None,
                &format!(
                    "(check (= (Get __rlcr_load 0) {expected_value}))\n(check (= (Get __rlcr_load 1) {expected_state}))"
                ),
            )?;
            Ok(())
        })
    }

    #[test]
    fn native_feature_path_matches_text_backend_for_mem_simple_case() {
        let _guard = test_lock::lock();
        let (expr, expected_value, expected_state) = candidate_parts();
        let schedule = mem_simple_schedule();

        text_store_forward_holds(
            &crate::prologue_egglog_text(),
            &expr,
            &schedule,
            &expected_value,
            &expected_state,
        );
        native_store_forward_holds(
            &crate::feature_execution_prologue(true, None),
            &expr,
            &schedule,
            None,
            &expected_value,
            &expected_state,
        )
        .unwrap();

        let ablated = native_store_forward_holds(
            &crate::feature_execution_prologue(true, Some("mem-simple")),
            &expr,
            &crate::ablate_schedule(&schedule, "mem-simple"),
            Some("mem-simple"),
            &expected_value,
            &expected_state,
        );

        assert!(
            ablated.is_err(),
            "ablating mem-simple should make the store-forward witness fail",
        );
    }
}
