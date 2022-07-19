include ../../mk/base.mk
include ../../mk/rust.mk

.DEFAULT_GOAL := build

lint:
	@echo "--- [$(shell basename ${CURDIR})] $@"
	cargo fmt --all -- --check
	## clippy disabled, because it is failing
	# cargo clippy -- -D warnings
	#RUSTDOCFLAGS="-Dwarnings" cargo doc --all

