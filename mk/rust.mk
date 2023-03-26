WATCH_PATHS ?= .
WATCH_TASK ?= clippy
SI_TEST_ARGS ?=
SI_RUN_ARGS ?=
SI_TEST_FILTER ?=

PKGID = $(shell cargo pkgid --offline --quiet)
WORKSPACE_ROOT = $(shell cargo metadata --offline --no-deps --quiet \
		 | jq -r .workspace_root)

## clean: Cleans all build/test temporary work files
default--clean:
	$(call header,$@)
	cd $(WORKSPACE_ROOT)
	@# cargo clean will warn about ignoring the version and url qualifiers
	@# on a full pkgid spec so we'll determine the short, versionless pkgid
	@# spec and use that
	cargo clean --package "$$(echo $(PKGID) \
		| awk -F '#|@' '{\
			if ($$0 ~ /^.+#[^#]+@[^@]+$$/) {print $$2} \
			else {print $$1} \
		}' \
		| xargs basename)"
.PHONY: default--clean

## build: Builds the Rust crate
default--build:
	$(call header,$@)
	cd $(WORKSPACE_ROOT)
	cargo build --package $(PKGID)
.PHONY: default--build

## check-lint: Checks all code and doc linting for the Rust crate
default--check-lint:
	$(call header,$@)
	cd $(WORKSPACE_ROOT)
	cargo clippy --package $(PKGID) \
		--no-deps --all-targets -- -D warnings
.PHONY: default--check-lint

## check-doc: Checks all documentation for the Rust crate
default--check-doc:
	$(call header,$@)
	cd $(WORKSPACE_ROOT)
	env RUSTDOCFLAGS="-Dwarnings" cargo doc --package $(PKGID) \
		--no-deps --document-private-items
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
	cd $(WORKSPACE_ROOT)
	cargo fix --package $(PKGID) \
		--edition-idioms --allow-dirty --allow-staged
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
	cd $(WORKSPACE_ROOT)
	@# cargo run doesn't seem to honor the full pkid spec so we'll determine
	@# the short, versionless  pkgid spec and use that
	cargo run --package "$$(echo $(PKGID) \
		| awk -F '#|@' '{\
			if ($$0 ~ /^.+#[^#]+@[^@]+$$/) {print $$2} \
			else {print $$1} \
		}' \
		| xargs basename)" -- $(SI_RUN_ARGS)
.PHONY: default--run

## watch: Runs `cargo watch` for the Rust crate
default--watch:
	$(call header,$@)
	cargo watch \
		$(foreach path,$(WATCH_PATHS),-w $(path)) -x $(WATCH_TASK)
.PHONY: default--watch

## test: Tests the Rust crate
default--test:
	$(call header,$@)
	cd $(WORKSPACE_ROOT)
	env RUST_BACKTRACE=1 cargo test --package $(PKGID) \
		$(SI_TEST_FILTER) -- $(SI_TEST_ARGS)
.PHONY: default--test

## prepush: Runs all checks & tests required before pushing commits
default--prepush: check test
.PHONY: default--prepush

# Thanks to:
# https://newbedev.com/make-file-warning-overriding-commands-for-target
%: default--%
	@true
