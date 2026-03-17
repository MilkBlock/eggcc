mod add_context;
mod canonicalize;
mod conditional_invariant_code_motion;
mod conditional_push_in;
mod context_of;
mod context_prop;
mod debug_helper;
mod drop_at;
mod expr_size;
mod interval_analysis;
mod ivt;
mod loop_invariant;
mod loop_simplify;
mod loop_strength_reduction;
mod loop_unroll;
mod mem_simple;
mod memory;
mod passthrough;
mod peepholes;
mod purity_analysis;
mod rec_to_loop;
mod schema;
mod schema_dsl;
mod select;
mod subst;
mod swap_if;
mod switch_rewrites;
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
        &context_of::fragment(),
        &subst::fragment(),
        &canonicalize::fragment(),
        &expr_size::fragment(),
        &drop_at::fragment(),
        &interval_analysis::fragment(),
        &switch_rewrites::fragment(),
        &select::fragment(),
        &peepholes::fragment(),
        &crate::optimizations::memory::rules(),
        &memory::fragment(),
        &mem_simple::fragment(),
        &loop_invariant::rules(),
        &loop_simplify::fragment(),
        &loop_unroll::fragment(),
        &swap_if::fragment(),
        &rec_to_loop::fragment(),
        &passthrough::fragment(),
        &loop_strength_reduction::fragment(),
        &ivt::fragment(),
        &conditional_invariant_code_motion::fragment(),
        &conditional_push_in::fragment(),
        &debug_helper::fragment(),
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

    fn strip_context_of_generated_sections(program: &str) -> String {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/context_of.rs)\n";
        const SECTION_HEADER: &str = "; We only have context for Exprs, not ListExprs.\n";
        const NEXT_SECTION_HEADER: &str =
            ";; Substitution rules allow for substituting some new expression for the argument\n";

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

    fn strip_subst_generated_sections(program: &str) -> String {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/subst.rs)\n";
        const SECTION_HEADER: &str =
            ";; Substitution rules allow for substituting some new expression for the argument\n";
        const NEXT_SECTION_HEADER: &str = "(ruleset canon)\n";

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

    fn strip_canonicalize_generated_sections(program: &str) -> String {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/canonicalize.rs)\n";
        const SECTION_HEADER: &str = "(ruleset canon)\n";
        const NEXT_SECTION_HEADER: &str = ";; Compute the tree size of program, not dag size\n";

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

    fn strip_expr_size_generated_sections(program: &str) -> String {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/expr_size.rs)\n";
        const SECTION_HEADER: &str = ";; Compute the tree size of program, not dag size\n";
        const NEXT_SECTION_HEADER: &str = ";; Like Subst but for dropping inputs to a region\n";

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

    fn strip_drop_at_generated_sections(program: &str) -> String {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/drop_at.rs)\n";
        const SECTION_HEADER: &str = ";; Like Subst but for dropping inputs to a region\n";
        const NEXT_SECTION_HEADER: &str = "(ruleset interval-analysis)\n";

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

    fn strip_interval_analysis_generated_sections(program: &str) -> String {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/interval_analysis.rs)\n";
        const SECTION_HEADER: &str = "(ruleset interval-analysis)\n";
        const NEXT_SECTION_HEADER: &str = "(ruleset switch_rewrite)\n";

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

    fn strip_switch_rewrites_generated_sections(program: &str) -> String {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/switch_rewrites.rs)\n";
        const SECTION_HEADER: &str = "(ruleset switch_rewrite)\n";
        const NEXT_SECTION_HEADER: &str = "(ruleset select_opt)\n";

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

    fn strip_select_generated_sections(program: &str) -> String {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/select.rs)\n";
        const SECTION_HEADER: &str = "(ruleset select_opt)\n";
        const NEXT_SECTION_HEADER: &str =
            "; Simple rewrites that don't do a ton with control flow.\n";

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

    fn strip_peepholes_generated_sections(program: &str) -> String {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/peepholes.rs)\n";
        const SECTION_HEADER: &str = "; Simple rewrites that don't do a ton with control flow.\n";
        const NEXT_SECTION_HEADER: &str = "(datatype IntOrInfinity\n";

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

    fn strip_memory_generated_sections(program: &str) -> String {
        const RAW_START: &str = "(sort ExprSetPrim (Set Expr))\n";
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/memory.rs)\n";
        const NEXT_SECTION_HEADER: &str = "(ruleset mem-simple)\n";

        let start = program
            .find(GENERATED_MARKER)
            .or_else(|| program.find(RAW_START))
            .expect("memory.egg fragment must contain the generated marker or raw start");
        let end = program[start..]
            .find(NEXT_SECTION_HEADER)
            .map(|idx| idx + start)
            .unwrap_or(program.len());

        let mut stripped = String::new();
        stripped.push_str(&program[..start]);
        stripped.push_str(&program[end..]);

        while stripped.contains("\n\n\n") {
            stripped = stripped.replace("\n\n\n", "\n\n");
        }

        stripped = stripped.replace(
            &format!("\n\n{NEXT_SECTION_HEADER}"),
            &format!("\n{NEXT_SECTION_HEADER}"),
        );

        stripped
    }

    fn strip_mem_simple_generated_sections(program: &str) -> String {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/mem_simple.rs)\n";
        const SECTION_HEADER: &str = "(ruleset mem-simple)\n";
        const NEXT_SECTION_HEADER: &str = ";; Loop Invariant\n";

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

    fn strip_loop_invariant_generated_sections(program: &str) -> String {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/loop_invariant.rs)\n";
        const SECTION_HEADER: &str = ";; Loop Invariant\n";

        let mut stripped = program.replace(GENERATED_MARKER, "");
        while stripped.contains("\n\n\n") {
            stripped = stripped.replace("\n\n\n", "\n\n");
        }
        stripped = stripped.replace(
            &format!("\n\n{SECTION_HEADER}"),
            &format!("\n{SECTION_HEADER}"),
        );
        stripped
    }

    fn strip_loop_simplify_generated_sections(program: &str) -> String {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/loop_simplify.rs)\n";
        const NEXT_SECTION_HEADER: &str = ";; Some simple simplifications of loops\n";

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

    fn strip_loop_unroll_generated_sections(program: &str) -> String {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/loop_unroll.rs)\n";
        const SECTION_HEADER: &str = ";; Some simple simplifications of loops\n";
        const NEXT_SECTION_HEADER: &str = "(ruleset swap-if)\n";

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

    fn strip_swap_if_generated_sections(program: &str) -> String {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/swap_if.rs)\n";
        const SECTION_HEADER: &str = "(ruleset swap-if)\n";
        const NEXT_SECTION_HEADER: &str = ";; this ruleset depends on swap_if running twice\n";

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

    fn strip_rec_to_loop_generated_sections(program: &str) -> String {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/rec_to_loop.rs)\n";
        const SECTION_HEADER: &str = ";; this ruleset depends on swap_if running twice\n";
        const NEXT_SECTION_HEADER: &str = "(ruleset passthrough)\n";

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

    fn strip_passthrough_generated_sections(program: &str) -> String {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/passthrough.rs)\n";
        const SECTION_HEADER: &str = "(ruleset passthrough)\n";
        const NEXT_SECTION_HEADER: &str = "(ruleset loop-strength-reduction)\n";

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

    fn strip_loop_strength_reduction_generated_sections(program: &str) -> String {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/loop_strength_reduction.rs)\n";
        const SECTION_HEADER: &str = ";; ORIGINAL\n";
        const NEXT_SECTION_HEADER: &str = "(relation IVTNewInputsAnalysisDemand (Expr))\n";

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

    fn strip_ivt_generated_sections(program: &str) -> String {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/ivt.rs)\n";
        const SECTION_HEADER: &str = "(relation IVTNewInputsAnalysisDemand (Expr))\n";
        const NEXT_SECTION_HEADER: &str = "(ruleset cicm)\n";

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

    fn strip_conditional_invariant_code_motion_generated_sections(program: &str) -> String {
        const GENERATED_MARKER: &str = "; (Generated from eggplant Rust: src/eggplant_backend/conditional_invariant_code_motion.rs)\n";
        const SECTION_HEADER: &str = "(ruleset cicm)\n";
        const NEXT_SECTION_HEADER: &str = "(ruleset push-in)\n";

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

    fn strip_conditional_push_in_generated_sections(program: &str) -> String {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/conditional_push_in.rs)\n";
        const SECTION_HEADER: &str = "(ruleset push-in)\n";
        const NEXT_SECTION_HEADER: &str =
            ";; use these rules to clean up the database, removing helpers\n";

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

    fn strip_debug_helper_generated_sections(program: &str) -> String {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/debug_helper.rs)\n";
        const SECTION_HEADER: &str =
            ";; use these rules to clean up the database, removing helpers\n";
        const NEXT_SECTION_HEADER: &str = ";; Hacker's delight optimizations\n";

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
            generated_prefix.starts_with(GENERATED_MARKER),
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
            generated_prefix.starts_with(GENERATED_MARKER),
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
            generated_prefix.starts_with(GENERATED_MARKER),
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
            generated_prefix.starts_with(GENERATED_MARKER),
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
            generated_prefix.starts_with(GENERATED_MARKER),
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
            generated_prefix.starts_with(GENERATED_MARKER),
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
    fn context_of_fragment_matches_file_except_generated_sections() {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/context_of.rs)\n";

        let expected = include_str!("../utility/context_of.egg")
            .trim_end()
            .to_string();
        let actual = super::context_of::fragment();
        let actual = actual.replace(GENERATED_MARKER, "").trim_end().to_string();

        let diff = if expected == actual {
            String::new()
        } else {
            similar::TextDiff::from_lines(&expected, &actual)
                .unified_diff()
                .header(
                    "utility/context_of.egg (generated marker stripped)",
                    "context_of::fragment() (generated marker stripped)",
                )
                .to_string()
        };

        insta::assert_snapshot!(diff, @"");
    }

    #[test]
    fn context_of_fragment_contains_generated_prefix() {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/context_of.rs)\n";

        let fragment = super::context_of::fragment();
        let generated_prefix = fragment.as_str();

        assert!(
            generated_prefix.starts_with(GENERATED_MARKER),
            "context_of::fragment() must mark the generated fragment"
        );

        for declaration in [
            "(relation ContextOf",
            "(ContextOf (Arg ty ctx) ctx)",
            "(ContextOf (Const c ty ctx) ctx)",
            "(ContextOf (Empty ty ctx) ctx)",
            "(panic \"Equivalent expressions have nonequivalent context",
            "(ContextOf (Top op x y z) ctx)",
            "(ContextOf (Bop op x y) ctx)",
            "(ContextOf (Uop op x) ctx)",
            "(ContextOf (Get tup i) ctx)",
            "(ContextOf (Concat x y) ctx)",
            "(ContextOf (Single x) ctx)",
            "(ContextOf (Switch pred inputs branches) ctx)",
            "(ContextOf (If pred inputs then else) ctx)",
            "(ContextOf (DoWhile in pred-and-output) ctx)",
            "(ContextOf (Call func arg) ctx)",
            "(ContextOf (Alloc amt e state ty) ctx)",
        ] {
            assert!(
                generated_prefix.contains(declaration),
                "Generated context_of fragment must contain {declaration}"
            );
        }
    }

    #[test]
    fn subst_fragment_matches_file_except_generated_sections() {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/subst.rs)\n";

        let expected = include_str!("../utility/subst.egg").trim_end().to_string();
        let actual = super::subst::fragment();
        let actual = actual.replace(GENERATED_MARKER, "").trim_end().to_string();

        let diff = if expected == actual {
            String::new()
        } else {
            similar::TextDiff::from_lines(&expected, &actual)
                .unified_diff()
                .header(
                    "utility/subst.egg (generated marker stripped)",
                    "subst::fragment() (generated marker stripped)",
                )
                .to_string()
        };

        insta::assert_snapshot!(diff, @"");
    }

    #[test]
    fn subst_fragment_contains_generated_prefix() {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/subst.rs)\n";

        let fragment = super::subst::fragment();
        let generated_prefix = fragment.as_str();

        assert!(
            generated_prefix.starts_with(GENERATED_MARKER),
            "subst::fragment() must mark the generated fragment"
        );

        for declaration in [
            "(ruleset subst)",
            "(ruleset apply-subst-unions)",
            "(constructor Subst",
            "(constructor IfSubst",
            "(If (Const (Bool true) ty ctx)",
            "(constructor DelayedSubstUnion",
            "(HasArgType lhs ty)",
            "(panic \"Substitution type mismatch!",
            "(DelayedSubstUnion lhs (AddContext assum to))",
            "(Const c newty assum)",
            "(Empty newty assum)",
            "(Top op (Subst assum to c1)",
            "(Bop op (Subst assum to c1)",
            "(Uop op (Subst assum to c1))",
            "(Get (Subst assum to c1) index)",
            "(Alloc id (Subst assum to c1)",
            "(Call name (Subst assum to c1))",
            "(Single (Subst assum to c1))",
            "(Concat (Subst assum to c1)",
            "(Switch (Subst assum to pred)",
            "(If (Subst assum to pred)",
            "(DoWhile (Subst assum to in)",
            "(Function name inty outty (Subst assum to body))",
            "(union lhs rhs)",
        ] {
            assert!(
                generated_prefix.contains(declaration),
                "Generated subst fragment must contain {declaration}"
            );
        }
    }

    #[test]
    fn canonicalize_fragment_matches_file_except_generated_sections() {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/canonicalize.rs)\n";

        let expected = include_str!("../utility/canonicalize.egg")
            .trim_end()
            .to_string();
        let actual = super::canonicalize::fragment();
        let actual = actual.replace(GENERATED_MARKER, "").trim_end().to_string();

        let diff = if expected == actual {
            String::new()
        } else {
            similar::TextDiff::from_lines(&expected, &actual)
                .unified_diff()
                .header(
                    "utility/canonicalize.egg (generated marker stripped)",
                    "canonicalize::fragment() (generated marker stripped)",
                )
                .to_string()
        };

        insta::assert_snapshot!(diff, @"");
    }

    #[test]
    fn canonicalize_fragment_contains_generated_prefix() {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/canonicalize.rs)\n";

        let fragment = super::canonicalize::fragment();
        let generated_prefix = fragment.as_str();

        assert!(
            generated_prefix.starts_with(GENERATED_MARKER),
            "canonicalize::fragment() must mark the generated fragment"
        );

        for declaration in [
            "(ruleset canon)",
            "(Bop (GreaterThan) x y)",
            "(Bop (LessThan) y x)",
            "(constructor SubTuple",
            "(SubTuple lhs 0 size)",
            "(Single (Get expr x))",
            "(constructor TupleRemoveAt",
            "(panic \"Index out of bounds for TupleRemoveAt\")",
            "(constructor TypeListRemoveAt",
            "(panic \"Index out of bounds for TypeListRemoveAt.\")",
        ] {
            assert!(
                generated_prefix.contains(declaration),
                "Generated canonicalize fragment must contain {declaration}"
            );
        }
    }

    #[test]
    fn expr_size_fragment_matches_file_except_generated_sections() {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/expr_size.rs)\n";

        let expected = include_str!("../utility/expr_size.egg")
            .trim_end()
            .to_string();
        let actual = super::expr_size::fragment();
        let actual = actual.replace(GENERATED_MARKER, "").trim_end().to_string();

        let diff = if expected == actual {
            String::new()
        } else {
            similar::TextDiff::from_lines(&expected, &actual)
                .unified_diff()
                .header(
                    "utility/expr_size.egg (generated marker stripped)",
                    "expr_size::fragment() (generated marker stripped)",
                )
                .to_string()
        };

        insta::assert_snapshot!(diff, @"");
    }

    #[test]
    fn expr_size_fragment_contains_generated_prefix() {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/expr_size.rs)\n";

        let fragment = super::expr_size::fragment();
        let generated_prefix = fragment.as_str();

        assert!(
            generated_prefix.starts_with(GENERATED_MARKER),
            "expr_size::fragment() must mark the generated fragment"
        );

        for declaration in [
            "(function Expr-size (Expr) i64 :merge (min old new) )",
            "(function ListExpr-size (ListExpr) i64 :merge (min old new))",
            "(set (Expr-size expr) (+ sum 1))",
            "(set (Expr-size expr) sum)",
            "(set (Expr-size (Empty ty assum)) 0)",
            "(set (ListExpr-size expr) sum)",
            "(set (ListExpr-size (Nil)) 0)",
            "(= expr (Alloc id e state ty))",
        ] {
            assert!(
                generated_prefix.contains(declaration),
                "Generated expr_size fragment must contain {declaration}"
            );
        }
    }

    #[test]
    fn drop_at_fragment_matches_file_except_generated_sections() {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/drop_at.rs)\n";

        let expected = include_str!("../utility/drop_at.egg")
            .trim_end()
            .to_string();
        let actual = super::drop_at::fragment();
        let actual = actual.replace(GENERATED_MARKER, "").trim_end().to_string();

        let diff = if expected == actual {
            String::new()
        } else {
            similar::TextDiff::from_lines(&expected, &actual)
                .unified_diff()
                .header(
                    "utility/drop_at.egg (generated marker stripped)",
                    "drop_at::fragment() (generated marker stripped)",
                )
                .to_string()
        };

        insta::assert_snapshot!(diff, @"");
    }

    #[test]
    fn drop_at_fragment_contains_generated_prefix() {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/drop_at.rs)\n";

        let fragment = super::drop_at::fragment();
        let generated_prefix = fragment.as_str();

        assert!(
            generated_prefix.starts_with(GENERATED_MARKER),
            "drop_at::fragment() must mark the generated fragment"
        );

        for declaration in [
            "(ruleset drop)",
            "(ruleset apply-drop-unions)",
            "(ruleset cleanup-drop)",
            "(constructor DropAt",
            "(constructor DelayedDropUnion",
            "(constructor DropAtInternal",
            "(TypeListRemoveAt oldty idx)",
            "(Get (Arg newty newctx) (- i 1))",
            "(DropAtInternal newty newctx idx c1)",
            "(subsume (DropAt newctx idx in))",
            "(subsume (DropAtInternal newty newctx idx in))",
            "(subsume (DelayedDropUnion lhs rhs))",
        ] {
            assert!(
                generated_prefix.contains(declaration),
                "Generated drop_at fragment must contain {declaration}"
            );
        }
    }

    #[test]
    fn interval_analysis_fragment_matches_file_except_generated_sections() {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/interval_analysis.rs)\n";

        let expected = include_str!("../interval_analysis.egg")
            .trim_end()
            .to_string();
        let actual = super::interval_analysis::fragment();
        let actual = actual.replace(GENERATED_MARKER, "").trim_end().to_string();

        let diff = if expected == actual {
            String::new()
        } else {
            similar::TextDiff::from_lines(&expected, &actual)
                .unified_diff()
                .header(
                    "interval_analysis.egg (generated marker stripped)",
                    "interval_analysis::fragment() (generated marker stripped)",
                )
                .to_string()
        };

        insta::assert_snapshot!(diff, @"");
    }

    #[test]
    fn interval_analysis_fragment_contains_generated_prefix() {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/interval_analysis.rs)\n";

        let fragment = super::interval_analysis::fragment();
        let generated_prefix = fragment.as_str();

        assert!(
            generated_prefix.starts_with(GENERATED_MARKER),
            "interval_analysis::fragment() must mark the generated fragment"
        );

        for declaration in [
            "(ruleset interval-analysis)",
            "(ruleset interval-rewrite)",
            "(datatype Bound",
            "(function lo-bound (Expr) Bound :merge (bound-max old new))",
            "(function hi-bound (Expr) Bound :merge (bound-min old new))",
            "(union expr (Const (Int x) ty ctx))",
            "(set (lo-bound lhs) (IntB (+ la lb)))",
            "(set (hi-bound lhs) (BoolB (bool-< la hb)))",
            "(union lhs (Subst if_ctx inputs thn))",
            "(set (lo-bound (Get ctx i)) lo)",
        ] {
            assert!(
                generated_prefix.contains(declaration),
                "Generated interval_analysis fragment must contain {declaration}"
            );
        }
    }

    #[test]
    fn switch_rewrites_fragment_matches_file_except_generated_sections() {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/switch_rewrites.rs)\n";

        let expected = include_str!("../optimizations/switch_rewrites.egg")
            .trim_end()
            .to_string();
        let actual = super::switch_rewrites::fragment();
        let actual = actual.replace(GENERATED_MARKER, "").trim_end().to_string();

        let diff = if expected == actual {
            String::new()
        } else {
            similar::TextDiff::from_lines(&expected, &actual)
                .unified_diff()
                .header(
                    "optimizations/switch_rewrites.egg (generated marker stripped)",
                    "switch_rewrites::fragment() (generated marker stripped)",
                )
                .to_string()
        };

        insta::assert_snapshot!(diff, @"");
    }

    #[test]
    fn switch_rewrites_fragment_contains_generated_prefix() {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/switch_rewrites.rs)\n";

        let fragment = super::switch_rewrites::fragment();
        let generated_prefix = fragment.as_str();

        assert!(
            generated_prefix.starts_with(GENERATED_MARKER),
            "switch_rewrites::fragment() must mark the generated fragment"
        );

        for declaration in [
            "(ruleset switch_rewrite)",
            "(ruleset always-switch-rewrite)",
            "(union (Get if_e k) (Bop (Smin) a b))",
            "(union (Get if_e k) (Bop (Smax) a b))",
            "(union (Get if_e k) (Top (Select) pred a b))",
            "(union (Get if_e i) (Top (Select) pred (Const x ty ctx) (Const y ty ctx)))",
            "(union (Get if_e k) (Top (Select) pred a (Const (Int y) ty ctx)))",
            "(union (Get if_e k) (Top (Select) pred (Const (Int y) ty ctx) b))",
            "(let outer_ins (Concat (Single b) ins))",
            "(let inner (If inner_pred sub_arg_false inner_X inner_Y))",
        ] {
            assert!(
                generated_prefix.contains(declaration),
                "Generated switch_rewrites fragment must contain {declaration}"
            );
        }
    }

    #[test]
    fn select_fragment_matches_file_except_generated_sections() {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/select.rs)\n";

        let expected = include_str!("../optimizations/select.egg")
            .trim_end()
            .to_string();
        let actual = super::select::fragment();
        let actual = actual.replace(GENERATED_MARKER, "").trim_end().to_string();

        let diff = if expected == actual {
            String::new()
        } else {
            similar::TextDiff::from_lines(&expected, &actual)
                .unified_diff()
                .header(
                    "optimizations/select.egg (generated marker stripped)",
                    "select::fragment() (generated marker stripped)",
                )
                .to_string()
        };

        insta::assert_snapshot!(diff, @"");
    }

    #[test]
    fn select_fragment_contains_generated_prefix() {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/select.rs)\n";

        let fragment = super::select::fragment();
        let generated_prefix = fragment.as_str();

        assert!(
            generated_prefix.starts_with(GENERATED_MARKER),
            "select::fragment() must mark the generated fragment"
        );

        for declaration in [
            "(ruleset select_opt)",
            "(ExprIsPure (Get thn i))",
            "(ExprIsPure (Get els i))",
            "(> 10 (Expr-size (Get thn i)))",
            "(= (TCPair t1 c1) (ExtractedExpr (Get thn i)))",
            "(= (TCPair t2 c2) (ExtractedExpr (Get els i)))",
            "(ContextOf if_e ctx)",
            "(Top (Select) pred (TermSubst ctx inputs t1) (TermSubst ctx inputs t2))",
        ] {
            assert!(
                generated_prefix.contains(declaration),
                "Generated select fragment must contain {declaration}"
            );
        }
    }

    #[test]
    fn peepholes_fragment_matches_file_except_generated_sections() {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/peepholes.rs)\n";

        let expected = include_str!("../optimizations/peepholes.egg")
            .trim_end()
            .to_string();
        let actual = super::peepholes::fragment();
        let actual = actual.replace(GENERATED_MARKER, "").trim_end().to_string();

        let diff = if expected == actual {
            String::new()
        } else {
            similar::TextDiff::from_lines(&expected, &actual)
                .unified_diff()
                .header(
                    "optimizations/peepholes.egg (generated marker stripped)",
                    "peepholes::fragment() (generated marker stripped)",
                )
                .to_string()
        };

        insta::assert_snapshot!(diff, @"");
    }

    #[test]
    fn peepholes_fragment_contains_generated_prefix() {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/peepholes.rs)\n";

        let fragment = super::peepholes::fragment();
        let generated_prefix = fragment.as_str();

        assert!(
            generated_prefix.starts_with(GENERATED_MARKER),
            "peepholes::fragment() must mark the generated fragment"
        );

        for declaration in [
            "; Simple rewrites that don't do a ton with control flow.",
            "(ruleset peepholes)",
            "(Bop (Mul) (Const (Int 0) ty ctx) e)",
            "(Bop (Sub) x x)",
            "(union expr (Const (Int 0) ty ctx))",
            "(Bop (Add) (Bop (Sub) x y) z)",
            "(Bop (Mul) a (Bop (Add) x (Const (Int 1) ty ctx)))",
            "(Top (Select) pred x x)",
            "(Bop (PtrAdd) p (Bop (Add) x y))",
        ] {
            assert!(
                generated_prefix.contains(declaration),
                "Generated peepholes fragment must contain {declaration}"
            );
        }
    }

    #[test]
    fn memory_fragment_matches_file_except_generated_sections() {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/memory.rs)\n";

        let expected = include_str!("../optimizations/memory.egg")
            .trim_end()
            .to_string();
        let actual = super::memory::fragment();
        let actual = actual.replace(GENERATED_MARKER, "").trim_end().to_string();

        let diff = if expected == actual {
            String::new()
        } else {
            similar::TextDiff::from_lines(&expected, &actual)
                .unified_diff()
                .header(
                    "optimizations/memory.egg (generated marker stripped)",
                    "memory::fragment() (generated marker stripped)",
                )
                .to_string()
        };

        insta::assert_snapshot!(diff, @"");
    }

    #[test]
    fn memory_fragment_contains_generated_prefix() {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/memory.rs)\n";

        let fragment = super::memory::fragment();
        let generated_prefix = fragment.as_str();

        assert!(
            generated_prefix.starts_with(GENERATED_MARKER),
            "memory::fragment() must mark the generated fragment"
        );

        for declaration in [
            "(sort ExprSetPrim (Set Expr))",
            "(datatype ExprSet (ES ExprSetPrim))",
            "(relation ExprSet-contains (ExprSet Expr))",
            "(datatype Pointees",
            "(relation Resolved-Pointees (Pointees))",
            "(constructor PointsToCells (Expr Pointees)     Pointees :unextractable)",
            "(rewrite (PointsToCells (If c inputs t e) aps)",
            "(constructor PointsToCellsAtIter (Pointees Expr Expr i64) Pointees)",
            "(set (PointsToCells (DoWhile inputs pred-body) aps)",
            "(relation DemandDontAlias (Expr Expr Pointees))",
            "(relation DontAlias (Expr Expr Pointees))",
            "(DemandDontAlias addr otheraddr (TypeToPointees argty))",
            "(constructor PointsToExpr (Expr           Expr) Expr :unextractable)",
            "(set (PointsToExpr (Get f 1) ptr) (Get f 0))",
            ":ruleset memory)",
        ] {
            assert!(
                generated_prefix.contains(declaration),
                "Generated memory fragment must contain {declaration}"
            );
        }
    }

    #[test]
    fn mem_simple_fragment_matches_file_except_generated_sections() {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/mem_simple.rs)\n";

        let expected = include_str!("../optimizations/mem_simple.egg")
            .trim_end()
            .to_string();
        let actual = super::mem_simple::fragment();
        let actual = actual.replace(GENERATED_MARKER, "").trim_end().to_string();

        let diff = if expected == actual {
            String::new()
        } else {
            similar::TextDiff::from_lines(&expected, &actual)
                .unified_diff()
                .header(
                    "optimizations/mem_simple.egg (generated marker stripped)",
                    "mem_simple::fragment() (generated marker stripped)",
                )
                .to_string()
        };

        insta::assert_snapshot!(diff, @"");
    }

    #[test]
    fn mem_simple_fragment_contains_generated_prefix() {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/mem_simple.rs)\n";

        let fragment = super::mem_simple::fragment();
        let generated_prefix = fragment.as_str();

        assert!(
            generated_prefix.starts_with(GENERATED_MARKER),
            "mem_simple::fragment() must mark the generated fragment"
        );

        for declaration in [
            "(ruleset mem-simple)",
            "(relation NoAlias (Expr Expr))",
            "(NoAlias inputs-i inputs-j)",
            "(NoAlias e (Bop (PtrAdd) e i))",
            "(relation DidMemOptimization (String))",
            "(DidMemOptimization \"commute write then load\")",
            "(DidMemOptimization \"duplicate load\")",
            "(DidMemOptimization \"store forward\")",
            "(DidMemOptimization \"duplicate write\")",
            "(DidMemOptimization \"shadowed write\")",
        ] {
            assert!(
                generated_prefix.contains(declaration),
                "Generated mem_simple fragment must contain {declaration}"
            );
        }
    }

    #[test]
    fn loop_invariant_fragment_matches_file_except_generated_sections() {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/loop_invariant.rs)\n";

        let expected = include_str!("../optimizations/loop_invariant.egg")
            .trim_end()
            .to_string();
        let actual = super::loop_invariant::fragment();
        let actual = actual.replace(GENERATED_MARKER, "").trim_end().to_string();

        let diff = if expected == actual {
            String::new()
        } else {
            similar::TextDiff::from_lines(&expected, &actual)
                .unified_diff()
                .header(
                    "optimizations/loop_invariant.egg (generated marker stripped)",
                    "loop_invariant::fragment() (generated marker stripped)",
                )
                .to_string()
        };

        insta::assert_snapshot!(diff, @"");
    }

    #[test]
    fn loop_invariant_fragment_contains_generated_prefix() {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/loop_invariant.rs)\n";

        let fragment = super::loop_invariant::fragment();
        let generated_prefix = fragment.as_str();

        assert!(
            generated_prefix.starts_with(GENERATED_MARKER),
            "loop_invariant::fragment() must mark the generated fragment"
        );

        for declaration in [
            ";; Loop Invariant",
            "(relation is-inv-Expr (Expr Expr))",
            "(relation is-inv-ListExpr (Expr ListExpr))",
            "(function to-hoist (Expr Expr) Expr :merge new)",
            "(function to-hoist-size (Expr Expr) i64 :merge (max old new))",
            "(ruleset boundary-analysis)",
            "(ruleset boundary-analysis-prep)",
            "(ruleset loop-inv-motion)",
            "(set (to-hoist inputs body) expr)",
            "(set (LoopNumItersGuess new_input new_body) iter-guess)",
        ] {
            assert!(
                generated_prefix.contains(declaration),
                "Generated loop_invariant fragment must contain {declaration}"
            );
        }
    }

    #[test]
    fn loop_simplify_fragment_matches_empty_file_with_generated_marker_only() {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/loop_simplify.rs)\n";

        let expected = include_str!("../optimizations/loop_simplify.egg");
        let actual = super::loop_simplify::fragment();
        let actual = actual.replace(GENERATED_MARKER, "");

        assert!(
            expected.is_empty(),
            "optimizations/loop_simplify.egg must stay empty until real rules are migrated"
        );
        assert!(
            actual.is_empty(),
            "loop_simplify::fragment() must only emit the generated marker while the source file is empty"
        );
        assert_eq!(
            expected, actual,
            "loop_simplify::fragment() must match the empty source file once the generated marker is stripped"
        );
    }

    #[test]
    fn loop_simplify_fragment_contains_generated_prefix() {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/loop_simplify.rs)\n";

        let fragment = super::loop_simplify::fragment();
        assert_eq!(
            fragment, GENERATED_MARKER,
            "loop_simplify::fragment() must be marker-only while optimizations/loop_simplify.egg is empty"
        );
    }

    #[test]
    fn loop_unroll_fragment_matches_file_except_generated_sections() {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/loop_unroll.rs)\n";

        let expected = include_str!("../optimizations/loop_unroll.egg")
            .trim_end()
            .to_string();
        let actual = super::loop_unroll::fragment();
        let actual = actual.replace(GENERATED_MARKER, "").trim_end().to_string();

        let diff = if expected == actual {
            String::new()
        } else {
            similar::TextDiff::from_lines(&expected, &actual)
                .unified_diff()
                .header(
                    "optimizations/loop_unroll.egg (generated marker stripped)",
                    "loop_unroll::fragment() (generated marker stripped)",
                )
                .to_string()
        };

        insta::assert_snapshot!(diff, @"");
    }

    #[test]
    fn loop_unroll_fragment_contains_generated_prefix() {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/loop_unroll.rs)\n";

        let fragment = super::loop_unroll::fragment();
        let generated_prefix = fragment.as_str();

        assert!(
            generated_prefix.starts_with(GENERATED_MARKER),
            "loop_unroll::fragment() must mark the generated fragment"
        );

        for declaration in [
            ";; Some simple simplifications of loops",
            "(ruleset loop-unroll)",
            "(ruleset loop-iters-analysis)",
            "(set (LoopNumItersGuess inputs outputs) 1000)",
            "(set (LoopNumItersGuess inputs outputs) 1)",
            "(= (% start_const 4) 0)",
            "(= (% end_constant 4) 0)",
            "(set (LoopNumItersGuess inputs unrolled) (/ old_cost 4))",
            ":ruleset loop-unroll)",
        ] {
            assert!(
                generated_prefix.contains(declaration),
                "Generated loop_unroll fragment must contain {declaration}"
            );
        }
    }

    #[test]
    fn swap_if_fragment_matches_file_except_generated_sections() {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/swap_if.rs)\n";

        let expected = include_str!("../optimizations/swap_if.egg")
            .trim_end()
            .to_string();
        let actual = super::swap_if::fragment();
        let actual = actual.replace(GENERATED_MARKER, "").trim_end().to_string();

        let diff = if expected == actual {
            String::new()
        } else {
            similar::TextDiff::from_lines(&expected, &actual)
                .unified_diff()
                .header(
                    "optimizations/swap_if.egg (generated marker stripped)",
                    "swap_if::fragment() (generated marker stripped)",
                )
                .to_string()
        };

        insta::assert_snapshot!(diff, @"");
    }

    #[test]
    fn swap_if_fragment_contains_generated_prefix() {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/swap_if.rs)\n";

        let fragment = super::swap_if::fragment();
        let generated_prefix = fragment.as_str();

        assert!(
            generated_prefix.starts_with(GENERATED_MARKER),
            "swap_if::fragment() must mark the generated fragment"
        );

        for declaration in [
            "(ruleset swap-if)",
            "(= lhs (If pred inputs then else))",
            "(union lhs (If (Uop (Not) pred) inputs else then))",
            "(= (tuple-length then) 2)",
            "(= (tuple-length else) 2)",
            "(Concat (Single (Get lhs 1)) (Single (Get lhs 0)))",
            ":ruleset swap-if)",
        ] {
            assert!(
                generated_prefix.contains(declaration),
                "Generated swap_if fragment must contain {declaration}"
            );
        }
    }

    #[test]
    fn rec_to_loop_fragment_matches_file_except_generated_sections() {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/rec_to_loop.rs)\n";

        let expected = include_str!("../optimizations/rec_to_loop.egg")
            .trim_end()
            .to_string();
        let actual = super::rec_to_loop::fragment();
        let actual = actual.replace(GENERATED_MARKER, "").trim_end().to_string();

        let diff = if expected == actual {
            String::new()
        } else {
            similar::TextDiff::from_lines(&expected, &actual)
                .unified_diff()
                .header(
                    "optimizations/rec_to_loop.egg (generated marker stripped)",
                    "rec_to_loop::fragment() (generated marker stripped)",
                )
                .to_string()
        };

        insta::assert_snapshot!(diff, @"");
    }

    #[test]
    fn rec_to_loop_fragment_contains_generated_prefix() {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/rec_to_loop.rs)\n";

        let fragment = super::rec_to_loop::fragment();
        let generated_prefix = fragment.as_str();

        assert!(
            generated_prefix.starts_with(GENERATED_MARKER),
            "rec_to_loop::fragment() must mark the generated fragment"
        );

        for declaration in [
            "(ruleset rec-to-loop)",
            "(Function name in out body)",
            "(= body (If pred always-runs (Call name rec_case) base-case))",
            "(relation Accum-Bop (BinaryOp i64 BinaryOp))",
            "(Accum-Bop (Add) 0 (Add))",
            "(Accum-Bop (Sub) 0 (Add))",
            "(Accum-Bop (Mul) 1 (Mul))",
            "(= then-case",
            ":ruleset rec-to-loop)",
        ] {
            assert!(
                generated_prefix.contains(declaration),
                "Generated rec_to_loop fragment must contain {declaration}"
            );
        }
    }

    #[test]
    fn passthrough_fragment_matches_file_except_generated_sections() {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/passthrough.rs)\n";

        let expected = include_str!("../optimizations/passthrough.egg")
            .trim_end()
            .to_string();
        let actual = super::passthrough::fragment();
        let actual = actual.replace(GENERATED_MARKER, "").trim_end().to_string();

        let diff = if expected == actual {
            String::new()
        } else {
            similar::TextDiff::from_lines(&expected, &actual)
                .unified_diff()
                .header(
                    "optimizations/passthrough.egg (generated marker stripped)",
                    "passthrough::fragment() (generated marker stripped)",
                )
                .to_string()
        };

        insta::assert_snapshot!(diff, @"");
    }

    #[test]
    fn passthrough_fragment_contains_generated_prefix() {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/passthrough.rs)\n";

        let fragment = super::passthrough::fragment();
        let generated_prefix = fragment.as_str();

        assert!(
            generated_prefix.starts_with(GENERATED_MARKER),
            "passthrough::fragment() must mark the generated fragment"
        );

        for declaration in [
            "(ruleset passthrough)",
            "(= loop (DoWhile inputs pred-outputs))",
            "(PureType lhs_ty)",
            "(= switch (Switch pred inputs branches))",
            "(= if (If pred inputs then_ else_))",
            "(ruleset state-edge-passthrough)",
            "(union (Get           if i) pred)",
            "(union (Get           if i) (Uop (Not) pred))",
        ] {
            assert!(
                generated_prefix.contains(declaration),
                "Generated passthrough fragment must contain {declaration}"
            );
        }
    }

    #[test]
    fn loop_strength_reduction_fragment_matches_file_except_generated_sections() {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/loop_strength_reduction.rs)\n";

        let expected = include_str!("../optimizations/loop_strength_reduction.egg")
            .trim_end()
            .to_string();
        let actual = super::loop_strength_reduction::fragment();
        let actual = actual.replace(GENERATED_MARKER, "").trim_end().to_string();

        let diff = if expected == actual {
            String::new()
        } else {
            similar::TextDiff::from_lines(&expected, &actual)
                .unified_diff()
                .header(
                    "optimizations/loop_strength_reduction.egg (generated marker stripped)",
                    "loop_strength_reduction::fragment() (generated marker stripped)",
                )
                .to_string()
        };

        insta::assert_snapshot!(diff, @"");
    }

    #[test]
    fn loop_strength_reduction_fragment_contains_generated_prefix() {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/loop_strength_reduction.rs)\n";

        let fragment = super::loop_strength_reduction::fragment();
        let generated_prefix = fragment.as_str();

        assert!(
            generated_prefix.starts_with(GENERATED_MARKER),
            "loop_strength_reduction::fragment() must mark the generated fragment"
        );

        for declaration in [
            ";; ORIGINAL",
            "(ruleset loop-strength-reduction)",
            "(relation lsr-inv (Expr Expr Expr))",
            "(= old-loop (DoWhile inputs pred-and-outputs))",
            "(let new-inputs (Concat inputs (Single d-init)))",
            "(union old-loop (SubTuple new-loop 0 n))",
        ] {
            assert!(
                generated_prefix.contains(declaration),
                "Generated loop_strength_reduction fragment must contain {declaration}"
            );
        }
    }

    #[test]
    fn ivt_fragment_matches_file_except_generated_sections() {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/ivt.rs)\n";

        let expected = include_str!("../optimizations/ivt.egg")
            .trim_end()
            .to_string();
        let actual = super::ivt::fragment();
        let actual = actual.replace(GENERATED_MARKER, "").trim_end().to_string();

        let diff = if expected == actual {
            String::new()
        } else {
            similar::TextDiff::from_lines(&expected, &actual)
                .unified_diff()
                .header(
                    "optimizations/ivt.egg (generated marker stripped)",
                    "ivt::fragment() (generated marker stripped)",
                )
                .to_string()
        };

        insta::assert_snapshot!(diff, @"");
    }

    #[test]
    fn ivt_fragment_contains_generated_prefix() {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/ivt.rs)\n";

        let fragment = super::ivt::fragment();
        let generated_prefix = fragment.as_str();

        assert!(
            generated_prefix.starts_with(GENERATED_MARKER),
            "ivt::fragment() must mark the generated fragment"
        );

        for declaration in [
            "(relation IVTNewInputsAnalysisDemand (Expr))",
            "(ruleset ivt-analysis)",
            "(constructor IVTAnalysisRes (Expr Expr             TypeList         i64) IVTRes)",
            "(function IVTNewInputsAnalysisImpl (Expr  Expr  Node) IVTRes :merge (IVTMin old new))",
            "(ruleset loop-inversion)",
            "(union final-permuted loop)",
        ] {
            assert!(
                generated_prefix.contains(declaration),
                "Generated ivt fragment must contain {declaration}"
            );
        }
    }

    #[test]
    fn conditional_invariant_code_motion_fragment_matches_file_except_generated_sections() {
        const GENERATED_MARKER: &str = "; (Generated from eggplant Rust: src/eggplant_backend/conditional_invariant_code_motion.rs)\n";

        let expected = include_str!("../optimizations/conditional_invariant_code_motion.egg")
            .trim_end()
            .to_string();
        let actual = super::conditional_invariant_code_motion::fragment();
        let actual = actual.replace(GENERATED_MARKER, "").trim_end().to_string();

        let diff = if expected == actual {
            String::new()
        } else {
            similar::TextDiff::from_lines(&expected, &actual)
                .unified_diff()
                .header(
                    "optimizations/conditional_invariant_code_motion.egg (generated marker stripped)",
                    "conditional_invariant_code_motion::fragment() (generated marker stripped)",
                )
                .to_string()
        };

        insta::assert_snapshot!(diff, @"");
    }

    #[test]
    fn conditional_invariant_code_motion_fragment_contains_generated_prefix() {
        const GENERATED_MARKER: &str = "; (Generated from eggplant Rust: src/eggplant_backend/conditional_invariant_code_motion.rs)\n";

        let fragment = super::conditional_invariant_code_motion::fragment();
        let generated_prefix = fragment.as_str();

        assert!(
            generated_prefix.starts_with(GENERATED_MARKER),
            "conditional_invariant_code_motion::fragment() must mark the generated fragment"
        );

        for declaration in [
            "(ruleset cicm)",
            "(ruleset cicm-index)",
            "(relation InvCodeMotionCandidate (Expr Expr))",
            "(relation ExtractedExprCache (Term Expr Assumption))",
            "(let new_term (TermSubst outer_ctx orig_ins t1))",
            "(union if_e (If pred new_ins new_thn new_els))",
        ] {
            assert!(
                generated_prefix.contains(declaration),
                "Generated conditional_invariant_code_motion fragment must contain {declaration}"
            );
        }
    }

    #[test]
    fn conditional_push_in_fragment_matches_file_except_generated_sections() {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/conditional_push_in.rs)\n";

        let expected = include_str!("../optimizations/conditional_push_in.egg")
            .trim_end()
            .to_string();
        let actual = super::conditional_push_in::fragment();
        let actual = actual.replace(GENERATED_MARKER, "").trim_end().to_string();

        let diff = if expected == actual {
            String::new()
        } else {
            similar::TextDiff::from_lines(&expected, &actual)
                .unified_diff()
                .header(
                    "optimizations/conditional_push_in.egg (generated marker stripped)",
                    "conditional_push_in::fragment() (generated marker stripped)",
                )
                .to_string()
        };

        insta::assert_snapshot!(diff, @"");
    }

    #[test]
    fn conditional_push_in_fragment_contains_generated_prefix() {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/conditional_push_in.rs)\n";

        let fragment = super::conditional_push_in::fragment();
        let generated_prefix = fragment.as_str();

        assert!(
            generated_prefix.starts_with(GENERATED_MARKER),
            "conditional_push_in::fragment() must mark the generated fragment"
        );

        for declaration in [
            "(ruleset push-in)",
            "(RELIESONCONTEXT)",
            "(let new_ins (Concat orig_inputs (Single x)))",
            "(let new_thn (Subst if_tr st_tr thn))",
            "(union if_e (If pred new_ins new_thn new_els))",
        ] {
            assert!(
                generated_prefix.contains(declaration),
                "Generated conditional_push_in fragment must contain {declaration}"
            );
        }
    }

    #[test]
    fn debug_helper_fragment_matches_file_except_generated_sections() {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/debug_helper.rs)\n";

        let expected = include_str!("../utility/debug-helper.egg")
            .trim_end()
            .to_string();
        let actual = super::debug_helper::fragment();
        let actual = actual.replace(GENERATED_MARKER, "").trim_end().to_string();

        let diff = if expected == actual {
            String::new()
        } else {
            similar::TextDiff::from_lines(&expected, &actual)
                .unified_diff()
                .header(
                    "utility/debug-helper.egg (generated marker stripped)",
                    "debug_helper::fragment() (generated marker stripped)",
                )
                .to_string()
        };

        insta::assert_snapshot!(diff, @"");
    }

    #[test]
    fn debug_helper_fragment_contains_generated_prefix() {
        const GENERATED_MARKER: &str =
            "; (Generated from eggplant Rust: src/eggplant_backend/debug_helper.rs)\n";

        let fragment = super::debug_helper::fragment();
        let generated_prefix = fragment.as_str();

        assert!(
            generated_prefix.starts_with(GENERATED_MARKER),
            "debug_helper::fragment() must mark the generated fragment"
        );

        for declaration in [
            "(ruleset debug-deletes)",
            "((delete (HasType a b)))",
            "((delete (ContextOf e a)))",
            "((delete (ExprIsResolved e)))",
            "((delete (IntT)))",
        ] {
            assert!(
                generated_prefix.contains(declaration),
                "Generated debug_helper fragment must contain {declaration}"
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
    fn prologue_runs_migrated_context_of_rules() {
        let ctx = "(InFunc \"RLCR\")";
        let expr = "(Bop (Add) (Const (Int 3) (Base (IntT)) (InFunc \"RLCR\")) (Const (Int 4) (Base (IntT)) (InFunc \"RLCR\")))";
        let program = format!(
            "{}\n(let __rlcr_expr {expr})\n(run-schedule {})\n(check (ContextOf __rlcr_expr {ctx}))\n",
            crate::prologue(),
            crate::schedule::types_and_indexing()
        );

        let mut egraph = egglog::EGraph::default();
        egraph.parse_and_run_program(None, &program).unwrap();
    }

    #[test]
    fn prologue_runs_migrated_subst_rules() {
        let to = "(Const (Int 3) (Base (IntT)) (InFunc \"RLCR\"))";
        let input = "(Arg (Base (IntT)) (InFunc \"OLD\"))";
        let expected = "(If (Const (Bool true) (Base (IntT)) (InFunc \"RLCR\")) (Const (Int 3) (Base (IntT)) (InFunc \"RLCR\")) (Arg (Base (IntT)) (InFunc \"OLD\")) (Arg (Base (IntT)) (InFunc \"OLD\")))";
        let program = format!(
            "{}\n(let __rlcr_to {to})\n(let __rlcr_in {input})\n(let __rlcr_subst (IfSubst __rlcr_to __rlcr_in))\n(run-schedule {})\n(run-schedule (saturate subst))\n(check (= __rlcr_subst {expected}))\n",
            crate::prologue(),
            crate::schedule::types_and_indexing()
        );

        let mut egraph = egglog::EGraph::default();
        egraph.parse_and_run_program(None, &program).unwrap();
    }

    #[test]
    fn prologue_runs_migrated_canonicalize_rules() {
        let lhs = "(Const (Int 7) (Base (IntT)) (InFunc \"RLCR\"))";
        let rhs = "(Const (Int 5) (Base (IntT)) (InFunc \"RLCR\"))";
        let expr = format!("(Bop (GreaterThan) {lhs} {rhs})");
        let expected = format!("(Bop (LessThan) {rhs} {lhs})");
        let program = format!(
            "{}\n(let __rlcr_expr {expr})\n(run-schedule (saturate canon))\n(check (= __rlcr_expr {expected}))\n",
            crate::prologue()
        );

        let mut egraph = egglog::EGraph::default();
        egraph.parse_and_run_program(None, &program).unwrap();
    }

    #[test]
    fn prologue_runs_migrated_expr_size_rules() {
        let expr = "(Concat (Single (Const (Int 3) (Base (IntT)) (InFunc \"RLCR\"))) (Single (Const (Int 4) (Base (IntT)) (InFunc \"RLCR\"))))";
        let branches = "(Cons (Const (Int 9) (Base (IntT)) (InFunc \"RLCR\")) (Nil))";
        let program = format!(
            "{}\n(let __rlcr_expr {expr})\n(let __rlcr_branches {branches})\n(run-schedule (saturate always-run))\n(check (= (Expr-size __rlcr_expr) 2))\n(check (= (ListExpr-size __rlcr_branches) 1))\n",
            crate::prologue()
        );

        let mut egraph = egglog::EGraph::default();
        egraph.parse_and_run_program(None, &program).unwrap();
    }

    #[test]
    fn prologue_runs_migrated_drop_at_rules() {
        let ctx = "(InFunc \"RLCR\")";
        let input = "(Get (Arg (TupleT (TCons (IntT) (TCons (BoolT) (TNil)))) (InFunc \"OLD\")) 1)";
        let expected = "(Get (Arg (TupleT (TCons (BoolT) (TNil))) (InFunc \"RLCR\")) 0)";
        let schedule = "(saturate (saturate type-helpers) type-analysis)";
        let program = format!(
            "{}\n(let __rlcr_input {input})\n(let __rlcr_drop (DropAt {ctx} 0 __rlcr_input))\n(run-schedule {schedule})\n(run-schedule (saturate is-resolved))\n(run-schedule (saturate drop))\n(run-schedule apply-drop-unions)\n(run-schedule cleanup-drop)\n(run-schedule {schedule})\n(check (= __rlcr_drop {expected}))\n",
            crate::prologue()
        );

        let mut egraph = egglog::EGraph::default();
        egraph.parse_and_run_program(None, &program).unwrap();
    }

    #[test]
    fn prologue_runs_migrated_interval_analysis_rules() {
        let expr =
            "(Bop (Add) (Const (Int 1) (Base (IntT)) (InFunc \"RLCR\")) (Const (Int 2) (Base (IntT)) (InFunc \"RLCR\")))";
        let expected = "(Const (Int 3) (Base (IntT)) (InFunc \"RLCR\"))";
        let schedule = crate::schedule::types_and_indexing();
        let program = format!(
            "{}\n(let __rlcr_expr {expr})\n(run-schedule {schedule})\n(run-schedule (saturate interval-analysis))\n(check (= __rlcr_expr {expected}))\n",
            crate::prologue()
        );

        let mut egraph = egglog::EGraph::default();
        egraph.parse_and_run_program(None, &program).unwrap();
    }

    #[test]
    fn prologue_runs_migrated_switch_rewrite_rules() {
        let tuple_ty = "(TupleT (TCons (IntT) (TCons (IntT) (TNil))))";
        let left = "(Const (Int 3) (Base (IntT)) (InFunc \"RLCR\"))";
        let right = "(Const (Int 5) (Base (IntT)) (InFunc \"RLCR\"))";
        let pred = format!("(Bop (LessThan) {left} {right})");
        let inputs = format!("(Concat (Single {left}) (Single {right}))");
        let then_branch = format!("(Single (Get (Arg {tuple_ty} (InIf true {pred} {inputs})) 0))");
        let else_branch = format!("(Single (Get (Arg {tuple_ty} (InIf false {pred} {inputs})) 1))");
        let schedule = crate::schedule::types_and_indexing();
        let program = format!(
            "{}\n(let __rlcr_if (If {pred} {inputs} {then_branch} {else_branch}))\n(run-schedule {schedule})\n(run-schedule (saturate switch_rewrite))\n(check (= (Get __rlcr_if 0) (Bop (Smin) {left} {right})))\n",
            crate::prologue()
        );

        let mut egraph = egglog::EGraph::default();
        egraph.parse_and_run_program(None, &program).unwrap();
    }

    #[test]
    fn prologue_runs_migrated_select_rules() {
        let arg_ty = "(TupleT (TNil))";
        let ctx = "(InFunc \"RLCR\")";
        let pred = format!("(Const (Bool true) {arg_ty} {ctx})");
        let inputs = format!("(Empty {arg_ty} {ctx})");
        let then_ctx = format!("(InIf true {pred} {inputs})");
        let else_ctx = format!("(InIf false {pred} {inputs})");
        let then_expr = format!(
            "(Bop (Add) (Const (Int 1) {arg_ty} {then_ctx}) (Const (Int 2) {arg_ty} {then_ctx}))"
        );
        let else_expr = format!(
            "(Bop (Sub) (Const (Int 9) {arg_ty} {else_ctx}) (Const (Int 4) {arg_ty} {else_ctx}))"
        );
        let expected_then =
            format!("(Bop (Add) (Const (Int 1) {arg_ty} {ctx}) (Const (Int 2) {arg_ty} {ctx}))");
        let expected_else =
            format!("(Bop (Sub) (Const (Int 9) {arg_ty} {ctx}) (Const (Int 4) {arg_ty} {ctx}))");
        let expected = format!("(Top (Select) {pred} {expected_then} {expected_else})");
        let helpers = crate::schedule::helpers();
        let program = format!(
            "{}\n(let __rlcr_if (If {pred} {inputs} (Single {then_expr}) (Single {else_expr})))\n(run-schedule {helpers})\n(run-schedule select_opt)\n(run-schedule {helpers})\n(check (= (Get __rlcr_if 0) {expected}))\n",
            crate::prologue()
        );

        let mut egraph = egglog::EGraph::default();
        egraph.parse_and_run_program(None, &program).unwrap();
    }

    #[test]
    fn prologue_runs_migrated_peepholes_rules() {
        let ty = "(Base (StateT))";
        let ctx = "(InFunc \"RLCR\")";
        let expr = format!("(Bop (Sub) (Const (Int 7) {ty} {ctx}) (Const (Int 7) {ty} {ctx}))");
        let expected = format!("(Const (Int 0) {ty} {ctx})");
        let schedule = crate::schedule::types_and_indexing();
        let program = format!(
            "{}\n(let __rlcr_expr {expr})\n(run-schedule {schedule})\n(run-schedule peepholes)\n(check (= __rlcr_expr {expected}))\n",
            crate::prologue()
        );

        let mut egraph = egglog::EGraph::default();
        egraph.parse_and_run_program(None, &program).unwrap();
    }

    #[test]
    fn prologue_runs_migrated_memory_rules() {
        use crate::ast::*;
        use crate::schema::{BaseType, Type};

        let one = int_ty(1, Type::Base(BaseType::IntT));
        let two = int(2).with_arg_types(tuplet!(statet()), Type::Base(intt()));
        let orig_state = get(arg_ty(tuplet!(statet())), 0);
        let ptr_and_state = alloc(0, one, orig_state.clone(), pointert(intt()));
        let ptr = get(ptr_and_state.clone(), 0);
        let state = get(ptr_and_state, 1);
        let state = write(ptr.clone(), two.clone(), state);
        let val_and_state = load(ptr, state);
        let val = get(val_and_state.clone(), 0);
        let state = get(val_and_state, 1);
        let res = tprint(val, state);
        let program = format!(
            "{}\n{res}\n(run-schedule\n    (repeat 6\n        (saturate\n            always-run\n            memory-helpers)\n        memory))\n(check (= {res} (Bop (Print) {two} rest)))\n",
            crate::prologue()
        );

        let mut egraph = egglog::EGraph::default();
        egraph.parse_and_run_program(None, &program).unwrap();
    }

    #[test]
    fn prologue_runs_migrated_mem_simple_rules() {
        use crate::ast::*;
        use crate::schema::{BaseType, Type};

        let one = int_ty(1, Type::Base(BaseType::IntT));
        let two = int(2).with_arg_types(tuplet!(statet()), Type::Base(intt()));
        let orig_state = get(arg_ty(tuplet!(statet())), 0);
        let ptr_and_state = alloc(0, one, orig_state.clone(), pointert(intt()));
        let ptr = get(ptr_and_state.clone(), 0);
        let state = get(ptr_and_state, 1);
        let write_expr = write(ptr.clone(), two.clone(), state);
        let load_expr = load(ptr, write_expr.clone());
        let program = format!(
            "{}\n(let __rlcr_load {load_expr})\n(run-schedule mem-simple)\n(check (= (Get __rlcr_load 0) {two}))\n(check (= (Get __rlcr_load 1) {write_expr}))\n",
            crate::prologue()
        );

        let mut egraph = egglog::EGraph::default();
        egraph.parse_and_run_program(None, &program).unwrap();
    }

    #[test]
    fn prologue_runs_migrated_loop_invariant_rules() -> crate::Result {
        use crate::add_context::ContextCache;
        use crate::ast::*;
        use crate::interpreter::Value;
        use crate::schema::Assumption;

        let mut cache = ContextCache::new_dummy_ctx();
        let output_ty = tuplet!(intt(), intt(), intt(), statet());
        let inner_inv = getat(1);
        let inv = add(inner_inv.clone(), int(1));
        let print = tprint(inv.clone(), getat(3));

        let my_loop = dowhile(
            parallel!(getat(0), getat(1), getat(2), getat(3)),
            parallel!(
                less_than(getat(0), getat(1)),
                int(3),
                getat(1),
                getat(2),
                print,
            ),
        )
        .with_arg_types(output_ty.clone(), output_ty.clone())
        .add_ctx_with_cache(Assumption::dummy(), &mut cache);

        let new_out_ty = tuplet!(intt(), intt(), intt(), statet(), intt());
        let mut cache = ContextCache::new_symbolic_ctx();

        let hoisted_loop = dowhile(
            parallel!(
                getat(0),
                getat(1),
                getat(2),
                getat(3),
                add(int(1), getat(1))
            ),
            parallel!(
                less_than(getat(0), getat(1)),
                int(3),
                getat(1),
                getat(2),
                tprint(getat(4), getat(3)),
                getat(4)
            ),
        )
        .with_arg_types(output_ty.clone(), new_out_ty)
        .add_ctx_with_cache(Assumption::dummy(), &mut cache);

        let build = format!("(let loop {}) \n", my_loop);
        let check = format!(
            "(check {})
             (check (= loop (SubTuple {} 0 4)))",
            hoisted_loop.clone(),
            hoisted_loop
        );

        crate::egglog_test(
            &build,
            &check,
            vec![],
            Value::Tuple(vec![]),
            Value::Tuple(vec![]),
            vec![],
        )
    }

    #[test]
    fn prologue_runs_migrated_loop_unroll_rules() -> crate::Result {
        use crate::ast::*;
        use crate::egglog_test;

        let prog = dowhile(
            parallel!(int(0)),
            parallel!(
                less_than(add(getat(0), int(1)), int(8)),
                add(getat(0), int(1))
            ),
        )
        .add_arg_type(base(intt()));

        let unrolled_add = add(add(add(add(getat(0), int(1)), int(1)), int(1)), int(1));
        let expected = dowhile(
            parallel!(int(0)),
            parallel!(less_than(unrolled_add.clone(), int(8)), unrolled_add),
        )
        .add_arg_type(base(intt()))
        .add_symbolic_ctx();

        egglog_test(
            &format!("{prog}"),
            &format!("(check (= {prog} {expected}))"),
            vec![prog.to_program(base(intt()), tuplet!(intt()))],
            intv(0),
            tuplev!(intv(8)),
            vec![],
        )
    }

    #[test]
    fn prologue_runs_migrated_swap_if_rules() {
        let arg_ty = "(TupleT (TNil))";
        let ctx = "(InFunc \"RLCR\")";
        let pred = format!("(Const (Bool true) (Base (BoolT)) {ctx})");
        let inputs = format!("(Empty {arg_ty} {ctx})");
        let then_ctx = format!("(InIf true {pred} {inputs})");
        let else_ctx = format!("(InIf false {pred} {inputs})");
        let then_branch = format!("(Single (Const (Int 1) (Base (IntT)) {then_ctx}))");
        let else_branch = format!("(Single (Const (Int 2) (Base (IntT)) {else_ctx}))");
        let if_expr = format!("(If {pred} {inputs} {then_branch} {else_branch})");
        let expected = format!("(If (Uop (Not) {pred}) {inputs} {else_branch} {then_branch})");
        let program = format!(
            "{}\n(let __rlcr_if {if_expr})\n(run-schedule swap-if)\n(check (= __rlcr_if {expected}))\n",
            crate::prologue()
        );

        let mut egraph = egglog::EGraph::default();
        egraph.parse_and_run_program(None, &program).unwrap();
    }

    #[test]
    fn prologue_runs_migrated_rec_to_loop_rules() {
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

        let tuple_int_state = crate::ast::tuplet!(crate::ast::intt(), crate::ast::statet());
        let always_runs = crate::ast::parallel!(crate::ast::getat(0), crate::ast::getat(1));
        let pred =
            crate::ast::less_than(crate::ast::get(always_runs.clone(), 0), crate::ast::int(5));
        let rec_case = crate::ast::parallel!(
            crate::ast::add(crate::ast::get(always_runs.clone(), 0), crate::ast::int(1)),
            crate::ast::get(always_runs.clone(), 1)
        );
        let f = crate::ast::function(
            "f",
            tuple_int_state.clone(),
            tuple_int_state.clone(),
            crate::ast::tif(
                pred,
                always_runs.clone(),
                crate::ast::call("f", rec_case),
                always_runs,
            ),
        );
        let main = crate::ast::function(
            "main",
            tuple_int_state.clone(),
            tuple_int_state,
            crate::ast::call("f", crate::ast::arg()),
        );
        let program = crate::ast::program!(main, f);

        let helpers = crate::schedule::helpers();
        let schedule = format!(
            "
(run-schedule
  (repeat 2
      {helpers}
      swap-if)
  {helpers}
  rec-to-loop
  {helpers})"
        );
        let egglog_prog =
            crate::build_program(&program, None, &program.fns(), &schedule, None, true);

        let suffix_anchor = "(relation InlinedCall (String Expr))";
        let suffix_start = egglog_prog.find(suffix_anchor).expect(
            "build_program output must contain the InlinedCall relation (used as the prologue boundary for this test)",
        );
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
                    "text backend extracted rec_to_loop program",
                    "eggplant backend extracted rec_to_loop program",
                )
                .to_string()
        };

        insta::assert_snapshot!(diff, @"");
        assert!(
            actual.contains("DoWhile"),
            "rec_to_loop regression should extract a looped program"
        );
    }

    #[test]
    fn prologue_runs_migrated_passthrough_rules() -> crate::Result {
        use crate::ast::*;

        let build = get(
            tif(
                less_than(arg(), int(5)),
                empty(),
                single(ttrue()),
                single(tfalse()),
            ),
            0,
        );
        let check = less_than(arg(), int(5));

        let (build, build_cache) = build.to_program(base(intt()), base(boolt())).add_context();
        let (check, check_cache) = check.to_program(base(intt()), base(boolt())).add_context();
        crate::egglog_test(
            &format!("(let b {build})\n{}", build_cache.get_unions()),
            &format!(
                "(let c {check})\n{} (check (= b c))",
                check_cache.get_unions()
            ),
            vec![build, check],
            intv(3),
            val_bool(true),
            vec![],
        )
    }

    #[test]
    fn prologue_runs_migrated_loop_strength_reduction_rules() -> crate::Result {
        use crate::ast::*;
        use crate::egglog_test;

        let prog = dowhile(
            parallel!(int(0), int(0)),
            parallel!(
                less_than(add(getat(0), int(1)), int(8)),
                add(getat(0), int(1)),
                mul(int(3), getat(0))
            ),
        )
        .add_arg_type(emptyt());

        let expected = dowhile(
            parallel!(int(0), int(0), int(0)),
            parallel!(
                less_than(add(getat(0), int(1)), int(8)),
                add(getat(0), int(1)),
                getat(2),
                add(getat(2), int(3))
            ),
        )
        .add_arg_type(emptyt())
        .add_symbolic_ctx();

        egglog_test(
            &format!("(let myloop {prog})"),
            &format!("(check (= myloop (SubTuple {expected} 0 2)))"),
            vec![prog.to_program(emptyt(), tuplet!(intt(), intt()))],
            emptyv(),
            tuplev!(intv(8), intv(21)),
            vec![],
        )
    }

    #[test]
    fn prologue_runs_migrated_ivt_rules() -> crate::Result {
        // Initialize the logger for this test
        let _ = env_logger::try_init();

        use crate::ast::*;
        use crate::egglog_test;

        let cond = less_than(getat(0), int(10));
        let if_in_loop = tif(
            cond.clone(),
            parallel!(add(getat(0), getat(1))),
            parallel!(add(getat(0), int(1)), int(2)),
            parallel!(getat(0), int(3)),
        );

        let my_loop = dowhile(
            parallel!(int(0), int(0), int(1)),
            parallel!(
                cond,
                get(if_in_loop.clone(), 0),
                get(if_in_loop, 1),
                getat(2)
            ),
        )
        .add_arg_type(tuplet!())
        .add_ctx(infunc("main"))
        .0;

        let added = add(getat(0), int(1));
        let inner_loop_new = dowhile(
            arg(),
            parallel!(
                less_than(added.clone(), int(10)),
                add(added, int(2)),
                getat(1)
            ),
        );
        let expected_if = tif(ttrue(), parallel!(int(0), int(1)), inner_loop_new, arg());

        let expected = parallel!(get(expected_if.clone(), 0), int(3), get(expected_if, 1))
            .add_arg_type(tuplet!())
            .add_symbolic_ctx();

        egglog_test(
            &format!("(let myloop {my_loop})"),
            &format!("(check (= myloop {expected}))"),
            vec![
                my_loop.to_program(emptyt(), tuplet!(intt(), intt(), intt())),
                expected
                    .add_ctx(infunc("main"))
                    .0
                    .to_program(emptyt(), tuplet!(intt(), intt(), intt())),
            ],
            tuplev!(),
            tuplev!(intv(12), intv(3), intv(1)),
            vec![],
        )
    }

    #[test]
    fn prologue_runs_migrated_conditional_invariant_code_motion_rules() {
        use crate::ast::*;

        fn run(prologue: &str, expr: &str) -> (String, Vec<String>) {
            let helpers = crate::schedule::helpers();
            let program = format!(
                "{prologue}\n(let __rlcr_expr {expr})\n(ExprIsValid __rlcr_expr)\n(run-schedule {helpers})\n(run-schedule cicm)\n(run-schedule {helpers})\n"
            );
            let mut egraph = egglog::EGraph::default();
            egraph.parse_and_run_program(None, &program).unwrap();

            let (serialized, _) = crate::greedy_dag_extractor::serialized_egraph(egraph.clone());
            let if_nodes = serialized
                .nodes
                .values()
                .filter(|node| node.op == "If")
                .map(|node| format!("{node:?}"))
                .collect::<Vec<_>>();

            let mut termdag = egglog::TermDag::default();
            let (sort, value) = egraph
                .eval_expr(&egglog::ast::Expr::Var(
                    egglog::ast::Span::Panic,
                    "__rlcr_expr".into(),
                ))
                .unwrap();
            let (_, extracted) = egraph.extract(value, &mut termdag, &sort).unwrap();
            let mut if_nodes = if_nodes;
            if_nodes.sort();
            (termdag.to_string(&extracted), if_nodes)
        }

        let candidate = tif(
            getat(2),
            parallel!(getat(0), getat(1), getat(3)),
            less_than(getat(0), int(7)),
            less_than(getat(1), int(7)),
        )
        .with_arg_types(tuplet!(intt(), intt(), boolt(), statet()), base(boolt()));

        let (expected_extracted, expected_if_nodes) =
            run(&crate::prologue_egglog_text(), &candidate.to_string());
        let (actual_extracted, actual_if_nodes) = run(&crate::prologue(), &candidate.to_string());

        assert_eq!(
            actual_extracted, expected_extracted,
            "migrated cicm fragment should match the text backend extracted expression"
        );
        assert_eq!(
            actual_if_nodes, expected_if_nodes,
            "migrated cicm fragment should match the text backend serialized If nodes"
        );
        assert!(
            actual_if_nodes.iter().any(|node| node.contains("SubTuple")),
            "helpers+cicm regression should preserve the normalized If input shape"
        );
    }

    #[test]
    fn prologue_runs_migrated_conditional_push_in_rules() {
        use crate::ast::*;

        fn run(prologue: &str, expr: &str) -> (String, Vec<String>) {
            let helpers = crate::schedule::helpers();
            let program = format!(
                "{prologue}\n(let __rlcr_expr {expr})\n(ExprIsValid __rlcr_expr)\n(run-schedule {helpers})\n(run-schedule push-in)\n(run-schedule {helpers})\n"
            );
            let mut egraph = egglog::EGraph::default();
            egraph.parse_and_run_program(None, &program).unwrap();

            let (serialized, _) = crate::greedy_dag_extractor::serialized_egraph(egraph.clone());
            let mut if_nodes = serialized
                .nodes
                .values()
                .filter(|node| node.op == "If")
                .map(|node| format!("{node:?}"))
                .collect::<Vec<_>>();
            if_nodes.sort();

            let mut termdag = egglog::TermDag::default();
            let (sort, value) = egraph
                .eval_expr(&egglog::ast::Expr::Var(
                    egglog::ast::Span::Panic,
                    "__rlcr_expr".into(),
                ))
                .unwrap();
            let (_, extracted) = egraph.extract(value, &mut termdag, &sort).unwrap();
            (termdag.to_string(&extracted), if_nodes)
        }

        let candidate = tif(
            getat(0),
            parallel!(add(int(1), getat(1))),
            getat(0),
            getat(0),
        )
        .with_arg_types(tuplet!(boolt(), intt()), base(intt()));

        let (expected_extracted, expected_if_nodes) =
            run(&crate::prologue_egglog_text(), &candidate.to_string());
        let (actual_extracted, actual_if_nodes) = run(&crate::prologue(), &candidate.to_string());

        assert_eq!(
            actual_extracted, expected_extracted,
            "migrated push-in fragment should match the text backend extracted expression"
        );
        assert_eq!(
            actual_if_nodes, expected_if_nodes,
            "migrated push-in fragment should match the text backend serialized If nodes"
        );
        assert!(
            actual_if_nodes.iter().any(|node| node.contains("SubTuple")),
            "helpers+push-in regression should preserve the normalized If input shape"
        );
    }

    #[test]
    fn prologue_runs_migrated_debug_helper_rules() {
        let expr = "(Const (Int 7) (Base (IntT)) (InFunc \"DUMMY\"))";
        let ty = "(Base (IntT))";
        let ctx = "(InFunc \"DUMMY\")";
        let program = format!(
            "{}\n(let __rlcr_expr {expr})\n(HasType __rlcr_expr {ty})\n(ContextOf __rlcr_expr {ctx})\n(run-schedule debug-deletes)\n(fail (check (HasType __rlcr_expr {ty})))\n(fail (check (ContextOf __rlcr_expr {ctx})))\n",
            crate::prologue()
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
        let expected = strip_context_of_generated_sections(&expected);
        let actual = strip_context_of_generated_sections(&actual);
        let expected = strip_subst_generated_sections(&expected);
        let actual = strip_subst_generated_sections(&actual);
        let expected = strip_canonicalize_generated_sections(&expected);
        let actual = strip_canonicalize_generated_sections(&actual);
        let expected = strip_expr_size_generated_sections(&expected);
        let actual = strip_expr_size_generated_sections(&actual);
        let expected = strip_drop_at_generated_sections(&expected);
        let actual = strip_drop_at_generated_sections(&actual);
        let expected = strip_interval_analysis_generated_sections(&expected);
        let actual = strip_interval_analysis_generated_sections(&actual);
        let expected = strip_switch_rewrites_generated_sections(&expected);
        let actual = strip_switch_rewrites_generated_sections(&actual);
        let expected = strip_select_generated_sections(&expected);
        let actual = strip_select_generated_sections(&actual);
        let expected = strip_peepholes_generated_sections(&expected);
        let actual = strip_peepholes_generated_sections(&actual);
        let expected = strip_memory_generated_sections(&expected);
        let actual = strip_memory_generated_sections(&actual);
        let expected = strip_mem_simple_generated_sections(&expected);
        let actual = strip_mem_simple_generated_sections(&actual);
        let expected = strip_loop_invariant_generated_sections(&expected);
        let actual = strip_loop_invariant_generated_sections(&actual);
        let expected = strip_loop_simplify_generated_sections(&expected);
        let actual = strip_loop_simplify_generated_sections(&actual);
        let expected = strip_loop_unroll_generated_sections(&expected);
        let actual = strip_loop_unroll_generated_sections(&actual);
        let expected = strip_swap_if_generated_sections(&expected);
        let actual = strip_swap_if_generated_sections(&actual);
        let expected = strip_rec_to_loop_generated_sections(&expected);
        let actual = strip_rec_to_loop_generated_sections(&actual);
        let expected = strip_passthrough_generated_sections(&expected);
        let actual = strip_passthrough_generated_sections(&actual);
        let expected = strip_loop_strength_reduction_generated_sections(&expected);
        let actual = strip_loop_strength_reduction_generated_sections(&actual);
        let expected = strip_ivt_generated_sections(&expected);
        let actual = strip_ivt_generated_sections(&actual);
        let expected = strip_conditional_invariant_code_motion_generated_sections(&expected);
        let actual = strip_conditional_invariant_code_motion_generated_sections(&actual);
        let expected = strip_conditional_push_in_generated_sections(&expected);
        let actual = strip_conditional_push_in_generated_sections(&actual);
        let expected = strip_debug_helper_generated_sections(&expected);
        let actual = strip_debug_helper_generated_sections(&actual);

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
