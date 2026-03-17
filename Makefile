.PHONY: test test-clean nits nightly runtime fixnits

RUNTIME_RUST_TOOLCHAIN ?= nightly-2024-05-02

DIRS = . dag_in_context

all: nits test

test: runtime
	bash scripts/with-local-build-env.sh cargo insta test --release --unreferenced=reject
	cd dag_in_context && bash ../scripts/with-local-build-env.sh cargo insta test --release --unreferenced=reject

test-clean: runtime
	bash scripts/with-local-build-env.sh cargo insta test --release --unreferenced=delete
	cd dag_in_context && bash ../scripts/with-local-build-env.sh cargo insta test --release --unreferenced=delete

nits:
	bash scripts/prettier-check.sh
	@rustup component list --installed | grep -q '^clippy' || (echo "clippy component is required; install it with: rustup component add clippy" >&2; exit 1)
	@rustup component list --installed | grep -q '^rustfmt' || (echo "rustfmt component is required; install it with: rustup component add rustfmt" >&2; exit 1)
	bash scripts/with-local-build-env.sh cargo clippy --tests -- -D warnings && bash scripts/with-local-build-env.sh cargo fmt --check
	cd dag_in_context && bash ../scripts/with-local-build-env.sh cargo clippy --tests -- -D warnings && bash ../scripts/with-local-build-env.sh cargo fmt --check


fixnits:
	bash scripts/prettier-write.sh
	bash scripts/with-local-build-env.sh cargo fmt
	cd dag_in_context && bash ../scripts/with-local-build-env.sh cargo fmt
	bash scripts/with-local-build-env.sh cargo clippy --fix --allow-dirty
	cd dag_in_context && bash ../scripts/with-local-build-env.sh cargo clippy --fix --allow-dirty

# build the llvm runtime for bril
# if you edit the runtime crate, you must re-run this to rebuild rt.bc
runtime:
	RUNTIME_RUST_TOOLCHAIN="$(RUNTIME_RUST_TOOLCHAIN)" bash scripts/with-local-build-env.sh bash runtime/install.sh

nightly:
	bash infra/nightly.sh "benchmarks/passing"
