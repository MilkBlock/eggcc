## Round 4 Summary

What I completed in this round:

- Finished the native-runner bridge so the `--features eggplant` optimize path can execute the existing typed peephole rules against `eggplant`'s bundled `egglog` types instead of the crate's direct `egglog` types.
- Added `greedy_dag_extractor::serialized_egraph_native(...)` to serialize `eggplant::egglog::EGraph` into the repo's existing `egraph_serialize` pipeline.
- Sanitized the native prologue before parsing by stripping the legacy diagnostic `(extract ...)` action blocks that `eggplant`'s bundled parser does not accept.
- Trimmed the native-only prologue to exclude the memory/debug-helper fragments that reference `PointsToCells` and other declarations that are not available in the current feature-native path.
- Updated the native peephole feature tests to use the real native extraction API (`eval_expr` + `extract_value`) and to compare text/native results by e-class equivalence when extraction chooses different representatives.

Validation:

- `CARGO_HOME=/tmp/eggcc-cargo-home cargo test --manifest-path dag_in_context/Cargo.toml --features eggplant schema_fragment_parses_as_egglog_program --offline -- --test-threads=1`
- `CARGO_HOME=/tmp/eggcc-cargo-home cargo test --manifest-path dag_in_context/Cargo.toml --features eggplant feature_path_uses_native_peepholes_when_text_rules_are_absent --offline -- --test-threads=1`
- `CARGO_HOME=/tmp/eggcc-cargo-home cargo test --manifest-path dag_in_context/Cargo.toml --features eggplant native_feature_runner_smoke_executes_without_deadlocking --offline -- --test-threads=1`
- `CARGO_HOME=/tmp/eggcc-cargo-home cargo test --manifest-path dag_in_context/Cargo.toml --features eggplant native_feature_path_matches_text_backend_for_fixed_input --offline -- --test-threads=1`
- `CARGO_HOME=/tmp/eggcc-cargo-home cargo test --manifest-path dag_in_context/Cargo.toml --features eggplant native_peepholes_align_with_text_backend_arith_case --offline -- --test-threads=1`

What remains open:

- `switch_rewrites` is still text-backed in the feature-native path.
- `select` is still text-backed in the feature-native path.
- I did not run the full `cargo test --manifest-path dag_in_context/Cargo.toml` or full `cargo test --manifest-path dag_in_context/Cargo.toml --features eggplant` suites in this round, so AC-5 is still not fully closed.

Notes for the next gate/review:

- The native bridge now advances past the earlier dual-`egglog` mismatch and prologue parse failures.
- The current native path still drops memory-specific fragments from `native_execution_prologue()`; that is an intentional compatibility trim for the existing typed-peephole slice, not a claim that memory/native parity is solved.

## Goal Tracker Update Request

### Requested Changes:
- Mark `Stabilize feature-native tests that share the singleton tx e-graph` as completed with evidence from commit `8cc2dd21` (`dag_in_context/src/eggplant_backend/test_lock.rs` plus the feature-native peephole tests acquiring that lock).
- Add a plan-evolution note that the native execution bridge now intentionally sanitizes unsupported legacy `(extract ...)` actions and excludes memory/debug-helper fragments from `native_execution_prologue()` so the current typed-peephole slice can execute on the shipped `eggplant` runtime.
- Keep the `switch_rewrites`, `select`, and validation-gate tasks active.

### Justification:
- The singleton-egraph serialization work is landed and independently validated, so it should no longer remain as an active flake-risk task.
- The native-bridge compatibility trim changes how the feature-native path is composed today; that should be explicit in the tracker so later rounds do not accidentally reintroduce the failing prologue pieces.
- The remaining typed migrations and full-suite validation are still unresolved and should continue to block completion.
