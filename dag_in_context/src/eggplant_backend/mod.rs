mod schema;
mod schema_dsl;

pub(crate) fn prologue() -> String {
    // Ensure the `eggplant` dependency is linked when this feature is enabled,
    // even though the backend is still transitioning from `.egg` text to Rust.
    let _ = eggplant::prelude::RunConfig::Once;

    [
        &schema::fragment(),
        include_str!("../type_analysis.egg"),
        include_str!("../utility/util.egg"),
        include_str!("../utility/terms.egg"),
        &crate::optimizations::is_valid::rules().join("\n"),
        &crate::optimizations::is_resolved::rules().join("\n"),
        &crate::optimizations::body_contains::rules().join("\n"),
        include_str!("../optimizations/purity_analysis.egg"),
        // TODO cond inv code motion with regions
        //&crate::optimizations::conditional_invariant_code_motion::rules().join("\n"),
        include_str!("../utility/add_context.egg"),
        include_str!("../utility/context-prop.egg"),
        include_str!("../utility/term-subst.egg"),
        include_str!("../utility/context_of.egg"),
        include_str!("../utility/subst.egg"),
        include_str!("../utility/canonicalize.egg"),
        include_str!("../utility/expr_size.egg"),
        include_str!("../utility/drop_at.egg"),
        include_str!("../interval_analysis.egg"),
        include_str!("../optimizations/switch_rewrites.egg"),
        include_str!("../optimizations/select.egg"),
        include_str!("../optimizations/peepholes.egg"),
        &crate::optimizations::memory::rules(),
        include_str!("../optimizations/memory.egg"),
        include_str!("../optimizations/mem_simple.egg"),
        &crate::optimizations::loop_invariant::rules().join("\n"),
        include_str!("../optimizations/loop_simplify.egg"),
        include_str!("../optimizations/loop_unroll.egg"),
        include_str!("../optimizations/swap_if.egg"),
        include_str!("../optimizations/rec_to_loop.egg"),
        include_str!("../optimizations/passthrough.egg"),
        include_str!("../optimizations/loop_strength_reduction.egg"),
        include_str!("../optimizations/ivt.egg"),
        include_str!("../optimizations/conditional_invariant_code_motion.egg"),
        include_str!("../optimizations/conditional_push_in.egg"),
        include_str!("../utility/debug-helper.egg"),
        include_str!("../optimizations/hackers_delight.egg"),
        include_str!("../optimizations/non_weakly_linear.egg"),
        &crate::schedule::rulesets(),
    ]
    .join("\n")
}

#[cfg(test)]
mod tests {
    fn strip_schema_types_section(program: &str) -> String {
        const TYPES_HEADER: &str = r#"; =================================
; Types
; =================================

"#;
        const ASSUMPTIONS_HEADER: &str = r#"; =================================
; Assumptions
; =================================

"#;

        let types_header_start = program
            .find(TYPES_HEADER)
            .expect("schema.egg must contain the Types section header");
        let types_body_start = types_header_start + TYPES_HEADER.len();
        let assumptions_header_start = program
            .find(ASSUMPTIONS_HEADER)
            .expect("schema.egg must contain the Assumptions section header");

        let mut stripped = String::new();
        stripped.push_str(&program[..types_body_start]);
        stripped.push_str(&program[assumptions_header_start..]);
        stripped
    }

    fn eval_and_extract_expr(prologue: &str, expr: &str) -> String {
        let binding = "__rlcr_expr";
        let program = format!("{prologue}\n(let {binding} {expr})\n");

        let mut egraph = egglog::EGraph::default();
        egraph.parse_and_run_program(None, &program).unwrap();

        let mut termdag = egglog::TermDag::default();
        let (sort, value) = egraph
            .eval_expr(&egglog::ast::Expr::Var(
                egglog::ast::Span::Panic,
                binding.into(),
            ))
            .unwrap();
        let (_, extracted) = egraph.extract(value, &mut termdag, &sort).unwrap();
        termdag.to_string(&extracted)
    }

    #[test]
    fn schema_fragment_matches_schema_file_except_types_section() {
        let expected = include_str!("../schema.egg");
        let actual = super::schema::fragment();

        let expected = strip_schema_types_section(expected);
        let actual = strip_schema_types_section(&actual);

        let diff = if expected == actual {
            String::new()
        } else {
            similar::TextDiff::from_lines(&expected, &actual)
                .unified_diff()
                .header("schema.egg (types stripped)", "schema::fragment() (types stripped)")
                .to_string()
        };

        insta::assert_snapshot!(diff, @"");
    }

    #[test]
    fn prologue_matches_text_backend_except_schema_types_section() {
        let expected = crate::prologue_egglog_text();
        let actual = crate::prologue();

        let expected = strip_schema_types_section(&expected);
        let actual = strip_schema_types_section(&actual);

        let diff = if expected == actual {
            String::new()
        } else {
            similar::TextDiff::from_lines(&expected, &actual)
                .unified_diff()
                .header(
                    "prologue_egglog_text() (types stripped)",
                    "eggplant_backend::prologue() (types stripped)",
                )
                .to_string()
        };

        insta::assert_snapshot!(diff, @"");
    }

    #[test]
    fn prologue_is_semantically_equivalent_on_fixed_input() {
        let expr = r#"(Const (Int 42) (Base (IntT)) (InFunc "DUMMY"))"#;

        let expected = eval_and_extract_expr(&crate::prologue_egglog_text(), expr);
        let actual = eval_and_extract_expr(&crate::prologue(), expr);

        let diff = if expected == actual {
            String::new()
        } else {
            similar::TextDiff::from_lines(&expected, &actual)
                .unified_diff()
                .header("text backend extracted expr", "eggplant backend extracted expr")
                .to_string()
        };

        insta::assert_snapshot!(diff, @"");
    }
}
