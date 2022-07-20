#
# The grand Makefile to Rule The Monorepo
#
# This Makefile is my huckleberry. It's certainly more than a little opaque,
# so I'll walk you through it. Essentially, you need to add your component
# to the list of components in COMPONENTS. From there, it will generate all
# the targets you need to build and test it.
#
# The real magic comes with targets defined as test//*/RDEPS. These should
# have dependencies that map to test//* targets that use this target in
# their own software, and want their tests to be run when the target has
# been updated.
#
# The special targets `build_from_git` and `test_from_git` will take the
# currently changed files in the repository, match them against the list
# of buildable/testable components, and run the tests - sweeping up any
# of the transitive dependencies we want.
#
# This is like a half-baked version of habitats rdeps work, things like
# mbt, and bazel. Lets see how far it gets us.

# We declare our path to the directory containing the root Makefile before other imports.
# This ensures that we have the accurate path to the root of the repository for our targets.
MAKEPATH := $(shell dirname $(realpath $(lastword $(MAKEFILE_LIST))))
CI := false
CI_FROM_REF := main
CI_TO_REF := HEAD
SHELL := /bin/bash

COMPONENTS = app/corp \
	app/web \
	bin/cyclone \
	bin/lang-js \
	bin/pinga \
	bin/sdf \
	bin/si-discord-bot \
	bin/veritech \
	lib/bytes-lines-codec \
	lib/cyclone \
	lib/deadpool-cyclone \
	lib/si-data \
	lib/si-test-macros \
	lib/veritech \
	lib/config-file \
	lib/dal \
	lib/sdf \
	lib/si-settings \
	lib/telemetry-rs 

RELEASEABLE_COMPONENTS = \
	component/nats \
	component/otelcol \
	component/postgres \
	bin/sdf \
	bin/veritech \
	bin/pinga \
	app/web

RUNABLE_COMPONENTS = \
	bin/veritech \
	bin/sdf \
	bin/pinga \
	app/web

BUILDABLE = $(patsubst %,build//%,$(COMPONENTS))
WATCHABLE = $(patsubst %,watch//%,$(COMPONENTS))
TESTABLE = $(patsubst %,test//%,$(COMPONENTS))
LINTABLE = $(patsubst %,lint//%,$(COMPONENTS))
CLEANABLE = $(patsubst %,clean//%,$(COMPONENTS))
RELEASEABLE = $(patsubst %,release//%,$(RELEASEABLE_COMPONENTS))
PROMOTABLE = $(patsubst %,promote//%,$(RELEASEABLE_COMPONENTS))
IMAGEABLE = $(patsubst %,image//%,$(RELEASEABLE_COMPONENTS))
RUNABLE = $(patsubst %,run//%,$(RUNABLE_COMPONENTS))

.DEFAULT_GOAL := help

.PHONY: $(BUILDABLE) $(TESTABLE) $(RELEASEABLE) $(IMAGEABLE) image release help

# @ echo "*** No test dependencies remaining for $@ ***"
%//TESTDEPS: ;

#@ echo "*** No reverse test dependencies remaining for $@ ***"
%//RTESTDEPS: ;

#@ echo "*** No build dependencies remaining for $@ ***"
%//BUILDDEPS: ;

$(WATCHABLE):
	@ pushd $(patsubst watch//%,%,$@); $(MAKE) watch

$(IMAGEABLE):
	cd $(patsubst image//%,%,$@) && $(MAKE) image

$(RELEASEABLE):
	 cd $(patsubst release//%,%,$@) && $(MAKE) release

$(PROMOTABLE):
	 cd $(patsubst promote//%,%,$@) && $(MAKE) promote

$(CLEANABLE):
	@cd $(patsubst clean//%,%,$@); $(MAKE) clean

$(LINTABLE):
	@echo "::group::$@"
	@cd $(patsubst lint//%,%,$@); $(MAKE) lint
	@echo "::endgroup::"

%//BUILD:
	@echo "::group::$@"
	@pushd $(patsubst build//%//BUILD,%,$@); $(MAKE) build
	@echo "::endgroup::"

$(BUILDABLE): % : %//BUILDDEPS %//BUILD

build//lib/cyclone//BUILDDEPS: build//bin/lang-js

build//bin/cyclone//BUILDDEPS: build//bin/lang-js

build//bin/veritech//BUILDDEPS: build//bin/cyclone

build//bin/sdf//BUILDDEPS: build//bin/veritech

build//bin/pinga//BUILDDEPS: build//bin/cyclone

%//TEST:
	@echo "::group::$@"
	@pushd $(patsubst test//%//TEST,%,$@); $(MAKE) CI=$(CI) CI_FROM_REF=$(CI_FROM_REF) CI_TO_REF=$(CI_TO_REF) test
	@echo "::endgroup::"

$(TESTABLE): % : %//TESTDEPS %//TEST %//RTESTDEPS

test//lib/dal//TESTDEPS: build//bin/veritech deploy//partial
test//lib/dal//RTESTDEPS: test//lib/sdf

test//lib/sdf//TESTDEPS: deploy//partial
test//lib/sdf//RTESTDEPS: test//bin/sdf test//bin/pinga

test//lib/bytes-lines-codec//RTESTDEPS: test//lib/cyclone

test//bin/cyclone//TESTDEPS: build//bin/lang-js

test//lib/cyclone//TESTDEPS: deploy//partial build//bin/lang-js
test//lib/cyclone//RTESTDEPS: test//lib/veritech test//bin/cyclone

test//lib/deadpool-cyclone//RTESTDEPS: test//lib/veritech

test//lib/si-data//RTESTDEPS: test//lib/veritech test//lib/dal test//lib/sdf

test//lib/si-test-macros//RTESTDEPS: test//lib/dal//TEST test//lib/sdf//TEST

test//lib/veritech//TESTDEPS: deploy//partial build//bin/cyclone
test//lib/veritech//RTESTDEPS: test//lib/dal//TEST test//lib/sdf//TEST test//bin/veritech 

test//lib/config-file//RTESTDEPS: test//lib/si-settings//TEST

test//lib/si-settings//RTESTDEPS: test//lib/veritech//TEST test//lib/cyclone//TEST test//lib/sdf//TEST

test//bin/lang-js//RTESTDEPS: test//lib/cyclone

test//app/web//TESTDEPS: build//app/web build//bin/pinga deploy//web

%//RUN:
	@echo "::group::$@"
	@pushd $(patsubst run//%//RUN,%,$@); $(MAKE) run
	@echo "::endgroup::"

run//app/web: run//app/web//RUN

$(RUNABLE): run//% : build//% run//%//RUN


test: $(TESTABLE)

build: $(BUILDABLE)

ci: 
	@echo $(CI_FROM_REF) $(CI_TO_REF)
	@$(MAKEPATH)/mk/test-changed.sh $(CI_FROM_REF) $(CI_TO_REF)
.PHONY: ci

deploy//web: deploy//down
	@echo "::group::deploy//web"
ifeq ($(CI),true)
	@echo "--- [$@]"
	@pushd $(MAKEPATH)/deploy; $(MAKE) CI_FROM_REF=$(CI_FROM_REF) CI_TO_REF=$(CI_TO_REF) web 
	@$(MAKEPATH)/ci/scripts/readiness-check.sh
else
	@echo "Skipping deploy//web outside of CI; set CI=true if you want this to happen automatically."
endif
	@echo "::endgroup::"

deploy//down: 
	@echo "::group::deploy//down"
ifeq ($(CI),true)
	@echo "--- [$@]"
	@pushd $(MAKEPATH)/deploy; $(MAKE) down
else
	@echo "Skipping deploy//down outside of CI; set CI=true if you want this to happen automatically."
endif
	@echo "::endgroup::"

deploy//partial: deploy//down
	@echo "::group::deploy//partial"
ifeq ($(CI),true)
	@echo "--- [$@]"
	@pushd $(MAKEPATH)/deploy; $(MAKE) partial
	@echo "Sleeping to not race postgres or the queue to being alive; you're welcome."
	@sleep 10 
else
	@echo "Skipping deploy//partial outside of CI; set CI=true if you want this to happen automatically."
endif
	@echo "::endgroup::"

clean: $(CLEANABLE)

force_clean:
	@echo "--- [$(shell basename ${CURDIR})] $@"
	sudo rm -rf ./target

# TODO(nick): The below targets are to be used during the transition period between the Vue 2 to
# Vue 3 rewrite. These targets should be merged into existing ones once the transition is complete.

down:
	$(MAKE) CI=true deploy//down
.PHONY: down

prepare: down
	$(MAKE) CI=true deploy//partial
.PHONY: prepare

troubleshoot: down
	cd $(MAKEPATH)/app/web; npm install
	cd $(MAKEPATH)/app/web; npm run vite-clean
	cd $(MAKEPATH); cargo clean
	$(MAKEPATH)/scripts/bootstrap.sh
.PHONY: troubleshoot

deploy-prod:
	@$(MAKEPATH)/scripts/deploy-prod.sh
.PHONY: deploy-prod

lint:
	cd $(MAKEPATH)/ci && $(MAKE) ci-lint
.PHONY: lint

tidy:
	cd $(MAKEPATH)/ci && $(MAKE) tidy
.PHONY: tidy

tidy-crates:
	cd $(MAKEPATH)/ci && $(MAKE) tidy-crates
.PHONY: tidy-crates

tidy-web:
	cd $(MAKEPATH)/ci && $(MAKE) tidy-web
.PHONY: tidy-web

docs-open:
	cd $(MAKEPATH); cargo doc --all
	cd $(MAKEPATH); cargo doc -p dal --open
.PHONY: docs-open

docs-watch:
	cd $(MAKEPATH); cargo watch -x doc
.PHONY: docs-watch

help:
	@echo "Check out DEVELOPING.md for more info"
