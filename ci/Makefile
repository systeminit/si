MAKEPATH := $(shell dirname $(realpath $(lastword $(MAKEFILE_LIST))))
REPOPATH := $(shell dirname $(MAKEPATH))

.DEFAULT_GOAL = ci-rust-lint

# ┌────────────────────────────────────────────────────────┐
# │ Makefile Legend                                        │
# ├─────────┬──────────────────────────────────────────────┤
# │ ci-*    │ directly executed in CI and locally          │
# │ setup-* │ dependencies for ci-* targets                │
# │ *       │ other targets intended for local development │
# └─────────┴──────────────────────────────────────────────┘

ci-rust-lint:
	cd $(REPOPATH); cargo fmt --all -- --check
	cd $(REPOPATH); cargo clippy -- -D warnings
.PHONY: ci-rust-lint

ci-crates: setup-crates
	cd $(REPOPATH); cargo test
.PHONY: ci-crates

ci-web: setup-web
	$(MAKEPATH)/scripts/readiness-check.sh
	cd $(REPOPATH)/app/web; env CYPRESS_BASE_URL="https://max:L0vesMiriya!@localhost" npm run ci
.PHONY: ci-web

setup-crates:
	$(MAKEPATH)/scripts/init-compose-env.sh
	cd $(REPOPATH)/bin/lang-js; npm ci && npm run package
	cd $(REPOPATH)/deploy && $(MAKE) partial
	cd $(REPOPATH); cargo build
.PHONY: setup-crates

setup-web:
	$(MAKEPATH)/scripts/init-compose-env.sh
	cd $(REPOPATH)/bin/lang-js; npm ci
	cd $(REPOPATH)/app/web; npm ci && env VUE_APP_SDF_BASE_HTTP_URL="https://localhost/api" \
		VUE_APP_SDF_BASE_WS_URL="wss://localhost/api/ws/billing_account_updates" \
		npm run build;
	cd $(REPOPATH); cargo build
	cd $(REPOPATH)/deploy && $(MAKE) ci
.PHONY: setup-web

teardown:
	cd $(REPOPATH)/deploy && $(MAKE) down
.PHONY: teardown

# TODO(nick): consider adding --all-targets --all-features to clippy
prepare:
	cd $(REPOPATH); cargo fix --edition-idioms --allow-dirty --allow-staged
	cd $(REPOPATH); cargo fmt --all
	cd $(REPOPATH); cargo clippy --fix --allow-dirty --allow-staged
.PHONY: prepare
