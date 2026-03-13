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
    fn strip_schema_generated_sections(program: &str) -> String {
        const EXPR_DECL_HEADER: &str = "; Every term is an `Expr` or a `ListExpr`.\n";
        const LIST_EXPR_HEADER: &str = r#"; Used for constructing a list of branches for `Switch`es
; or a list of functions in a `Program`.
"#;
        const TYPES_HEADER: &str = r#"; =================================
; Types
; =================================

"#;
        const ASSUMPTIONS_HEADER: &str = r#"; =================================
; Assumptions
; =================================

"#;
        const LEAF_NODES_HEADER: &str = r#"; =================================
; Leaf nodes
; Constants, argument, and empty tuple
; =================================

"#;
        const CONSTANTS_MARKER: &str = "; Constants\n";
        const CONST_CONSTRUCTOR_COMMENT: &str = "; All leaf nodes need the type of the argument\n";
        const OPERATORS_HEADER: &str = r#"; =================================
; Operators
; =================================

"#;
        const OPERATORS_CONSTRUCTORS_START: &str = "(constructor Get";
        const TOP_LEVEL_EXPRESSIONS_HEADER: &str = r#"; =================================
; Top-level expressions
; =================================
"#;
        const FUNCTION_CONSTRUCTOR_START: &str = "(constructor Function";
        const TERMS_MARKER: &str = "; TERMS\n";
        const TERM_ASSUMPTION_TODO_COMMENT: &str =
            "; TODO: Will probably need ctx so that we can resubstitute?\n";

        fn strip_section(program: &str, header: &str, next: &str) -> String {
            let header_start = program
                .find(header)
                .unwrap_or_else(|| panic!("schema.egg must contain the section header:\n{header}"));
            let body_start = header_start + header.len();
            let next_start = program[body_start..]
                .find(next)
                .map(|idx| idx + body_start)
                .unwrap_or_else(|| panic!("schema.egg must contain the section boundary:\n{next}"));

            let mut stripped = String::new();
            stripped.push_str(&program[..body_start]);
            stripped.push_str(&program[next_start..]);
            stripped
        }

        let stripped = strip_section(program, EXPR_DECL_HEADER, LIST_EXPR_HEADER);
        let stripped = strip_section(&stripped, LIST_EXPR_HEADER, TYPES_HEADER);
        let stripped = strip_section(&stripped, TYPES_HEADER, ASSUMPTIONS_HEADER);
        let stripped = strip_section(&stripped, ASSUMPTIONS_HEADER, LEAF_NODES_HEADER);
        let stripped = strip_section(&stripped, CONSTANTS_MARKER, CONST_CONSTRUCTOR_COMMENT);
        let stripped = strip_section(&stripped, OPERATORS_HEADER, OPERATORS_CONSTRUCTORS_START);
        let stripped = strip_section(
            &stripped,
            TOP_LEVEL_EXPRESSIONS_HEADER,
            FUNCTION_CONSTRUCTOR_START,
        );
        let stripped = strip_section(&stripped, TERMS_MARKER, TERM_ASSUMPTION_TODO_COMMENT);

        stripped
            .replace("(constructor Arg (Type Assumption) Expr)\n", "")
            .replace("(constructor Const (Constant Type Assumption) Expr)\n", "")
            .replace("(constructor Empty (Type Assumption) Expr)\n", "")
            .replace("(constructor Top   (TernaryOp Expr Expr Expr) Expr)\n", "")
            .replace("(constructor Bop   (BinaryOp Expr Expr) Expr)\n", "")
            .replace("(constructor Uop   (UnaryOp Expr) Expr)\n", "")
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
    fn schema_fragment_matches_schema_file_except_generated_sections() {
        let expected = include_str!("../schema.egg");
        let actual = super::schema::fragment();

        let expected = strip_schema_generated_sections(expected);
        let actual = strip_schema_generated_sections(&actual);

        let diff = if expected == actual {
            String::new()
        } else {
            similar::TextDiff::from_lines(&expected, &actual)
                .unified_diff()
                .header(
                    "schema.egg (generated sections stripped)",
                    "schema::fragment() (generated sections stripped)",
                )
                .to_string()
        };

        insta::assert_snapshot!(diff, @"");
    }

    #[test]
    fn schema_fragment_contains_generated_markers_for_expr_terms_and_program_type() {
        let schema = super::schema::fragment();
        let marker = "; (Generated from eggplant DSL: src/eggplant_backend/schema_dsl.rs)\n";

        assert!(
            schema.contains(&format!(
                "; Every term is an `Expr` or a `ListExpr`.\n{marker}"
            )),
            "schema::fragment() must inject the Expr datatype from eggplant DSL"
        );
        assert!(
            schema.contains(&format!("; TERMS\n{marker}")),
            "schema::fragment() must inject the Terms datatypes from eggplant DSL"
        );
        assert!(
            schema.contains(&format!(
                "; =================================\n; Top-level expressions\n; =================================\n{marker}"
            )),
            "schema::fragment() must inject the ProgramType sort/constructor from eggplant DSL"
        );
    }

    #[test]
    fn injected_expr_section_contains_migrated_constructors() {
        const EXPR_DECL_HEADER: &str = "; Every term is an `Expr` or a `ListExpr`.\n";
        const LIST_EXPR_HEADER: &str = r#"; Used for constructing a list of branches for `Switch`es
; or a list of functions in a `Program`.
"#;
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant DSL: src/eggplant_backend/schema_dsl.rs)\n";

        let schema = super::schema::fragment();
        let expr_block_start = schema
            .find(EXPR_DECL_HEADER)
            .expect("schema::fragment() must contain the Expr declaration header")
            + EXPR_DECL_HEADER.len();
        let expr_block_end = schema
            .find(LIST_EXPR_HEADER)
            .expect("schema::fragment() must contain the ListExpr section header");
        let expr_block = &schema[expr_block_start..expr_block_end];

        assert!(
            expr_block.contains(GENERATED_MARKER),
            "Expr section must be generated from eggplant DSL"
        );

        for constructor in ["(Arg", "(Const", "(Empty", "(Top", "(Bop", "(Uop"] {
            assert!(
                expr_block.contains(constructor),
                "Injected Expr section must contain {constructor}"
            );
        }
    }

    #[test]
    fn schema_fragment_parses_migrated_expr_constructor_with_dependent_sorts() {
        let program = format!(
            "{}\n(let __rlcr_expr (Arg (Base (IntT)) (InFunc \"DUMMY\")))\n",
            super::schema::fragment()
        );

        let mut egraph = egglog::EGraph::default();
        egraph.parse_and_run_program(None, &program).unwrap();
    }

    #[test]
    fn prologue_matches_text_backend_except_generated_schema_sections() {
        let expected = crate::prologue_egglog_text();
        let actual = crate::prologue();

        let expected = strip_schema_generated_sections(&expected);
        let actual = strip_schema_generated_sections(&actual);

        let diff = if expected == actual {
            String::new()
        } else {
            similar::TextDiff::from_lines(&expected, &actual)
                .unified_diff()
                .header(
                    "prologue_egglog_text() (generated sections stripped)",
                    "eggplant_backend::prologue() (generated sections stripped)",
                )
                .to_string()
        };

        insta::assert_snapshot!(diff, @"");
    }

    #[test]
    fn prologue_is_semantically_equivalent_on_fixed_input() {
        let expr = r#"(Switch (Const (Int 0) (Base (IntT)) (InFunc "DUMMY")) (Empty (TupleT (TNil)) (InFunc "DUMMY")) (Cons (Const (Int 42) (Base (IntT)) (InLoop (Arg (Base (StateT)) (InFunc "DUMMY")) (Empty (TupleT (TNil)) (InFunc "DUMMY")))) (Nil)))"#;

        let expected = eval_and_extract_expr(&crate::prologue_egglog_text(), expr);
        let actual = eval_and_extract_expr(&crate::prologue(), expr);

        let diff = if expected == actual {
            String::new()
        } else {
            similar::TextDiff::from_lines(&expected, &actual)
                .unified_diff()
                .header(
                    "text backend extracted expr",
                    "eggplant backend extracted expr",
                )
                .to_string()
        };

        insta::assert_snapshot!(diff, @"");
    }

    #[test]
    fn prologue_is_semantically_equivalent_on_terms_fixed_input() {
        let list_term = r#"(TermCons (TermArg) (TermNil))"#;

        let expected = eval_and_extract_expr(&crate::prologue_egglog_text(), list_term);
        let actual = eval_and_extract_expr(&crate::prologue(), list_term);

        let diff = if expected == actual {
            String::new()
        } else {
            similar::TextDiff::from_lines(&expected, &actual)
                .unified_diff()
                .header(
                    "text backend extracted term",
                    "eggplant backend extracted term",
                )
                .to_string()
        };

        insta::assert_snapshot!(diff, @"");
    }

    #[test]
    fn prologue_is_semantically_equivalent_on_tiny_program() {
        fn run_and_extract(program: &crate::schema::TreeProgram, egglog_program: &str) -> String {
            let mut egraph = egglog::EGraph::default();
            egraph.parse_and_run_program(None, egglog_program).unwrap();

            let (serialized, unextractables) =
                crate::greedy_dag_extractor::serialized_egraph(egraph);

            let mut termdag = egglog::TermDag::default();
            let extracted = crate::greedy_dag_extractor::greedy_dag_extract(
                program,
                program.fns(),
                serialized,
                unextractables,
                &mut termdag,
                crate::greedy_dag_extractor::DefaultCostModel,
                true,
                false,
            )
            .1;

            extracted.add_dummy_ctx().0.to_string()
        }

        let main = crate::ast::function(
            "main",
            crate::ast::tuplet_vec(vec![crate::ast::intt(), crate::ast::statet()]),
            crate::ast::tuplet_vec(vec![crate::ast::intt(), crate::ast::statet()]),
            {
                let arg = crate::ast::arg_ty(crate::ast::tuplet_vec(vec![
                    crate::ast::intt(),
                    crate::ast::statet(),
                ]));
                crate::ast::parallel_vec(vec![
                    crate::ast::add(crate::ast::first(arg.clone()), crate::ast::int(1)),
                    crate::ast::second(arg),
                ])
            },
        );
        let program = crate::ast::program_vec(main, vec![]);

        let schedule = format!("(run-schedule {})", crate::schedule::types_and_indexing());
        let egglog_prog =
            crate::build_program(&program, None, &program.fns(), &schedule, None, true);

        let suffix_anchor = "(relation InlinedCall (String Expr))";
        let suffix_start = egglog_prog
            .find(suffix_anchor)
            .expect("build_program output must contain the InlinedCall relation (used as the prologue boundary for this test)");
        let suffix = &egglog_prog[suffix_start..];

        let expected_program_egglog = format!(
            "\n; Prologue\n{}\n\n{}",
            crate::prologue_egglog_text(),
            suffix
        );
        let actual_program_egglog = format!("\n; Prologue\n{}\n\n{}", crate::prologue(), suffix);

        let expected = run_and_extract(&program, &expected_program_egglog);
        let actual = run_and_extract(&program, &actual_program_egglog);

        let diff = if expected == actual {
            String::new()
        } else {
            similar::TextDiff::from_lines(&expected, &actual)
                .unified_diff()
                .header(
                    "text backend extracted program",
                    "eggplant backend extracted program",
                )
                .to_string()
        };

        insta::assert_snapshot!(diff, @"");
    }
}
