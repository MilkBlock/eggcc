mod schema;

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
    #[test]
    fn schema_fragment_matches_schema_file() {
        let expected = include_str!("../schema.egg");
        let actual = super::schema::fragment();

        let diff = if expected == actual {
            String::new()
        } else {
            similar::TextDiff::from_lines(expected, &actual)
                .unified_diff()
                .header("schema.egg", "schema::fragment()")
                .to_string()
        };

        insta::assert_snapshot!(diff, @"");
    }

    #[test]
    fn prologue_matches_text_backend() {
        let expected = crate::prologue_egglog_text();
        let actual = crate::prologue();

        let diff = if expected == actual {
            String::new()
        } else {
            similar::TextDiff::from_lines(&expected, &actual)
                .unified_diff()
                .header("prologue_egglog_text()", "eggplant_backend::prologue()")
                .to_string()
        };

        insta::assert_snapshot!(diff, @"");
    }
}

