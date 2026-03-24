# Phase 0 CUDA Demo

This document packages the accepted Phase 0 CUDA frontend MVP into runnable examples.

## Scope

The current Phase 0 frontend supports:

- `__global__` / `__device__`
- builtin explicit parameters (`threadIdx`, `blockIdx`, `blockDim`, `gridDim`, `warpSize`)
- scalar locals and assignments
- `if` / `for` / `while` / `return`
- arrays and pointers via `alloc` / `ptradd` / `load` / `store`
- CUDA sync builtins such as `__syncthreads()` lowered as opaque call stubs

It intentionally does **not** extend into host launch lowering, richer address spaces, or broader CUDA C++ semantics.

## Examples

Files live under `examples/cuda/phase0/`:

- `shared_saxpy.cu` — shared memory + builtin explicit params
- `for_fill.cu` — loop lowering
- `barrier_stub.cu` — barrier lowered as an opaque effect stub
- `include_bias.cu` + `helper.cuh` — sibling include resolution

## Fast path

Use the helper script:

```bash
cd ~/Repos/egg_related/eggcc_rlcr
scripts/phase0-cuda-demo.sh parse
scripts/phase0-cuda-demo.sh rvsdg-conversion
scripts/phase0-cuda-demo.sh optimized-rvsdg
open .artifacts/phase0-cuda-demo/rvsdg_conversion.svg
```

These two helper invocations are the currently verified observable Phase 0 demo
path on the shared worktree; `optimized-rvsdg` is also currently working for the
same packaged example.

The helper writes outputs and logs to `.artifacts/phase0-cuda-demo/`.

## VSCode debug launcher

If you want a VSCode window prepared like the `eggplant_pattern_view_plugin/dev-vscode.sh` flow, use:

```bash
cd ~/Repos/egg_related/eggcc_rlcr
./dev-vscode.sh
```

This will:

- generate the packaged Phase 0 parse artifact
- generate the packaged Phase 0 RVSDG SVG
- open VSCode on the repo, the sample `.cu`, the demo doc, and the generated artifacts/logs

For non-GUI verification:

```bash
cd ~/Repos/egg_related/eggcc_rlcr
./dev-vscode.sh headless
```

## Direct commands

### 1. Observe `.cu -> Bril`

```bash
cd ~/Repos/egg_related/eggcc_rlcr
bash scripts/with-local-build-env.sh cargo +1.88.0 run -- \
  --run-mode parse \
  examples/cuda/phase0/shared_saxpy.cu \
  > /tmp/phase0_shared.bril
```

Look for these markers in the output:

- `@saxpy`
- `__cuda_threadIdx_x: int`
- `__cuda_blockIdx_x: int`
- `alloc`
- `ptradd`
- `load`
- `store`

### 2. Observe RVSDG SVG

```bash
cd ~/Repos/egg_related/eggcc_rlcr
bash scripts/with-local-build-env.sh cargo +1.88.0 run -- \
  --run-mode rvsdg-conversion \
  examples/cuda/phase0/shared_saxpy.cu \
  > /tmp/phase0_shared.rvsdg.svg
open /tmp/phase0_shared.rvsdg.svg
```

### 3. Emit multiple debug artifacts

```bash
cd ~/Repos/egg_related/eggcc_rlcr
scripts/phase0-cuda-demo.sh debug-dir
```

This generates outputs such as:

- `.artifacts/phase0-cuda-demo/debug/shared_saxpy-rvsdg-conversion.svg`

The helper keeps emitted debug artifacts even if a later non-demo CLI path
still exits non-zero.

## Notes

- `--run-mode parse` is still the cleanest way to inspect the frontend boundary itself.
- Keep using `cargo +1.88.0 ...` because `rust-toolchain` is still pinned to `1.87.0`.
- `scripts/with-local-build-env.sh` wires the local LLVM 18 / build environment expected by this repo.
- If the helper fails on another machine or another worktree, inspect `.artifacts/phase0-cuda-demo/*.log` first.
- `debug-dir` can still trip a later non-demo CLI path, but the helper keeps the emitted debug SVGs and treats that case as a usable partial success.
