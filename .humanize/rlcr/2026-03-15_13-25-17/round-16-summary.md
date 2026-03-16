Implemented the Round 16 strict-order migration for `optimizations/loop_strength_reduction.egg`.

## Completed Work

- Added `dag_in_context/src/eggplant_backend/loop_strength_reduction.rs` as a marker-prefixed raw-string fragment containing the full `loop_strength_reduction.egg` source.
- Wired `dag_in_context/src/eggplant_backend/mod.rs` to use `loop_strength_reduction::fragment()` in `eggplant_backend::prologue()` in place of `include_str!("../optimizations/loop_strength_reduction.egg")`.
- Added fragment drift guards:
  - `loop_strength_reduction_fragment_matches_file_except_generated_sections`
  - `loop_strength_reduction_fragment_contains_generated_prefix`
- Extended the prologue diff guard with `strip_loop_strength_reduction_generated_sections(...)`, stripping the generated marker and normalizing the local boundary before `ivt.egg`.
- Added `prologue_runs_migrated_loop_strength_reduction_rules`, a focused semantic regression that checks a representative induction-variable multiplication loop rewrites to a strength-reduced loop with an extra carried temporary.

## Verification

- `env PKG_CONFIG_PATH=/opt/homebrew/opt/cbc/lib/pkgconfig LIBRARY_PATH=/opt/homebrew/opt/cbc/lib DYLD_FALLBACK_LIBRARY_PATH=/opt/homebrew/opt/cbc/lib cargo test --manifest-path dag_in_context/Cargo.toml --features eggplant loop_strength_reduction -- --nocapture`
  - Passed: 3 tests
- `env PKG_CONFIG_PATH=/opt/homebrew/opt/cbc/lib/pkgconfig LIBRARY_PATH=/opt/homebrew/opt/cbc/lib DYLD_FALLBACK_LIBRARY_PATH=/opt/homebrew/opt/cbc/lib cargo test --manifest-path dag_in_context/Cargo.toml --features eggplant prologue_matches_text_backend_except_generated_schema_and_type_analysis_sections -- --nocapture`
  - Passed: 1 test
- `env PKG_CONFIG_PATH=/opt/homebrew/opt/cbc/lib/pkgconfig LIBRARY_PATH=/opt/homebrew/opt/cbc/lib DYLD_FALLBACK_LIBRARY_PATH=/opt/homebrew/opt/cbc/lib cargo test --manifest-path dag_in_context/Cargo.toml --features eggplant --quiet`
  - Passed: `172 passed; 0 failed`

## Goal Tracker Update Request

### Requested Changes:
- Mark `迁移 optimizations/loop_strength_reduction.egg` as completed and verified with Round 16 evidence.
- Add a Round 16 Plan Evolution Log entry documenting that the prologue diff guard now strips the `loop_strength_reduction` generated marker and normalizes the local boundary before `optimizations/ivt.egg`.
- Advance the next Active Task to `迁移 optimizations/ivt.egg`.

### Justification:
This round completed the next file in strict compilation order, preserved the established fragment/drift/regression workflow, and revalidated the full `--features eggplant` suite. Updating the tracker keeps AC-4 aligned with the repository’s real migration state and moves the loop to the next pending `.egg` file without skipping scope.
