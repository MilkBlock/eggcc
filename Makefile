.PHONY: test test-clean nits nightly runtime fixnits

LLVM18_PREFIX ?= /opt/homebrew/opt/llvm@18
CBC_PREFIX ?= /opt/homebrew/opt/cbc
ZSTD_PREFIX ?= /opt/homebrew/opt/zstd
HOMEBREW_LIB ?= /opt/homebrew/lib
RUNTIME_RUST_TOOLCHAIN ?= nightly-2024-05-02

LLVM_ENV = PATH="$(LLVM18_PREFIX)/bin:$${PATH}" LLVM_SYS_180_PREFIX="$(LLVM18_PREFIX)"
LINK_ENV = PKG_CONFIG_PATH="$(CBC_PREFIX)/lib/pkgconfig:$(ZSTD_PREFIX)/lib/pkgconfig:$${PKG_CONFIG_PATH}" LIBRARY_PATH="$(HOMEBREW_LIB):$(CBC_PREFIX)/lib:$(ZSTD_PREFIX)/lib:$${LIBRARY_PATH}" DYLD_FALLBACK_LIBRARY_PATH="$(HOMEBREW_LIB):$(CBC_PREFIX)/lib:$(ZSTD_PREFIX)/lib:$${DYLD_FALLBACK_LIBRARY_PATH}"
BUILD_ENV = $(LLVM_ENV) $(LINK_ENV)

DIRS = . dag_in_context

all: nits test

test: runtime
	env $(BUILD_ENV) cargo insta test --release --unreferenced=reject
	cd dag_in_context && env $(BUILD_ENV) cargo insta test --release --unreferenced=reject

test-clean: runtime
	env $(BUILD_ENV) cargo insta test --release --unreferenced=delete
	cd dag_in_context && env $(BUILD_ENV) cargo insta test --release --unreferenced=delete

nits:
	bash scripts/prettier-check.sh
	@rustup component list --installed | grep -q '^clippy' || (echo "clippy component is required; install it with: rustup component add clippy" >&2; exit 1)
	@rustup component list --installed | grep -q '^rustfmt' || (echo "rustfmt component is required; install it with: rustup component add rustfmt" >&2; exit 1)
	env $(BUILD_ENV) cargo clippy --tests -- -D warnings && env $(BUILD_ENV) cargo fmt --check
	cd dag_in_context && env $(BUILD_ENV) cargo clippy --tests -- -D warnings && env $(BUILD_ENV) cargo fmt --check


fixnits:
	bash scripts/prettier-write.sh
	env $(BUILD_ENV) cargo fmt
	cd dag_in_context && env $(BUILD_ENV) cargo fmt
	env $(BUILD_ENV) cargo clippy --fix --allow-dirty
	cd dag_in_context && env $(BUILD_ENV) cargo clippy --fix --allow-dirty

# build the llvm runtime for bril
# if you edit the runtime crate, you must re-run this to rebuild rt.bc
runtime:
	env $(LLVM_ENV) RUNTIME_RUST_TOOLCHAIN="$(RUNTIME_RUST_TOOLCHAIN)" bash runtime/install.sh

nightly:
	bash infra/nightly.sh "benchmarks/passing"
