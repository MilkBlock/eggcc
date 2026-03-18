## Round 4 Summary

What I completed in this round:

- Finished the native-runner bridge so the `--features eggplant` optimize path can execute the existing typed peephole rules against `eggplant`'s bundled `egglog` types instead of the crate's direct `egglog` types.
- Added `greedy_dag_extractor::serialized_egraph_native(...)` to serialize `eggplant::egglog::EGraph` into the repo's existing `egraph_serialize` pipeline without moving the singleton tx e-graph out of process-global state.
- Sanitized the native prologue before parsing by stripping the legacy diagnostic `(extract ...)` action blocks that `eggplant`'s bundled parser does not accept.
- Restored `debug_helper::fragment()` to the default `eggplant_backend::prologue()` so the generated prologue matches the text backend again and the migrated debug-helper ruleset remains available.
- Kept `mem_simple::fragment()` in `native_execution_prologue()` for schedule compatibility, but left the full memory helper fragment out because `eggplant` still rejects its ungrounded helper rule on the feature-native path.
- Refactored the native runner to operate on the singleton tx e-graph in place via `with_native_rules_egraph(...)`, which fixes the full-suite regression where feature-path runs left later direct typed tests without the eggplant runtime state they expect.
- Updated the native peephole feature tests to use the real native extraction API (`eval_expr` + `extract_value`) and to compare text/native results by e-class equivalence when extraction chooses different representatives.
- Made the shared native test mutex recover from poisoning so one failing test does not cascade into unrelated feature-native failures.

Validation:

- `CARGO_HOME=/tmp/eggcc-cargo-home cargo test --manifest-path dag_in_context/Cargo.toml --offline -- --test-threads=1`
- `CARGO_HOME=/tmp/eggcc-cargo-home cargo test --manifest-path dag_in_context/Cargo.toml --features eggplant --offline -- --test-threads=1`
- `CARGO_HOME=/tmp/eggcc-cargo-home cargo test --manifest-path dag_in_context/Cargo.toml --features eggplant schema_fragment_parses_as_egglog_program --offline -- --test-threads=1`
- `CARGO_HOME=/tmp/eggcc-cargo-home cargo test --manifest-path dag_in_context/Cargo.toml --features eggplant feature_path_uses_native_peepholes_when_text_rules_are_absent --offline -- --test-threads=1`
- `CARGO_HOME=/tmp/eggcc-cargo-home cargo test --manifest-path dag_in_context/Cargo.toml --features eggplant native_feature_runner_smoke_executes_without_deadlocking --offline -- --test-threads=1`
- `CARGO_HOME=/tmp/eggcc-cargo-home cargo test --manifest-path dag_in_context/Cargo.toml --features eggplant native_feature_path_matches_text_backend_for_fixed_input --offline -- --test-threads=1`
- `CARGO_HOME=/tmp/eggcc-cargo-home cargo test --manifest-path dag_in_context/Cargo.toml --features eggplant native_peepholes_align_with_text_backend_arith_case --offline -- --test-threads=1`

What remains open:

- `switch_rewrites` is still text-backed in the feature-native path.
- `select` is still text-backed in the feature-native path.

Notes for the next gate/review:

- The native bridge now advances past the earlier dual-`egglog` mismatch, prologue parse failures, and singleton-egraph teardown regression.
- The current native path includes `mem-simple` but still excludes the full memory-helper fragment from `native_execution_prologue()`; that is an intentional compatibility trim for the existing typed-peephole slice, not a claim that memory/native parity is solved.

## Goal Tracker Update Request

### Requested Changes:
- Mark `Stabilize feature-native tests that share the singleton tx e-graph` as completed with evidence from commit `8cc2dd21` plus the follow-up in-place native runner fix in this round (`with_native_rules_egraph(...)`) and the full `--features eggplant` suite passing.
- Mark the validation-gate task as completed with evidence: `CARGO_HOME=/tmp/eggcc-cargo-home cargo test --manifest-path dag_in_context/Cargo.toml --offline -- --test-threads=1` and `CARGO_HOME=/tmp/eggcc-cargo-home cargo test --manifest-path dag_in_context/Cargo.toml --features eggplant --offline -- --test-threads=1` both passed.
- Update the plan-evolution/open-issue wording to reflect the final Round 4 native-prologue state: unsupported legacy `(extract ...)` actions are sanitized, `mem-simple` stays in the native path, and the full memory-helper fragment is still excluded because it triggers an ungrounded-rule error under the shipped `eggplant` runtime.
- Keep the `switch_rewrites` and `select` tasks active.

### Justification:
- The singleton-egraph stabilization work is now backed by both targeted tests and the full passing feature-gated suite, so it should no longer remain open as an AC-5 blocker.
- AC-5 is now satisfied for the touched scope and should be reflected in the tracker.
- The native-prologue compatibility note changed during the follow-up fix, so the tracker should describe the final state accurately.
- The remaining unfinished work is concentrated in the still-text-backed `switch_rewrites` and `select` modules.
