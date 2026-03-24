#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DEMO_SCRIPT="$ROOT_DIR/scripts/phase0-cuda-demo.sh"
DOC_FILE="$ROOT_DIR/docs/phase0_cuda_demo.md"
ARTIFACT_DIR="$ROOT_DIR/.artifacts/phase0-cuda-demo"
HEADLESS=0
EXAMPLE="${1:-shared_saxpy}"

usage() {
  cat <<'EOF'
Usage:
  ./dev-vscode.sh [headless] [shared_saxpy|for_fill|barrier_stub|include_bias|path/to/file.cu]

Examples:
  ./dev-vscode.sh
  ./dev-vscode.sh headless
  ./dev-vscode.sh for_fill
  ./dev-vscode.sh examples/cuda/phase0/include_bias.cu

Behavior:
  - prepares the packaged Phase 0 parse + RVSDG demo artifacts
  - opens VSCode on the repo plus the key source/doc/artifact files
  - with 'headless', only prepares artifacts and prints their paths
EOF
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

if [[ "${1:-}" == "headless" || "${1:-}" == "--headless" ]]; then
  HEADLESS=1
  shift
  EXAMPLE="${1:-shared_saxpy}"
fi

resolve_input() {
  case "$1" in
    shared_saxpy) echo "examples/cuda/phase0/shared_saxpy.cu" ;;
    for_fill) echo "examples/cuda/phase0/for_fill.cu" ;;
    barrier_stub) echo "examples/cuda/phase0/barrier_stub.cu" ;;
    include_bias) echo "examples/cuda/phase0/include_bias.cu" ;;
    *)
      echo "$1"
      ;;
  esac
}

INPUT_REL="$(resolve_input "$EXAMPLE")"
INPUT_ABS="$ROOT_DIR/$INPUT_REL"

if [[ ! -f "$INPUT_ABS" ]]; then
  echo "Input file not found: $INPUT_ABS" >&2
  usage >&2
  exit 1
fi

if [[ ! -x "$DEMO_SCRIPT" ]]; then
  echo "Demo helper missing or not executable: $DEMO_SCRIPT" >&2
  exit 1
fi

echo "[1/3] Preparing parse artifact..."
bash "$DEMO_SCRIPT" parse "$INPUT_REL"

echo "[2/3] Preparing RVSDG artifact..."
bash "$DEMO_SCRIPT" rvsdg-conversion "$INPUT_REL"

PARSE_ARTIFACT="$ARTIFACT_DIR/parse.bril"
RVSDG_ARTIFACT="$ARTIFACT_DIR/rvsdg_conversion.svg"
PARSE_LOG="$ARTIFACT_DIR/parse.log"
RVSDG_LOG="$ARTIFACT_DIR/rvsdg_conversion.log"

echo "[3/3] Debug bundle ready"
echo "source:   $INPUT_ABS"
echo "doc:      $DOC_FILE"
echo "parse:    $PARSE_ARTIFACT"
echo "rvsdg:    $RVSDG_ARTIFACT"
echo "logs:     $PARSE_LOG , $RVSDG_LOG"

launch_vscode() {
  if command -v code >/dev/null 2>&1; then
    exec code \
      --new-window \
      "$ROOT_DIR" \
      "$INPUT_ABS" \
      "$DOC_FILE" \
      "$PARSE_ARTIFACT" \
      "$RVSDG_ARTIFACT" \
      "$PARSE_LOG" \
      "$RVSDG_LOG"
  fi

  if [[ "$OSTYPE" == darwin* ]]; then
    exec open -na "Visual Studio Code" --args \
      --new-window \
      "$ROOT_DIR" \
      "$INPUT_ABS" \
      "$DOC_FILE" \
      "$PARSE_ARTIFACT" \
      "$RVSDG_ARTIFACT" \
      "$PARSE_LOG" \
      "$RVSDG_LOG"
  fi

  echo "Could not find the VSCode CLI ('code') and no macOS fallback is available." >&2
  echo "Install the 'code' shell command in VSCode, or re-run with 'headless'." >&2
  exit 1
}

if [[ "$HEADLESS" -eq 1 ]]; then
  exit 0
fi

echo "Launching VSCode..."
launch_vscode
