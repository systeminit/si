WATCH_PATHS ?= .
WATCH_TASK ?= clippy
TEST_ARGS ?=
TEST_FILTER ?=
SI_LOG ?= info

## clean: Cleans all build/test temporary work files
default--clean:
	$(call header,$@)
	cargo clean
.PHONY: default--clean

## build: Builds the Rust crate
default--build:
	$(call header,$@)
	cargo build
.PHONY: default--build

## check-lint: Checks all code and doc linting for the Rust crate
default--check-lint:
	$(call header,$@)
	cargo clippy --no-deps --all-targets -- -D warnings
.PHONY: default--check-lint

## check-doc: Checks all documentation for the Rust crate
default--check-doc:
	$(call header,$@)
	env RUSTDOCFLAGS="-Dwarnings" cargo doc --no-deps --document-private-items
.PHONY: default--check-doc

## check-format: Checks all code formatting for the Rust crate
default--check-format:
	$(call header,$@)
	cargo fmt -- --check
.PHONY: default--check-format

## check: Checks all linting, formatting, & other rules
default--check: check-format check-lint check-doc
.PHONY: default--check

## fix-lint: Updates code with linting fixes for the crate (may modify sources)
default--fix-lint:
	$(call header,$@)
	cargo fix --edition-idioms --allow-dirty --allow-staged
.PHONY: default--fix-lint

## fix-format: Updates code formatting for the crate (may modify sources)
default--fix-format:
	$(call header,$@)
	cargo fmt
.PHONY: default--fix-format

## fix: Updates all linting fixes & formatting for the crate (may modify sources)
default--fix: fix-format fix-lint
.PHONY: default--fix

## start: Alias for `make run`
default--start: run
.PHONY: default--start

## run: Runs the default bin of the Rust crate
default--run:
	$(call header,$@)
	cargo run
.PHONY: default--run

## watch: Runs `cargo watch` for the Rust crate
default--watch:
	$(call header,$@)
	env SI_LOG=$(SI_LOG) cargo watch \
		$(foreach path,$(WATCH_PATHS),-w $(path)) -x $(WATCH_TASK)
.PHONY: default--watch

## test: Tests the Rust crate
default--test:
	$(call header,$@)
	env RUST_BACKTRACE=1 cargo test $(TEST_FILTER) -- $(TEST_ARGS)
.PHONY: default--test

## prepush: Runs all checks & tests required before pushing commits
default--prepush: check test
.PHONY: default--prepush

# Thanks to:
# https://newbedev.com/make-file-warning-overriding-commands-for-target
%: default--%
	@true
