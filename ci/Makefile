MAKEPATH := $(shell dirname $(realpath $(lastword $(MAKEFILE_LIST))))
REPOPATH := $(shell dirname $(MAKEPATH))

.DEFAULT_GOAL = ci-lint

# ┌────────────────────────────────────────────────────────┐
# │ Makefile Legend                                        │
# ├─────────┬──────────────────────────────────────────────┤
# │ ci-*    │ directly executed in CI and locally          │
# │ setup-* │ dependencies for ci-* targets                │
# │ *       │ other targets intended for local development │
# └─────────┴──────────────────────────────────────────────┘

ci-lint: ci-rust-lint ci-web-lint
.PHONY: ci-lint

ci-rust-lint:
	cd $(REPOPATH); cargo fmt --all -- --check
	cd $(REPOPATH); cargo clippy -- -D warnings
	cd $(REPOPATH); RUSTDOCFLAGS="-Dwarnings" cargo doc --all
.PHONY: ci-rust-lint

ci-web-lint:
	cd $(REPOPATH)/app/web; npm run prettier-check
	cd $(REPOPATH)/app/web; npm run type-check
	cd $(REPOPATH)/app/web; npm run eslint
.PHONY: ci-web-lint

ci-si: setup-base ci-crates ci-web
.PHONY: ci-si

ci-crates: down
	cd $(REPOPATH)/deploy && $(MAKE) partial
	cd $(REPOPATH); cargo test --all-features
.PHONY: ci-crates

ci-web: down
	cd $(REPOPATH)/app/web; npm ci
	cd $(REPOPATH)/app/web; env VUE_APP_SDF_BASE_HTTP_URL="https://localhost/api" \
		VUE_APP_SDF_BASE_WS_URL="wss://localhost/api/ws/billing_account_updates" \
		npm run build;
	cd $(REPOPATH)/deploy && $(MAKE) ci
	$(MAKEPATH)/scripts/readiness-check.sh
	cd $(REPOPATH)/app/web; env CYPRESS_BASE_URL='https://max:L0vesMiriya!@localhost' npm run cypress:run || exit 0
.PHONY: ci-web

setup-base: down
	$(MAKEPATH)/scripts/init-compose-env.sh
	cd $(REPOPATH)/bin/lang-js; npm ci
	cd $(REPOPATH)/bin/lang-js; npm run package
	cd $(REPOPATH); cargo build
.PHONY: setup-base

down:
	-cd $(REPOPATH)/deploy && $(MAKE) down
.PHONY: down

tidy: tidy-web tidy-crates
.PHONY: tidy

# TODO(nick): consider adding --all-targets --all-features to clippy
tidy-crates:
	cd $(REPOPATH); cargo fix --edition-idioms --allow-dirty --allow-staged
	cd $(REPOPATH); cargo fmt --all
	cd $(REPOPATH); cargo doc
	cd $(REPOPATH); cargo clippy
.PHONY: tidy-crates

tidy-web:
	cd $(REPOPATH)/app/web/; npm run format
	cd $(REPOPATH)/app/web/; npm run eslint:fix
.PHONY: tidy-web
