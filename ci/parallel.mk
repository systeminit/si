# NOTE(nick): this Makefile has been deprecated. It can be restored once
# parallel CI tasks/jobs are desired once more.
#
#MAKEPATH := $(shell dirname $(realpath $(lastword $(MAKEFILE_LIST))))
#REPOPATH := $(shell dirname $(MAKEPATH))
#
#.DEFAULT_GOAL = ci-lint
#
## ┌────────────────────────────────────────────────────────┐
## │ Makefile Legend                                        │
## ├─────────┬──────────────────────────────────────────────┤
## │ ci-*    │ directly executed in CI and locally          │
## │ setup-* │ dependencies for ci-* targets                │
## │ *       │ other targets intended for local development │
## └─────────┴──────────────────────────────────────────────┘
#
#ci-lint: ci-rust-lint ci-web-lint
#.PHONY: ci-lint
#
#ci-rust-lint:
#	cd $(REPOPATH); cargo fmt --all -- --check
#	cd $(REPOPATH); cargo clippy -- -D warnings
#.PHONY: ci-rust-lint
#
#ci-web-lint:
#	cd $(REPOPATH)/app/web; npm run prettier-check
#	cd $(REPOPATH)/app/web; npm run type-check || exit 0
#	cd $(REPOPATH)/app/web; npm run eslint || exit 0
#.PHONY: ci-web-lint
#
#ci-crates: teardown setup-crates
#	cd $(REPOPATH); cargo test --all-features
#.PHONY: ci-crates
#
#ci-web: teardown setup-web
#	$(MAKEPATH)/scripts/readiness-check.sh
#	cd $(REPOPATH)/app/web; env CYPRESS_BASE_URL='https://max:L0vesMiriya!@localhost' npm run cypress:run || exit 0
#.PHONY: ci-web
#
#setup-crates:
#	$(MAKEPATH)/scripts/init-compose-env.sh
#	cd $(REPOPATH)/bin/lang-js; npm ci
#	cd $(REPOPATH)/bin/lang-js; npm run package
#	cd $(REPOPATH)/deploy && $(MAKE) partial
#	cd $(REPOPATH); cargo build
#.PHONY: setup-crates
#
#setup-web:
#	$(MAKEPATH)/scripts/init-compose-env.sh
#	cd $(REPOPATH)/bin/lang-js; npm ci
#	cd $(REPOPATH)/bin/lang-js; npm run package
#	cd $(REPOPATH)/app/web; npm ci
#	cd $(REPOPATH)/app/web; env VUE_APP_SDF_BASE_HTTP_URL="https://localhost/api" \
#		VUE_APP_SDF_BASE_WS_URL="wss://localhost/api/ws/billing_account_updates" \
#		npm run build || exit 0;
#	cd $(REPOPATH); cargo build
#	cd $(REPOPATH)/deploy && $(MAKE) ci
#.PHONY: setup-web
#
#teardown:
#	-cd $(REPOPATH)/deploy && $(MAKE) down
#.PHONY: teardown
#
## TODO(nick): consider adding --all-targets --all-features to clippy
#tidy:
#	cd $(REPOPATH); cargo fix --edition-idioms --allow-dirty --allow-staged
#	cd $(REPOPATH); cargo fmt --all
#	cd $(REPOPATH); cargo clippy
#.PHONY: tidy
