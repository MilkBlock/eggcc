#!/bin/bash

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
default_input="examples/cuda/phase0/shared_saxpy.cu"
mode="${1:-parse}"
input="${2:-$default_input}"
out_dir="${PHASE0_DEMO_OUT_DIR:-$repo_root/.artifacts/phase0-cuda-demo}"
artifact_path=""
debug_artifact_dir=""

usage() {
    cat <<'EOF'
Usage:
  scripts/phase0-cuda-demo.sh [parse|rvsdg-conversion|optimized-rvsdg|debug-dir] [input.cu]

Defaults:
  mode   = parse
  input  = examples/cuda/phase0/shared_saxpy.cu

Output:
  Writes artifacts and logs under .artifacts/phase0-cuda-demo/

Notes:
  parse + rvsdg-conversion + optimized-rvsdg are the verified observable
  Phase 0 demo paths.
  debug-dir keeps emitted artifacts even if a later non-demo CLI path fails.
EOF
}

if [[ "$mode" == "-h" || "$mode" == "--help" ]]; then
    usage
    exit 0
fi

case "$mode" in
    parse|rvsdg-conversion|optimized-rvsdg|debug-dir)
        ;;
    *)
        echo "Unknown mode: $mode" >&2
        usage >&2
        exit 2
        ;;
esac

mkdir -p "$out_dir"
input_path="$repo_root/$input"

if [[ ! -f "$input_path" ]]; then
    echo "Input file not found: $input_path" >&2
    exit 2
fi

if cargo +1.88.0 --version >/dev/null 2>&1; then
    cargo_cmd=(cargo +1.88.0)
else
    cargo_cmd=(cargo)
fi

log_file="$out_dir/${mode//-/_}.log"

run_phase0() {
    if [[ "$mode" == "debug-dir" ]]; then
        debug_artifact_dir="$out_dir/debug"
        mkdir -p "$debug_artifact_dir"
        bash "$repo_root/scripts/with-local-build-env.sh" \
            "${cargo_cmd[@]}" run -- \
            --debug-dir "$debug_artifact_dir" \
            "$input_path"
    else
        local artifact_ext
        case "$mode" in
            parse) artifact_ext="bril" ;;
            rvsdg-conversion|optimized-rvsdg) artifact_ext="svg" ;;
            *) artifact_ext="out" ;;
        esac
        artifact_path="$out_dir/${mode//-/_}.${artifact_ext}"
        local rc=0
        bash "$repo_root/scripts/with-local-build-env.sh" \
            "${cargo_cmd[@]}" run -- \
            --run-mode "$mode" \
            "$input_path" \
            > "$artifact_path" || rc=$?
        if [[ $rc -ne 0 ]]; then
            return $rc
        fi
    fi
}

set +e
run_phase0 >"$log_file" 2>&1
status=$?
set -e

if [[ $status -ne 0 && "$mode" == "debug-dir" ]]; then
    if find "$debug_artifact_dir" -maxdepth 1 -name '*-rvsdg-conversion.svg' | grep -q .; then
        echo "warning: debug-dir emitted RVSDG artifacts before a later non-demo CLI failure; keeping artifacts" >&2
        status=0
    fi
fi

if [[ $status -ne 0 ]]; then
    echo "phase0 demo failed; log: $log_file" >&2
    if grep -q "requires rustc 1.88.0" "$log_file"; then
        echo "known blocker: toolchain is older than rustc 1.88.0" >&2
    fi
    if grep -q "No suitable version of LLVM was found" "$log_file"; then
        echo "known blocker: LLVM 18 environment is missing; check LLVM_SYS_180_PREFIX" >&2
    fi
    if grep -q "dag_in_context/src/eggplant_backend/interval_analysis.rs" "$log_file"; then
        echo "known blocker: current workspace still has an eggplant backend compile failure in interval_analysis.rs" >&2
    fi
    if grep -q "Span::Panic in impl Display" "$log_file"; then
        echo "known blocker: optimized-rvsdg currently aborts in the egglog display path; use rvsdg-conversion for the observable Phase 0 demo" >&2
    fi
    exit $status
fi

if [[ "$mode" == "debug-dir" ]]; then
    echo "artifacts: $debug_artifact_dir"
else
    echo "artifact: $artifact_path"
fi

echo "log: $log_file"
