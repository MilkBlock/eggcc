Implemented the Round 17 strict-order migration for `optimizations/ivt.egg`.

## Completed Work

- Added `dag_in_context/src/eggplant_backend/ivt.rs` as a marker-prefixed raw-string fragment containing the full `ivt.egg` source.
- Wired `dag_in_context/src/eggplant_backend/mod.rs` to use `ivt::fragment()` in `eggplant_backend::prologue()` in place of `include_str!("../optimizations/ivt.egg")`.
- Added fragment drift guards:
  - `ivt_fragment_matches_file_except_generated_sections`
  - `ivt_fragment_contains_generated_prefix`
- Extended the prologue diff guard with `strip_ivt_generated_sections(...)`, stripping the generated marker and normalizing the local boundary before `optimizations/conditional_invariant_code_motion.egg`.
- Added `prologue_runs_migrated_ivt_rules`, reusing the existing representative loop-inversion scenario and asserting the migrated backend rewrites to the expected peeled-if/inner-loop form.
- Fixed one fragment copy mismatch caught by the drift guard (`new-pperm`) plus a whitespace mismatch so the raw-string fragment matches `ivt.egg` exactly.

## Verification

- `env PKG_CONFIG_PATH=/opt/homebrew/opt/cbc/lib/pkgconfig LIBRARY_PATH=/opt/homebrew/opt/cbc/lib DYLD_FALLBACK_LIBRARY_PATH=/opt/homebrew/opt/cbc/lib cargo test --manifest-path dag_in_context/Cargo.toml --features eggplant ivt -- --nocapture`
  - Passed: 4 tests
- `env PKG_CONFIG_PATH=/opt/homebrew/opt/cbc/lib/pkgconfig LIBRARY_PATH=/opt/homebrew/opt/cbc/lib DYLD_FALLBACK_LIBRARY_PATH=/opt/homebrew/opt/cbc/lib cargo test --manifest-path dag_in_context/Cargo.toml --features eggplant prologue_matches_text_backend_except_generated_schema_and_type_analysis_sections -- --nocapture`
  - Passed: 1 test
- `env PKG_CONFIG_PATH=/opt/homebrew/opt/cbc/lib/pkgconfig LIBRARY_PATH=/opt/homebrew/opt/cbc/lib DYLD_FALLBACK_LIBRARY_PATH=/opt/homebrew/opt/cbc/lib cargo test --manifest-path dag_in_context/Cargo.toml --features eggplant --quiet`
  - Passed: `175 passed; 0 failed`

## Goal Tracker Update Request

### Requested Changes:
- Mark `迁移 optimizations/ivt.egg` as completed and verified with Round 17 evidence.
- Add a Round 17 Plan Evolution Log entry documenting that the prologue diff guard now strips the `ivt` generated marker and normalizes the boundary before `optimizations/conditional_invariant_code_motion.egg`.
- Advance the next Active Task to `迁移 optimizations/conditional_invariant_code_motion.egg`.

### Justification:
This round completed the next file in strict compilation order, preserved the established fragment/drift/regression workflow, and revalidated the full `--features eggplant` suite. Updating the tracker keeps AC-4 aligned with the repository’s real migration state and moves the loop cleanly to the next pending `.egg` file.
