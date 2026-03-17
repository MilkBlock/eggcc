#!/bin/bash

echo "Building runtime.bc and rt.o"

set -euo pipefail

TOOLCHAIN="${RUNTIME_RUST_TOOLCHAIN:-nightly-2024-05-02}"

# remove rt.bc if it exists
if [ -f runtime/rt.bc ]; then
    rm runtime/rt.bc
fi

if [ -f runtime/rt.o ]; then
    rm runtime/rt.o
fi

cd runtime
# Duplicate runtime.bc files can mess things up,
# so make sure we start from a clean slate.
cargo clean
# Use an installed LLVM-18 toolchain and fail fast instead of triggering rustup downloads.
if ! rustup which --toolchain "$TOOLCHAIN" cargo >/dev/null 2>&1; then
    echo "Rust toolchain '$TOOLCHAIN' is not installed." >&2
    echo "Install it first or override RUNTIME_RUST_TOOLCHAIN to an installed LLVM-18 toolchain." >&2
    exit 1
fi

cargo +"$TOOLCHAIN" rustc --release -- --emit=llvm-bc
cp ./target/release/deps/runtime-*.bc ./rt.bc
cc -c rt.c -o rt.o
