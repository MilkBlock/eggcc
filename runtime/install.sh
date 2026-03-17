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
# Prefer an already-installed toolchain when present, but allow rustup/cargo
# to install it in CI or on fresh machines where network access is available.
cargo +"$TOOLCHAIN" rustc --release -- --emit=llvm-bc
cp ./target/release/deps/runtime-*.bc ./rt.bc
cc -c rt.c -o rt.o
