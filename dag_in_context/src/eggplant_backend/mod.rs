mod add_context;
mod context_prop;
mod purity_analysis;
mod schema;
mod schema_dsl;
mod term_subst;
mod terms;
mod type_analysis;
mod util;

pub(crate) fn prologue() -> String {
    // Ensure the `eggplant` dependency is linked when this feature is enabled,
    // even though the backend is still transitioning from `.egg` text to Rust.
    let _ = eggplant::prelude::RunConfig::Once;

    [
        &schema::fragment(),
        &type_analysis::fragment(),
        &util::fragment(),
        &terms::fragment(),
        &crate::optimizations::is_valid::rules().join("\n"),
        &crate::optimizations::is_resolved::rules().join("\n"),
        &crate::optimizations::body_contains::rules().join("\n"),
        &purity_analysis::fragment(),
        // TODO cond inv code motion with regions
        //&crate::optimizations::conditional_invariant_code_motion::rules().join("\n"),
        &add_context::fragment(),
        &context_prop::fragment(),
        &term_subst::fragment(),
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
        const TUPLE_OPERATIONS_HEADER: &str = r#"; =================================
; Tuple operations
; =================================

"#;
        const CONTROL_FLOW_HEADER: &str = r#"; =================================
; Control flow
; =================================

"#;
        const TOP_LEVEL_EXPRESSIONS_HEADER: &str = r#"; =================================
; Top-level expressions
; =================================
"#;
        const FUNCTION_HAS_TYPE_RELATION: &str = "(relation FunctionHasType";
        const TERMS_MARKER: &str = "; TERMS\n";
        const LOOP_NUM_ITERS_GUESS_FUNCTION: &str = "(function LoopNumItersGuess";

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
        let stripped = strip_section(&stripped, OPERATORS_HEADER, TUPLE_OPERATIONS_HEADER);
        let stripped = strip_section(&stripped, TUPLE_OPERATIONS_HEADER, CONTROL_FLOW_HEADER);
        let stripped = strip_section(&stripped, CONTROL_FLOW_HEADER, TOP_LEVEL_EXPRESSIONS_HEADER);
        let stripped = strip_section(
            &stripped,
            TOP_LEVEL_EXPRESSIONS_HEADER,
            FUNCTION_HAS_TYPE_RELATION,
        );
        let stripped = strip_section(&stripped, TERMS_MARKER, LOOP_NUM_ITERS_GUESS_FUNCTION);

        stripped
            .replace("(constructor Arg (Type Assumption) Expr)\n", "")
            .replace("(constructor Const (Constant Type Assumption) Expr)\n", "")
            .replace("(constructor Empty (Type Assumption) Expr)\n", "")
    }

    fn strip_type_analysis_generated_sections(program: &str) -> String {
        const RAW_START: &str = "(ruleset type-analysis)\n";
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/type_analysis.rs)\n";
        const END_MARKER: &str = "; find which types are pure\n";

        let start = program
            .find(GENERATED_MARKER)
            .or_else(|| program.find(RAW_START))
            .expect("type_analysis.egg fragment must contain the generated marker or raw start");
        let end = program[start..]
            .find(END_MARKER)
            .map(|idx| idx + start)
            .unwrap_or(program.len());

        let mut stripped = String::new();
        stripped.push_str(&program[..start]);
        stripped.push_str(&program[end..]);
        stripped
    }

    fn strip_util_generated_sections(program: &str) -> String {
        const RAW_START: &str = "(function ListExpr-length (ListExpr) i64 :no-merge)\n";
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/util.rs)\n";
        const NEXT_PROLOGUE_BOUNDARY: &str = "(ruleset terms)\n";

        let start = program
            .find(GENERATED_MARKER)
            .or_else(|| program.find(RAW_START))
            .expect("util.egg fragment must contain the generated marker or raw start");
        let end = program[start..]
            .find(NEXT_PROLOGUE_BOUNDARY)
            .map(|idx| idx + start)
            .unwrap_or(program.len());

        let mut stripped = String::new();
        stripped.push_str(&program[..start]);
        stripped.push_str(&program[end..]);
        stripped
    }

    fn strip_terms_generated_sections(program: &str) -> String {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/terms.rs)\n";

        let mut stripped = program.replace(GENERATED_MARKER, "");
        while stripped.contains("\n\n\n") {
            stripped = stripped.replace("\n\n\n", "\n\n");
        }
        stripped
    }

    fn strip_purity_analysis_generated_sections(program: &str) -> String {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/purity_analysis.rs)\n";
        const NEXT_SECTION_HEADER: &str =
            "; This file provides AddContext, a helpers that copies a sub-egraph into\n";

        let mut stripped = program.replace(GENERATED_MARKER, "");
        while stripped.contains("\n\n\n") {
            stripped = stripped.replace("\n\n\n", "\n\n");
        }
        stripped = stripped.replace(
            &format!("\n\n{NEXT_SECTION_HEADER}"),
            &format!("\n{NEXT_SECTION_HEADER}"),
        );
        stripped
    }

    fn strip_add_context_generated_sections(program: &str) -> String {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/add_context.rs)\n";
        const SECTION_HEADER: &str =
            "; This file provides AddContext, a helpers that copies a sub-egraph into\n";
        const NEXT_SECTION_HEADER: &str = "(ruleset term-subst)\n";

        let mut stripped = program.replace(GENERATED_MARKER, "");
        while stripped.contains("\n\n\n") {
            stripped = stripped.replace("\n\n\n", "\n\n");
        }
        stripped = stripped.replace(
            &format!("\n\n{SECTION_HEADER}"),
            &format!("\n{SECTION_HEADER}"),
        );
        stripped = stripped.replace(
            &format!("\n\n{NEXT_SECTION_HEADER}"),
            &format!("\n{NEXT_SECTION_HEADER}"),
        );
        stripped
    }

    fn strip_context_prop_generated_sections(program: &str) -> String {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/context_prop.rs)\n";
        const NEXT_SECTION_HEADER: &str = "(ruleset term-subst)\n";

        let mut stripped = program.replace(GENERATED_MARKER, "");
        while stripped.contains("\n\n\n") {
            stripped = stripped.replace("\n\n\n", "\n\n");
        }
        stripped = stripped.replace(
            &format!("\n\n{NEXT_SECTION_HEADER}"),
            &format!("\n{NEXT_SECTION_HEADER}"),
        );
        stripped
    }

    fn strip_term_subst_generated_sections(program: &str) -> String {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/term_subst.rs)\n";
        const SECTION_HEADER: &str = "(ruleset term-subst)\n";
        const NEXT_SECTION_HEADER: &str = "; We only have context for Exprs, not ListExprs.\n";

        let mut stripped = program.replace(GENERATED_MARKER, "");
        while stripped.contains("\n\n\n") {
            stripped = stripped.replace("\n\n\n", "\n\n");
        }
        stripped = stripped.replace(
            &format!("\n\n{SECTION_HEADER}"),
            &format!("\n{SECTION_HEADER}"),
        );
        stripped = stripped.replace(
            &format!("\n\n{NEXT_SECTION_HEADER}"),
            &format!("\n{NEXT_SECTION_HEADER}"),
        );
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

        for constructor in [
            "(Arg",
            "(Const",
            "(Empty",
            "(Top",
            "(Bop",
            "(Uop",
            "(Get",
            "(Alloc",
            "(Call",
            "(Single",
            "(Concat",
            "(Switch",
            "(If",
            "(DoWhile",
            "(Function",
        ] {
            assert!(
                expr_block.contains(constructor),
                "Injected Expr section must contain {constructor}"
            );
        }
    }

    #[test]
    fn injected_terms_section_contains_migrated_constructors() {
        const TERMS_MARKER: &str = "; TERMS\n";
        const LOOP_NUM_ITERS_GUESS_FUNCTION: &str = "(function LoopNumItersGuess";
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant DSL: src/eggplant_backend/schema_dsl.rs)\n";

        let schema = super::schema::fragment();
        let terms_block_start = schema
            .find(TERMS_MARKER)
            .expect("schema::fragment() must contain the Terms marker")
            + TERMS_MARKER.len();
        let terms_block_end = schema
            .find(LOOP_NUM_ITERS_GUESS_FUNCTION)
            .expect("schema::fragment() must contain the LoopNumItersGuess function");
        let terms_block = &schema[terms_block_start..terms_block_end];

        assert!(
            terms_block.contains(GENERATED_MARKER),
            "Terms section must be generated from eggplant DSL"
        );

        for constructor in [
            "(TermArg",
            "(TermConst",
            "(TermEmpty",
            "(TermTop",
            "(TermBop",
            "(TermUop",
            "(TermGet",
            "(TermAlloc",
            "(TermCall",
            "(TermSingle",
            "(TermConcat",
        ] {
            assert!(
                terms_block.contains(constructor),
                "Injected Terms section must contain {constructor}"
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
    fn schema_fragment_parses_migrated_operator_expr_constructors() {
        let program = format!(
            "{}\n(let __rlcr_expr (Call \"callee\" (Get (Alloc 0 (Const (Int 4) (Base (IntT)) (InFunc \"DUMMY\")) (Arg (Base (StateT)) (InFunc \"DUMMY\")) (IntT)) 0)))\n",
            super::schema::fragment()
        );

        let mut egraph = egglog::EGraph::default();
        egraph.parse_and_run_program(None, &program).unwrap();
    }

    #[test]
    fn schema_fragment_parses_migrated_tuple_operation_constructors() {
        let program = format!(
            "{}\n(let __rlcr_expr (Concat (Single (Const (Int 1) (Base (IntT)) (InFunc \"DUMMY\"))) (Single (Arg (Base (IntT)) (InFunc \"DUMMY\")))))\n",
            super::schema::fragment()
        );

        let mut egraph = egglog::EGraph::default();
        egraph.parse_and_run_program(None, &program).unwrap();
    }

    #[test]
    fn schema_fragment_parses_migrated_control_flow_constructors() {
        let program = format!(
            "{}\n(let __rlcr_expr (If (Const (Bool true) (Base (BoolT)) (InFunc \"DUMMY\")) (Empty (TupleT (TNil)) (InFunc \"DUMMY\")) (Switch (Const (Int 0) (Base (IntT)) (InFunc \"DUMMY\")) (Empty (TupleT (TNil)) (InFunc \"DUMMY\")) (Cons (Single (Const (Int 1) (Base (IntT)) (InFunc \"DUMMY\"))) (Nil))) (DoWhile (Empty (TupleT (TNil)) (InFunc \"DUMMY\")) (Single (Const (Bool false) (Base (BoolT)) (InFunc \"DUMMY\"))))))\n",
            super::schema::fragment()
        );

        let mut egraph = egglog::EGraph::default();
        egraph.parse_and_run_program(None, &program).unwrap();
    }

    #[test]
    fn schema_fragment_parses_migrated_function_constructor() {
        let program = format!(
            "{}\n(let __rlcr_expr (Function \"main\" (Base (IntT)) (Base (IntT)) (Arg (Base (IntT)) (InFunc \"main\"))))\n",
            super::schema::fragment()
        );

        let mut egraph = egglog::EGraph::default();
        egraph.parse_and_run_program(None, &program).unwrap();
    }

    #[test]
    fn schema_fragment_parses_migrated_term_leaf_constructors() {
        let program = format!(
            "{}\n(let __rlcr_term (TermCons (TermConst (Int 1)) (TermCons (TermEmpty) (TermCons (TermArg) (TermNil)))))\n",
            super::schema::fragment()
        );

        let mut egraph = egglog::EGraph::default();
        egraph.parse_and_run_program(None, &program).unwrap();
    }

    #[test]
    fn schema_fragment_parses_migrated_term_operator_constructors() {
        let program = format!(
            "{}\n(let __rlcr_term (TermCall \"callee\" (TermGet (TermAlloc 0 (TermConst (Int 4)) (TermArg) (IntT)) 0)))\n",
            super::schema::fragment()
        );

        let mut egraph = egglog::EGraph::default();
        egraph.parse_and_run_program(None, &program).unwrap();
    }

    #[test]
    fn schema_fragment_parses_migrated_tuple_term_constructors() {
        let program = format!(
            "{}\n(let __rlcr_term (TermConcat (TermSingle (TermConst (Int 1))) (TermSingle (TermCall \"callee\" (TermArg)))))\n",
            super::schema::fragment()
        );

        let mut egraph = egglog::EGraph::default();
        egraph.parse_and_run_program(None, &program).unwrap();
    }

    #[test]
    fn type_analysis_fragment_matches_type_analysis_file_except_generated_sections() {
        let expected = include_str!("../type_analysis.egg");
        let actual = super::type_analysis::fragment();

        let expected = strip_type_analysis_generated_sections(expected);
        let actual = strip_type_analysis_generated_sections(&actual);

        let diff = if expected == actual {
            String::new()
        } else {
            similar::TextDiff::from_lines(&expected, &actual)
                .unified_diff()
                .header(
                    "type_analysis.egg (generated sections stripped)",
                    "type_analysis::fragment() (generated sections stripped)",
                )
                .to_string()
        };

        insta::assert_snapshot!(diff, @"");
    }

    #[test]
    fn type_analysis_fragment_contains_generated_declaration_prefix() {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/type_analysis.rs)\n";

        let fragment = super::type_analysis::fragment();
        let generated_prefix = fragment.as_str();

        assert!(
            generated_prefix.contains(GENERATED_MARKER),
            "type_analysis::fragment() must mark the generated declaration prefix"
        );

        for declaration in [
            "(ruleset type-analysis)",
            "(ruleset type-helpers)",
            "(constructor TLConcat",
            "(function TypeList-length",
            "(constructor TypeList-ith",
            "(relation HasType",
            "(relation ExpectType",
            "(relation HasArgType",
            "(rule ((= lhs (Uop _ e))",
            "(rule ((= lhs (DoWhile ins body))",
            "(rule ((= lhs (Const (Int i) ty ctx)))",
            "(rule ((= lhs (Empty ty ctx)))",
            "(ExpectType e (Base (BoolT)) \"(Not)\")",
            "(ExpectType e (Base (IntT)) \"(Neg)\")",
            "(panic \"Don't print a tuple\")",
            "(panic \"(Select) branches had different types\")",
            "(relation bop-of-type",
            "(relation bpred-of-type",
            "(bop-of-type (Add) (Base (IntT)))",
            "(ExpectType val (Base ty) \"(Write)\")",
            "(Bop (PtrAdd) ptr n)",
            "(ExpectType amt (Base (IntT)) \"(Alloc)\")",
            "(HasType lhs (Base (TypeList-ith tylist i)))",
            "(panic \"index out of bounds\")",
            "(panic \"negative index\")",
            "(panic \"don't nest tuples\")",
            "(HasType lhs (TupleT (TCons basety (TNil))))",
            "(HasType lhs (TupleT (TLConcat tylist1 tylist2)))",
            "(ExpectType pred (Base (BoolT)) \"If predicate must be boolean\")",
            "(panic \"if branches had different types\")",
            "(ExpectType pred (Base (IntT)) \"Switch predicate must be integer\")",
            "(panic \"switch branches had different types\")",
            "(HasType (Arg ty ctx) ty)",
            "(panic \"loop input must be tuple\")",
            "(ExpectType (Get pred-body 0) (Base (BoolT)) \"loop pred must be bool\")",
            "(panic \"input types and output types don't match\")",
            "(ExpectType body out-ty \"Function body had wrong type\")",
            "(ExpectType arg in-ty \"function called with wrong arg type\")",
            "(HasType lhs out-ty)",
            "(relation PureBaseType",
            "(relation PureType",
            "(relation PureTypeList",
            "(PureBaseType (IntT))",
            "(PureType (Base ty))",
            "(PureTypeList (TCons hd tl))",
        ] {
            assert!(
                generated_prefix.contains(declaration),
                "Generated type_analysis prefix must contain {declaration}"
            );
        }
    }

    #[test]
    fn util_fragment_matches_util_file_except_generated_sections() {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/util.rs)\n";
        const TUPLE_LENGTH_DECL: &str = "(function tuple-length (Expr) i64 :no-merge)\n";
        const TMP_CTX_DECL: &str = "(constructor TmpCtx () Assumption)\n";

        fn normalize_util_text(text: &str) -> String {
            let mut normalized = text
                .replace(GENERATED_MARKER, "")
                .replace(TUPLE_LENGTH_DECL, "")
                .replace(TMP_CTX_DECL, "");

            while normalized.contains("\n\n\n") {
                normalized = normalized.replace("\n\n\n", "\n\n");
            }

            normalized = normalized.replace(
                "the query.\n\n(rule ((TmpCtx))",
                "the query.\n(rule ((TmpCtx))",
            );

            normalized.trim_end().to_string()
        }

        let expected = normalize_util_text(include_str!("../utility/util.egg"));
        let actual = normalize_util_text(&super::util::fragment());

        let diff = if expected == actual {
            String::new()
        } else {
            similar::TextDiff::from_lines(&expected, &actual)
                .unified_diff()
                .header(
                    "utility/util.egg (generated sections stripped)",
                    "util::fragment() (generated sections stripped)",
                )
                .to_string()
        };

        insta::assert_snapshot!(diff, @"");
    }

    #[test]
    fn util_fragment_contains_generated_prefix() {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/util.rs)\n";

        let fragment = super::util::fragment();
        let generated_prefix = fragment.as_str();

        assert!(
            generated_prefix.contains(GENERATED_MARKER),
            "util::fragment() must mark the generated prefix"
        );

        for declaration in [
            "(function ListExpr-length",
            "(constructor ListExpr-ith",
            "(constructor ListExpr-suffix",
            "(constructor Append",
            "(function tuple-length",
            "(relation leading-Expr",
            "(relation leading-Expr-list",
            "(relation Add-Gets",
            "(relation Not-Just-Concat",
            "(relation Add-All-Gets",
            "(constructor TmpCtx",
            "(ruleset subsume-after-helpers)",
            "(relation ToSubsumeIf",
            "(ruleset add-to-debug-expr)",
            ":no-merge",
            ":unextractable",
            "(union (ListExpr-suffix branch 0) branch)",
            "(union (ListExpr-ith top n) hd)",
            "(set (ListExpr-length list) n)",
            "(rewrite (Append (Cons a b) e)",
            "(rewrite (Append (Nil) e)",
            "(set (tuple-length expr) len)",
            "(leading-Expr inputs)",
            "(leading-Expr-list branch)",
            "(union (Get (Single expr) 0) expr)",
            "(Get tuple 0)",
            "(Get tuple (+ 1 i))",
            "(Add-Gets orig right (+ n len))",
            "(union (Get orig n) e)",
            "(Add-All-Gets orig something n 0)",
            "(union (Get orig (+ offset pos)) (Get something pos))",
            "(Not-Just-Concat lhs)",
            "(panic \"TmpCtx should not exist outside rule body\")",
            "(subsume (If a b c d))",
        ] {
            assert!(
                generated_prefix.contains(declaration),
                "Generated util prefix must contain {declaration}"
            );
        }
    }

    #[test]
    fn terms_fragment_matches_terms_file_except_generated_sections() {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/terms.rs)\n";

        let expected = include_str!("../utility/terms.egg").trim_end().to_string();
        let actual = super::terms::fragment();

        let actual = actual.replace(GENERATED_MARKER, "").trim_end().to_string();

        let diff = if expected == actual {
            String::new()
        } else {
            similar::TextDiff::from_lines(&expected, &actual)
                .unified_diff()
                .header(
                    "utility/terms.egg (generated sections stripped)",
                    "terms::fragment() (generated sections stripped)",
                )
                .to_string()
        };

        insta::assert_snapshot!(diff, @"");
    }

    #[test]
    fn terms_fragment_contains_generated_prefix() {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/terms.rs)\n";

        let fragment = super::terms::fragment();
        let generated_prefix = fragment.as_str();

        assert!(
            generated_prefix.contains(GENERATED_MARKER),
            "terms::fragment() must mark the generated prefix"
        );

        for declaration in [
            "(ruleset terms)",
            "(ruleset terms-helpers)",
            "(ruleset terms-helpers-helpers)",
            "(sort TermAndCost)",
            "(constructor Smaller",
            "(function ExtractedExpr",
            ":merge (Smaller old new)",
            "(relation PotentialExtractedExpr",
            "(constructor TCPair",
            "(constructor NoTerm",
            "(TCPair (NoTerm) 10000000000000000)",
            "(panic \"Negative cost\")",
            "(union lhs (TCPair t1 cost1))",
            "(union lhs (TCPair t2 cost2))",
            "(PotentialExtractedExpr lhs (TCPair (TermConst c) 1))",
            "(PotentialExtractedExpr lhs (TCPair (TermArg) 1))",
            "(TermBop o t1 t2)",
            "(TermTop o t1 t2 t3)",
            "(TermUop o t1)",
            "(TermGet t1 i)",
            "(TermSingle t1)",
            "(TermConcat t1 t2)",
            "(sort Node)",
            "(constructor IfNode",
        ] {
            assert!(
                generated_prefix.contains(declaration),
                "Generated terms prefix must contain {declaration}"
            );
        }
    }

    #[test]
    fn purity_analysis_fragment_matches_file_except_generated_sections() {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/purity_analysis.rs)\n";

        let expected = include_str!("../optimizations/purity_analysis.egg")
            .trim_end()
            .to_string();
        let actual = super::purity_analysis::fragment();
        let actual = actual.replace(GENERATED_MARKER, "").trim_end().to_string();

        let diff = if expected == actual {
            String::new()
        } else {
            similar::TextDiff::from_lines(&expected, &actual)
                .unified_diff()
                .header(
                    "optimizations/purity_analysis.egg (generated marker stripped)",
                    "purity_analysis::fragment() (generated marker stripped)",
                )
                .to_string()
        };

        insta::assert_snapshot!(diff, @"");
    }

    #[test]
    fn purity_analysis_fragment_contains_generated_prefix() {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/purity_analysis.rs)\n";

        let fragment = super::purity_analysis::fragment();
        let generated_prefix = fragment.as_str();

        assert!(
            generated_prefix.contains(GENERATED_MARKER),
            "purity_analysis::fragment() must mark the generated fragment"
        );

        for declaration in [
            "(relation ExprIsPure",
            "(relation ListExprIsPure",
            "(relation BinaryOpIsPure",
            "(relation UnaryOpIsPure",
            "(relation TernaryOpIsPure",
            "(BinaryOpIsPure (Add))",
            "(UnaryOpIsPure (Neg))",
            "(ExprIsPure (Function _name _tyin _tyout _out))",
            "(ExprIsPure (Const _n _ty _ctx))",
            "(ExprIsPure (Bop _op _x _y))",
            "(ExprIsPure lhs)",
            "(ExprIsPure (Call _f _arg))",
            "(ListExprIsPure (Cons _hd _tl))",
            "(ListExprIsPure (Nil))",
        ] {
            assert!(
                generated_prefix.contains(declaration),
                "Generated purity_analysis fragment must contain {declaration}"
            );
        }
    }

    #[test]
    fn add_context_fragment_matches_file_except_generated_sections() {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/add_context.rs)\n";

        let expected = include_str!("../utility/add_context.egg")
            .trim_end()
            .to_string();
        let actual = super::add_context::fragment();
        let actual = actual.replace(GENERATED_MARKER, "").trim_end().to_string();

        let diff = if expected == actual {
            String::new()
        } else {
            similar::TextDiff::from_lines(&expected, &actual)
                .unified_diff()
                .header(
                    "utility/add_context.egg (generated marker stripped)",
                    "add_context::fragment() (generated marker stripped)",
                )
                .to_string()
        };

        insta::assert_snapshot!(diff, @"");
    }

    #[test]
    fn add_context_fragment_contains_generated_prefix() {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/add_context.rs)\n";

        let fragment = super::add_context::fragment();
        let generated_prefix = fragment.as_str();

        assert!(
            generated_prefix.contains(GENERATED_MARKER),
            "add_context::fragment() must mark the generated fragment"
        );

        for declaration in [
            "(ruleset context)",
            "(constructor AddContext",
            "(constructor AddContextList",
            "(union lhs inner)",
            "(Arg ty ctx)",
            "(Const c ty ctx)",
            "(Empty ty ctx)",
            "(Top op",
            "(Bop op",
            "(Uop op (AddContext ctx c1))",
            "(Get (AddContext ctx c1) index)",
            "(Alloc id (AddContext ctx c1) (AddContext ctx state) ty)",
            "(Call name (AddContext ctx c1))",
            "(AddContextList ctx rest)",
            "(Switch (AddContext ctx pred)",
            "(If (AddContext ctx pred)",
            "(DoWhile",
        ] {
            assert!(
                generated_prefix.contains(declaration),
                "Generated add_context fragment must contain {declaration}"
            );
        }
    }

    #[test]
    fn context_prop_fragment_matches_empty_file_with_generated_marker_only() {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/context_prop.rs)\n";

        let expected = include_str!("../utility/context-prop.egg");
        let actual = super::context_prop::fragment();
        let actual = actual.replace(GENERATED_MARKER, "");

        assert!(
            expected.is_empty(),
            "utility/context-prop.egg must stay empty until real rules are migrated"
        );
        assert!(
            actual.is_empty(),
            "context_prop::fragment() must only emit the generated marker while the source file is empty"
        );
        assert_eq!(
            expected, actual,
            "context_prop::fragment() must match the empty source file once the generated marker is stripped"
        );
    }

    #[test]
    fn context_prop_fragment_contains_generated_prefix() {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/context_prop.rs)\n";

        let fragment = super::context_prop::fragment();
        assert_eq!(
            fragment, GENERATED_MARKER,
            "context_prop::fragment() must be marker-only while utility/context-prop.egg is empty"
        );
    }

    #[test]
    fn term_subst_fragment_matches_file_except_generated_sections() {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/term_subst.rs)\n";

        let expected = include_str!("../utility/term-subst.egg")
            .trim_end()
            .to_string();
        let actual = super::term_subst::fragment();
        let actual = actual.replace(GENERATED_MARKER, "").trim_end().to_string();

        let diff = if expected == actual {
            String::new()
        } else {
            similar::TextDiff::from_lines(&expected, &actual)
                .unified_diff()
                .header(
                    "utility/term-subst.egg (generated marker stripped)",
                    "term_subst::fragment() (generated marker stripped)",
                )
                .to_string()
        };

        insta::assert_snapshot!(diff, @"");
    }

    #[test]
    fn term_subst_fragment_contains_generated_prefix() {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/term_subst.rs)\n";

        let fragment = super::term_subst::fragment();
        let generated_prefix = fragment.as_str();

        assert!(
            generated_prefix.contains(GENERATED_MARKER),
            "term_subst::fragment() must mark the generated fragment"
        );

        for declaration in [
            "(ruleset term-subst)",
            "(constructor TermSubst",
            "(HasArgType lhs ty)",
            "(AddContext ctx e)",
            "(Const c newty ctx)",
            "(Empty newty ctx)",
            "(Top op (TermSubst ctx e t1)",
            "(Bop op (TermSubst ctx e t1)",
            "(Uop op (TermSubst ctx e t1))",
            "(Get (TermSubst ctx e t) idx)",
            "(Alloc id (TermSubst ctx e t1)",
            "(Call name (TermSubst ctx e t))",
            "(Single (TermSubst ctx e t))",
            "(Concat (TermSubst ctx e t1)",
            "; Control Flow",
        ] {
            assert!(
                generated_prefix.contains(declaration),
                "Generated term_subst fragment must contain {declaration}"
            );
        }
    }

    #[test]
    fn prologue_parses_migrated_type_analysis_declarations() {
        let program = format!(
            "{}\n(let __rlcr_type (TypeList-ith (TCons (IntT) (TNil)) 0))\n(set (TypeList-length (TLConcat (TNil) (TCons (IntT) (TNil)))) 1)\n(HasType (Arg (Base (IntT)) (InFunc \"DUMMY\")) (Base (IntT)))\n(ExpectType (Arg (Base (IntT)) (InFunc \"DUMMY\")) (Base (IntT)) \"ok\")\n(HasArgType (Arg (Base (IntT)) (InFunc \"DUMMY\")) (Base (IntT)))\n",
            crate::prologue()
        );

        let mut egraph = egglog::EGraph::default();
        egraph.parse_and_run_program(None, &program).unwrap();
    }

    #[test]
    fn prologue_runs_migrated_has_arg_type_propagation_rules() {
        let inner = "(Const (Int 7) (Base (StateT)) (InFunc \"DUMMY\"))";
        let expr = format!("(Uop (Neg) {inner})");
        let schedule = format!("(run-schedule {})", crate::schedule::types_and_indexing());
        let program = format!(
            "{}\n(let __rlcr_expr {expr})\n{schedule}\n(check (HasType __rlcr_expr (Base (IntT))))\n(check (HasArgType __rlcr_expr (Base (StateT))))\n",
            crate::prologue()
        );

        let mut egraph = egglog::EGraph::default();
        egraph.parse_and_run_program(None, &program).unwrap();
    }

    #[test]
    fn prologue_runs_migrated_type_analysis_primitives() {
        let int_const = "(Const (Int 7) (Base (StateT)) (InFunc \"DUMMY\"))";
        let empty = "(Empty (Base (BoolT)) (InFunc \"DUMMY\"))";
        let schedule = format!("(run-schedule {})", crate::schedule::types_and_indexing());
        let program = format!(
            "{}\n(let __rlcr_int {int_const})\n(let __rlcr_empty {empty})\n{schedule}\n(check (HasType __rlcr_int (Base (IntT))))\n(check (HasArgType __rlcr_int (Base (StateT))))\n(check (HasType __rlcr_empty (TupleT (TNil))))\n(check (HasArgType __rlcr_empty (Base (BoolT))))\n",
            crate::prologue()
        );

        let mut egraph = egglog::EGraph::default();
        egraph.parse_and_run_program(None, &program).unwrap();
    }

    #[test]
    fn prologue_runs_migrated_type_analysis_binary_ops() {
        let lhs = "(Const (Int 7) (Base (StateT)) (InFunc \"DUMMY\"))";
        let rhs = "(Const (Int 5) (Base (StateT)) (InFunc \"DUMMY\"))";
        let expr = format!("(Bop (Add) {lhs} {rhs})");
        let schedule = format!("(run-schedule {})", crate::schedule::types_and_indexing());
        let program = format!(
            "{}\n(let __rlcr_expr {expr})\n{schedule}\n(check (HasType __rlcr_expr (Base (IntT))))\n(check (HasArgType __rlcr_expr (Base (StateT))))\n",
            crate::prologue()
        );

        let mut egraph = egglog::EGraph::default();
        egraph.parse_and_run_program(None, &program).unwrap();
    }

    #[test]
    fn prologue_runs_migrated_type_analysis_other_ops() {
        let amount = "(Const (Int 4) (Base (StateT)) (InFunc \"DUMMY\"))";
        let state = "(Arg (Base (StateT)) (InFunc \"DUMMY\"))";
        let alloc = format!("(Alloc 0 {amount} {state} (IntT))");
        let expr = format!("(Get {alloc} 0)");
        let schedule = format!("(run-schedule {})", crate::schedule::types_and_indexing());
        let program = format!(
            "{}\n(let __rlcr_expr {expr})\n{schedule}\n(check (HasType __rlcr_expr (Base (IntT))))\n(check (HasArgType __rlcr_expr (Base (StateT))))\n",
            crate::prologue()
        );

        let mut egraph = egglog::EGraph::default();
        egraph.parse_and_run_program(None, &program).unwrap();
    }

    #[test]
    fn prologue_runs_migrated_type_analysis_tuple_operations() {
        let left = "(Single (Const (Int 7) (Base (StateT)) (InFunc \"DUMMY\")))";
        let right = "(Single (Const (Bool true) (Base (StateT)) (InFunc \"DUMMY\")))";
        let expr = format!("(Concat {left} {right})");
        let schedule = format!("(run-schedule {})", crate::schedule::types_and_indexing());
        let expected_ty = "(TupleT (TCons (IntT) (TCons (BoolT) (TNil))))";
        let program = format!(
            "{}\n(let __rlcr_expr {expr})\n{schedule}\n(check (HasType __rlcr_expr {expected_ty}))\n(check (HasArgType __rlcr_expr (Base (StateT))))\n",
            crate::prologue()
        );

        let mut egraph = egglog::EGraph::default();
        egraph.parse_and_run_program(None, &program).unwrap();
    }

    #[test]
    fn prologue_runs_migrated_type_analysis_control_flow() {
        let pred = "(Const (Bool true) (Base (StateT)) (InFunc \"DUMMY\"))";
        let inputs = "(Empty (TupleT (TNil)) (InFunc \"DUMMY\"))";
        let then_branch = "(Const (Int 7) (TupleT (TNil)) (InFunc \"DUMMY\"))";
        let else_branch = "(Const (Int 9) (TupleT (TNil)) (InFunc \"DUMMY\"))";
        let expr = format!("(If {pred} {inputs} {then_branch} {else_branch})");
        let schedule = format!("(run-schedule {})", crate::schedule::types_and_indexing());
        let program = format!(
            "{}\n(let __rlcr_expr {expr})\n{schedule}\n(check (HasType __rlcr_expr (Base (IntT))))\n(check (HasArgType __rlcr_expr (Base (StateT))))\n",
            crate::prologue()
        );

        let mut egraph = egglog::EGraph::default();
        egraph.parse_and_run_program(None, &program).unwrap();
    }

    #[test]
    fn prologue_runs_migrated_type_analysis_functions() {
        let arg = "(Const (Int 7) (Base (StateT)) (InFunc \"DUMMY\"))";
        let expr = "(Call \"callee\" (Const (Int 7) (Base (StateT)) (InFunc \"DUMMY\")))";
        let schedule = format!("(run-schedule {})", crate::schedule::types_and_indexing());
        let program = format!(
            "{}\n(FunctionHasType \"callee\" (Base (IntT)) (Base (BoolT)))\n(let __rlcr_arg {arg})\n(let __rlcr_expr {expr})\n{schedule}\n(check (HasType __rlcr_expr (Base (BoolT))))\n(check (HasArgType __rlcr_expr (Base (StateT))))\n",
            crate::prologue()
        );

        let mut egraph = egglog::EGraph::default();
        egraph.parse_and_run_program(None, &program).unwrap();
    }

    #[test]
    fn prologue_runs_migrated_type_analysis_pure_type_tail() {
        let tylist = "(TCons (IntT) (TNil))";
        let tuple_ty = format!("(TupleT {tylist})");
        let schedule = format!("(run-schedule {})", crate::schedule::types_and_indexing());
        let program = format!(
            "{}\n(let __rlcr_tuple_ty {tuple_ty})\n(let __rlcr_tylist {tylist})\n{schedule}\n(check (PureBaseType (BoolT)))\n(check (PureType __rlcr_tuple_ty))\n(check (PureTypeList __rlcr_tylist))\n",
            crate::prologue()
        );

        let mut egraph = egglog::EGraph::default();
        egraph.parse_and_run_program(None, &program).unwrap();
    }

    #[test]
    fn prologue_runs_migrated_util_prefix_rules() {
        let left = "(Single (Const (Int 7) (Base (StateT)) (InFunc \"DUMMY\")))";
        let right = "(Single (Const (Bool true) (Base (StateT)) (InFunc \"DUMMY\")))";
        let expr = format!("(Concat {left} {right})");
        let schedule = format!("(run-schedule {})", crate::schedule::types_and_indexing());
        let program = format!(
            "{}\n(let __rlcr_expr {expr})\n{schedule}\n(check (= (tuple-length __rlcr_expr) 2))\n",
            crate::prologue()
        );

        let mut egraph = egglog::EGraph::default();
        egraph.parse_and_run_program(None, &program).unwrap();
    }

    #[test]
    fn prologue_runs_migrated_util_leading_get_rules() {
        let tuple_ty = "(TupleT (TCons (IntT) (TCons (BoolT) (TNil))))";
        let cond = "(Const (Bool true) (Base (StateT)) (InFunc \"DUMMY\"))";
        let left = "(Single (Const (Int 7) (Base (StateT)) (InFunc \"DUMMY\")))";
        let right = "(Single (Const (Bool false) (Base (StateT)) (InFunc \"DUMMY\")))";
        let inputs = format!("(Concat {left} {right})");
        let then_branch = format!("(Const (Int 1) {tuple_ty} (InFunc \"DUMMY\"))");
        let else_branch = format!("(Const (Int 2) {tuple_ty} (InFunc \"DUMMY\"))");
        let schedule = format!("(run-schedule {})", crate::schedule::types_and_indexing());
        let program = format!(
            "{}\n(let __rlcr_inputs {inputs})\n(let __rlcr_if (If {cond} __rlcr_inputs {then_branch} {else_branch}))\n{schedule}\n(check (= (Get __rlcr_inputs 0) (Const (Int 7) (Base (StateT)) (InFunc \"DUMMY\"))))\n(check (= (Get __rlcr_inputs 1) (Const (Bool false) (Base (StateT)) (InFunc \"DUMMY\"))))\n",
            crate::prologue()
        );

        let mut egraph = egglog::EGraph::default();
        egraph.parse_and_run_program(None, &program).unwrap();
    }

    #[test]
    fn prologue_runs_migrated_util_tmp_ctx_rules() {
        let tuple_ty = "(TupleT (TCons (IntT) (TCons (IntT) (TNil))))";
        let inputs = "(Concat (Single (Const (Int 7) (Base (StateT)) (InFunc \"DUMMY\"))) (Single (Const (Int 9) (Base (StateT)) (InFunc \"DUMMY\"))))";
        let body = format!(
            "(Concat (Single (Const (Bool true) {tuple_ty} (TmpCtx))) (Arg {tuple_ty} (TmpCtx)))"
        );
        let schedule = format!("(run-schedule {})", crate::schedule::types_and_indexing());
        let program = format!(
            "{}\n(let __rlcr_inputs {inputs})\n(let __rlcr_body {body})\n(let __rlcr_loop (DoWhile __rlcr_inputs __rlcr_body))\n(union (TmpCtx) (InLoop __rlcr_inputs __rlcr_body))\n(delete (TmpCtx))\n{schedule}\n(check (= (Get __rlcr_inputs 0) (Const (Int 7) (Base (StateT)) (InFunc \"DUMMY\"))))\n(check (= (Get __rlcr_inputs 1) (Const (Int 9) (Base (StateT)) (InFunc \"DUMMY\"))))\n",
            crate::prologue()
        );

        let mut egraph = egglog::EGraph::default();
        egraph.parse_and_run_program(None, &program).unwrap();
    }

    #[test]
    fn prologue_runs_migrated_util_subsume_after_helpers_rules() {
        let cond = "(Const (Bool true) (Base (StateT)) (InFunc \"DUMMY\"))";
        let inputs = "(Empty (TupleT (TNil)) (InFunc \"DUMMY\"))";
        let then_branch = "(Const (Int 1) (TupleT (TNil)) (InFunc \"DUMMY\"))";
        let else_branch = "(Const (Int 2) (TupleT (TNil)) (InFunc \"DUMMY\"))";
        let if_expr = format!("(If {cond} {inputs} {then_branch} {else_branch})");
        let program = format!(
            "{}\n(let __rlcr_if {if_expr})\n(ToSubsumeIf {cond} {inputs} {then_branch} {else_branch})\n(run-schedule subsume-after-helpers)\n",
            crate::prologue()
        );

        let mut egraph = egglog::EGraph::default();
        egraph.parse_and_run_program(None, &program).unwrap();

        let (serialized, _) = crate::greedy_dag_extractor::serialized_egraph(egraph);
        assert!(
            serialized
                .nodes
                .values()
                .any(|node| node.op == "If" && node.subsumed),
            "subsume-after-helpers should mark the target If node as subsumed"
        );
    }

    #[test]
    fn prologue_runs_migrated_terms_prefix_rules() {
        let left = "(Single (Const (Int 7) (Base (StateT)) (InFunc \"DUMMY\")))";
        let right = "(Single (Const (Bool true) (Base (StateT)) (InFunc \"DUMMY\")))";
        let expr = format!("(Concat {left} {right})");
        let schedule =
            "(run-schedule (saturate terms (saturate terms-helpers (saturate terms-helpers-helpers))))";
        let program = format!(
            "{}\n(let __rlcr_expr {expr})\n{schedule}\n(check (= (ExtractedExpr __rlcr_expr) (TCPair (TermConcat (TermSingle (TermConst (Int 7))) (TermSingle (TermConst (Bool true)))) 2)))\n",
            crate::prologue()
        );

        let mut egraph = egglog::EGraph::default();
        egraph.parse_and_run_program(None, &program).unwrap();
    }

    #[test]
    fn prologue_runs_migrated_terms_if_node_tail() {
        let cond = "(Const (Bool true) (Base (StateT)) (InFunc \"DUMMY\"))";
        let inputs = "(Empty (TupleT (TNil)) (InFunc \"DUMMY\"))";
        let then_branch = "(Const (Int 1) (TupleT (TNil)) (InFunc \"DUMMY\"))";
        let else_branch = "(Const (Int 2) (TupleT (TNil)) (InFunc \"DUMMY\"))";
        let if_expr = format!("(If {cond} {inputs} {then_branch} {else_branch})");
        let schedule =
            "(run-schedule (saturate terms (saturate terms-helpers (saturate terms-helpers-helpers))))";
        let program = format!(
            "{}\n(let __rlcr_if {if_expr})\n(let __rlcr_if_node (IfNode __rlcr_if {cond} {inputs} {then_branch} {else_branch}))\n{schedule}\n(check (= __rlcr_if_node (IfNode __rlcr_if {cond} {inputs} {then_branch} {else_branch})))\n",
            crate::prologue()
        );

        let mut egraph = egglog::EGraph::default();
        egraph.parse_and_run_program(None, &program).unwrap();
    }

    #[test]
    fn prologue_runs_migrated_purity_analysis_rules() {
        let pure_expr = "(Bop (Add) (Const (Int 7) (Base (IntT)) (InFunc \"DUMMY\")) (Const (Int 5) (Base (IntT)) (InFunc \"DUMMY\")))";
        let impure_expr = "(Arg (Base (StateT)) (InFunc \"DUMMY\"))";
        let schedule = format!("(run-schedule {})", crate::schedule::types_and_indexing());
        let program = format!(
            "{}\n(let __rlcr_pure {pure_expr})\n(let __rlcr_impure {impure_expr})\n{schedule}\n(check (ExprIsPure __rlcr_pure))\n(fail (check (ExprIsPure __rlcr_impure)))\n",
            crate::prologue()
        );

        let mut egraph = egglog::EGraph::default();
        egraph.parse_and_run_program(None, &program).unwrap();
    }

    #[test]
    fn prologue_runs_migrated_add_context_rules() {
        let ctx = "(InFunc \"RLCR\")";
        let expr = "(Concat (Single (Const (Int 3) (Base (IntT)) (InFunc \"OLD\"))) (Single (Const (Int 4) (Base (IntT)) (InFunc \"OLD\"))))";
        let expected = "(Concat (Single (Const (Int 3) (Base (IntT)) (InFunc \"RLCR\"))) (Single (Const (Int 4) (Base (IntT)) (InFunc \"RLCR\"))))";
        let program = format!(
            "{}\n(let __rlcr_expr {expr})\n(let __rlcr_ctx {ctx})\n(let __rlcr_added (AddContext __rlcr_ctx __rlcr_expr))\n(run-schedule (saturate context))\n(check (= __rlcr_added {expected}))\n",
            crate::prologue()
        );

        let mut egraph = egglog::EGraph::default();
        egraph.parse_and_run_program(None, &program).unwrap();
    }

    #[test]
    fn prologue_runs_migrated_term_subst_rules() {
        let ctx = "(InFunc \"RLCR\")";
        let expr = "(Const (Int 3) (Base (IntT)) (InFunc \"OLD\"))";
        let term = "(TermConcat (TermSingle (TermArg)) (TermSingle (TermConst (Int 9))))";
        let expected = "(Concat (Single (Const (Int 3) (Base (IntT)) (InFunc \"RLCR\"))) (Single (Const (Int 9) (Base (IntT)) (InFunc \"RLCR\"))))";
        let program = format!(
            "{}\n(let __rlcr_ctx {ctx})\n(let __rlcr_expr {expr})\n(let __rlcr_term {term})\n(let __rlcr_subst (TermSubst __rlcr_ctx __rlcr_expr __rlcr_term))\n(run-schedule {})\n(run-schedule (saturate term-subst))\n(run-schedule (saturate context))\n(check (= __rlcr_subst {expected}))\n",
            crate::prologue(),
            crate::schedule::types_and_indexing()
        );

        let mut egraph = egglog::EGraph::default();
        egraph.parse_and_run_program(None, &program).unwrap();
    }

    #[test]
    fn prologue_matches_text_backend_except_generated_schema_and_type_analysis_sections() {
        let expected = crate::prologue_egglog_text();
        let actual = crate::prologue();

        let expected = strip_schema_generated_sections(&expected);
        let actual = strip_schema_generated_sections(&actual);
        let expected = strip_type_analysis_generated_sections(&expected);
        let actual = strip_type_analysis_generated_sections(&actual);
        let expected = strip_util_generated_sections(&expected);
        let actual = strip_util_generated_sections(&actual);
        let expected = strip_terms_generated_sections(&expected);
        let actual = strip_terms_generated_sections(&actual);
        let expected = strip_purity_analysis_generated_sections(&expected);
        let actual = strip_purity_analysis_generated_sections(&actual);
        let expected = strip_add_context_generated_sections(&expected);
        let actual = strip_add_context_generated_sections(&actual);
        let expected = strip_context_prop_generated_sections(&expected);
        let actual = strip_context_prop_generated_sections(&actual);
        let expected = strip_term_subst_generated_sections(&expected);
        let actual = strip_term_subst_generated_sections(&actual);

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
