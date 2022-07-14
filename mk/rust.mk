WATCH_PATHS ?= .
WATCH_TASK ?= run
TEST_ARGS ?=
SI_LOG ?= info

build: ## Builds the Rust crate
	@echo "--- [$(shell basename ${CURDIR})] $@"
	cargo build
.PHONY: release

test: ## Tests the Rust crate
	@echo "--- [$(shell basename ${CURDIR})] $@"
	env RUST_BACKTRACE=1 cargo test --all-features -- $(TEST_ARGS)
.PHONY: release

run: ## Runs the default bin of the Rust crate
	@echo "--- [$(shell basename ${CURDIR})] $@"
	cargo run
.PHONY: run

start: run ## Alias for `make run`
.PHONY: start

clean: ## Cleans all build/test temporary work files
	@echo "--- [$(shell basename ${CURDIR})] $@"
	cargo clean
.PHONY: clean

watch: ## Runs `cargo watch` for the Rust crate
	@echo "--- [$(shell basename ${CURDIR})] $@"
	env SI_LOG=$(SI_LOG) cargo watch \
		$(foreach path,$(WATCH_PATHS),-w $(path)) -x $(WATCH_TASK)
.PHONY: watch

lint:
	@echo "--- [$(shell basename ${CURDIR})] $@"
	cargo fmt --all -- --check
	cargo clippy --no-deps -- -D warnings
# RUSTDOCFLAGS="-Dwarnings" cargo doc --all
