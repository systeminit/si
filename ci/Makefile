MAKEPATH := $(shell dirname $(realpath $(lastword $(MAKEFILE_LIST))))
REPOROOT := $(shell dirname $(MAKEPATH))

ci-rust:
	cd $(REPOROOT); cargo fmt --all -- --check
	cd $(REPOROOT); cargo clippy -- -D warnings
.PHONY: ci-rust

ci:
	cd $(REPOROOT)/bin/lang-js; npm install
	cd $(REPOROOT)/bin/lang-js; npm run package
	$(MAKEPATH)/scripts/init-compose-env.sh
	cd $(REPOROOT)/deploy && $(MAKE) partial-detached
	cd $(REPOROOT); cargo build
	cd $(REPOROOT); cargo test
.PHONY: ci

# TODO(nick): consider adding --all-targets --all-features to clippy
ci-prepare:
	cd $(REPOROOT); cargo fix --edition-idioms --allow-dirty --allow-staged
	cd $(REPOROOT); cargo fmt --all
	cd $(REPOROOT); cargo clippy --fix --allow-dirty --allow-staged
.PHONY: ci-prepare