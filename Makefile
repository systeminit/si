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

# We declare our path to the directory containing the root Makefile before
# other imports. This ensures that we have the accurate path to the root of the
# repository for our targets.

MAKEPATH := $(shell dirname $(realpath $(lastword $(MAKEFILE_LIST))))
THIS_FILE := $(lastword $(MAKEFILE_LIST))
include mk/base.mk

CI := false
CI_FROM_REF := main
CI_TO_REF := HEAD
FORCE := false
SHELL := /bin/bash
LOCAL_PG := false

.DEFAULT_GOAL := help

COMPONENTS = \
	app/corp \
	app/web \
	bin/cyclone \
	bin/lang-js \
	bin/pinga \
	bin/sdf \
	bin/si-discord-bot \
	bin/veritech \
	lib/bytes-lines-codec \
	lib/config-file \
	lib/cyclone-client \
	lib/cyclone-core \
	lib/cyclone-server \
	lib/dal \
	lib/dal-test \
	lib/deadpool-cyclone \
	lib/pinga-server \
	lib/sdf \
	lib/si-data-faktory \
	lib/si-data-nats \
	lib/si-data-pg \
	lib/si-settings \
	lib/si-std \
	lib/si-test-macros \
	lib/telemetry-application-rs \
	lib/telemetry-rs \
	lib/veritech-client \
	lib/veritech-core \
	lib/veritech-server

RELEASEABLE_COMPONENTS = \
	app/web \
	bin/pinga \
	bin/sdf \
	bin/veritech \
	component/nats \
	component/otelcol \
	component/postgres

RUNABLE_COMPONENTS = \
	app/web \
	bin/pinga \
	bin/sdf \
	bin/veritech

BUILDABLE = $(patsubst %,build//%,$(COMPONENTS))
CHECKABLE = $(patsubst %,check//%,$(COMPONENTS))
CLEANABLE = $(patsubst %,clean//%,$(COMPONENTS))
FIXABLE = $(patsubst %,fix//%,$(COMPONENTS))
IMAGEABLE = $(patsubst %,image//%,$(RELEASEABLE_COMPONENTS))
PREPUSHABLE = $(patsubst %,prepush//%,$(COMPONENTS))
PROMOTABLE = $(patsubst %,promote//%,$(RELEASEABLE_COMPONENTS))
RELEASEABLE = $(patsubst %,release//%,$(RELEASEABLE_COMPONENTS))
RUNABLE = $(patsubst %,run//%,$(RUNABLE_COMPONENTS))
TESTABLE = $(patsubst %,test//%,$(COMPONENTS))
WATCHABLE = $(patsubst %,watch//%,$(COMPONENTS))

## ci: Invokes the primary continuous integration task
ci:
	$(call header,$@)
	@echo "    CI_FROM_REF='$(CI_FROM_REF)'"
	@echo "    CI_TO_REF='$(CI_TO_REF)'"
	$(MAKEPATH)/mk/test-changed.sh $(CI_FROM_REF) $(CI_TO_REF)
.PHONY: ci

## build: Builds all components
build: $(BUILDABLE)
.PHONY: build

## build//<cmpt>: Builds <cmpt>
$(BUILDABLE): % : %//BUILDDEPS %//BUILD
.PHONY: $(BUILDABLE)

build//bin/cyclone//BUILDDEPS: build//bin/lang-js
build//bin/veritech//BUILDDEPS: build//bin/cyclone
build//lib/cyclone-server//BUILDDEPS: build//bin/lang-js
build//lib/veritech-server//BUILDDEPS: build//bin/cyclone

#@ echo "*** No build dependencies remaining for $@ ***"
%//BUILDDEPS: ;

## build//<cmpt>//BUILD: Skips build dependencies & builds <cmpt>
%//BUILD:
ifeq ($(CI),true)
	@echo "::group::make $@"
endif
	@cd $(patsubst build//%//BUILD,%,$@); $(MAKE) build
ifeq ($(CI),true)
	@echo "::endgroup::"
endif

## check: Checks all components' linting, formatting, & other rules
check: $(CHECKABLE)
.PHONY: check

## check//<cmpt>: Checks all linting, formatting & other rules for <cmpt>
$(CHECKABLE):
ifeq ($(CI),true)
	@echo "::group::make $@"
endif
	@cd $(patsubst check//%,%,$@); $(MAKE) check
ifeq ($(CI),true)
	@echo "::endgroup::"
endif
.PHONY: $(CHECKABLE)

## clean: Cleans all build/test temporary work files
clean: $(CLEANABLE)
.PHONY: $(CLEANABLE)

## clean//<cmpt>: Cleans all build/test temporary files for <cmpt>
$(CLEANABLE):
	@cd $(patsubst clean//%,%,$@); $(MAKE) clean
.PHONY: $(CLEANABLE)

## fix: Updates all linting fixes & formatting for all components (may modify sources)
fix: $(FIXABLE)
.PHONY: fix

## fix//<cmpt>: Updates all linting fixes & formatting for <cmpt> (may modify sources)
$(FIXABLE):
	@cd $(patsubst fix//%,%,$@); $(MAKE) fix
.PHONY: $(FIXABLE)

## image: Builds all container images for relevant components
image: $(IMAGEABLE)
.PHONY: image

## image//<cmpt>: Builds the container for <cmpt>
$(IMAGEABLE):
	@cd $(patsubst image//%,%,$@) && $(MAKE) image
.PHONY: $(IMAGEABLE)

## prepush: Runs all checks & tests required before pushing commits
prepush: check test
.PHONY: prepush

## prepush//<cmpt>: Runs all checks & tests required before pushing commits for <cmpt>
$(PREPUSHABLE):
	@cd $(patsubst prepush//%,%,$@); $(MAKE) prepush
.PHONY: $(PREPUSHABLE)

## promote//<cmpt>: Tags & pushes the current Git revision image as 'stable' for <cmpt>
$(PROMOTABLE):
	@cd $(patsubst promote//%,%,$@) && $(MAKE) promote
.PHONY: $(PROMOTABLE)

## release//<cmpt>: Builds & pushes the image for <cmpt> to the repository
$(RELEASEABLE):
	@cd $(patsubst release//%,%,$@) && $(MAKE) release
.PHONY: $(RELEASEABLE)

## run//<cmpt>: Runs the default program/server for <cmpt>
$(RUNABLE): run//% : build//% run//%//RUN
.PHONY: $(RUNABLE)

## run//<cmpt>//RUN: Skips build dependencies & runs <cmpt>
%//RUN:
ifeq ($(CI),true)
	@echo "::group::make $@"
endif
	@cd $(patsubst run//%//RUN,%,$@); $(MAKE) run
ifeq ($(CI),true)
	@echo "::endgroup::"
endif

## test: Tests all components
test: $(TESTABLE)
.PHONY: test

test//app/web//TESTDEPS: build//app/web

test//bin/cyclone//TESTDEPS: build//bin/lang-js
test//bin/cyclone//RTESTDEPS: test//lib/veritech-server

test//bin/lang-js//RTESTDEPS: test//lib/cyclone-server

test//lib/bytes-lines-codec//RTESTDEPS: test//lib/cyclone-server

test//lib/config-file//RTESTDEPS: test//lib/si-settings//TEST

test//lib/cyclone-client//TESTDEPS: build//bin/lang-js
test//lib/cyclone-client//RTESTDEPS: test//lib/deadpool-cyclone

test//lib/cyclone-server//TESTDEPS: build//bin/lang-js
test//lib/cyclone-server//RTESTDEPS: test//bin/cyclone test//lib/cyclone-client

test//lib/dal//TESTDEPS: build//bin/cyclone deploy//partial
test//lib/dal//RTESTDEPS: test//lib/sdf

test//lib/dal-test//RTESTDEPS: test//lib/dal//TEST test//lib/sdf//TEST

test//lib/deadpool-cyclone//TESTDEPS: build//bin/cyclone
test//lib/deadpool-cyclone//RTESTDEPS: test//lib/veritech-server

test//lib/pinga-server//RTESTDEPS: test//bin/pinga

test//lib/sdf//TESTDEPS: build//bin/cyclone deploy//partial
test//lib/sdf//RTESTDEPS: test//bin/sdf test//bin/pinga

test//lib/si-data-faktory//RTESTDEPS: test//lib/sdf test//lib/pinga-server

test//lib/si-data-nats//RTESTDEPS: test//lib/veritech-client test//lib/veritech-server test//lib/dal test//lib/pinga-server test//lib/sdf

test//lib/si-data-pg//RTESTDEPS: test//lib/dal test//lib/pinga-server test//lib/sdf

test//lib/si-settings//RTESTDEPS: test//lib/veritech-server//TEST test//lib/cyclone-server//TEST test//lib/pinga-server//TEST test//lib/sdf//TEST

test//lib/si-std//RTESTDEPS: test//lib/dal-test test//lib/sdf

test//lib/si-test-macros//RTESTDEPS: test//lib/dal//TEST test//lib/sdf//TEST

test//lib/telemetry-application-rs//RTESTDEPS: test//bin/cyclone test//bin/pinga test//bin/sdf test//bin/veritech

test//lib/veritech-client//TESTDEPS: build//bin/cyclone
test//lib/veritech-client//RTESTDEPS: test//lib/dal

test//lib/veritech-server//TESTDEPS: build//bin/cyclone
test//lib/veritech-server//RTESTDEPS: test//bin/veritech test//lib/veritech-client

# @ echo "*** No test dependencies remaining for $@ ***"
%//TESTDEPS: ;

#@ echo "*** No reverse test dependencies remaining for $@ ***"
%//RTESTDEPS: ;

## test//<cmpt>//TEST: Skips test dependencies & runs tests for <cmpt>
%//TEST:
ifeq ($(CI),true)
	@echo "::group::make $@"
endif
	@cd $(patsubst test//%//TEST,%,$@); $(MAKE) test
ifeq ($(CI),true)
	@echo "::endgroup::"
endif

## test//<cmpt>: Tests <cmpt>
$(TESTABLE): % : %//TESTDEPS %//TEST %//RTESTDEPS
.PHONY: $(TESTABLE)

## watch//<cmpt>: Runs the default watch task for <cmpt>
$(WATCHABLE):
	@cd $(patsubst watch//%,%,$@); $(MAKE) watch
.PHONY: $(WATCHABLE)

deploy//down:
ifeq ($(CI),true)
	@echo "::group::make $@"
endif
	$(call header,$@)
ifeq ($(shell [[ $(CI) == "true" || $(FORCE) == "true" ]] && echo "true"),true)
	cd $(MAKEPATH)/deploy; $(MAKE) down
else
	@echo "  - Skipping $@ outside of CI; set FORCE=true if you want this to happen automatically."
endif
ifeq ($(CI),true)
	@echo "::endgroup::"
endif
.PHONY: deploy//down

deploy//partial: deploy//down
ifeq ($(CI),true)
	@echo "::group::make $@"
endif
	$(call header,$@)
ifeq ($(shell [[ $(FORCE) == "true" && $(LOCAL_PG) == "true" ]] && echo "true"),true)
	cd $(MAKEPATH)/deploy; $(MAKE) partial-local-pg
	@echo "  - Sleeping to not race LOCAL postgres or the queue to being alive; you're welcome."
	@sleep 10
else ifeq ($(shell [[ $(CI) == "true" || $(FORCE) == "true" ]] && echo "true"),true)
	cd $(MAKEPATH)/deploy; $(MAKE) partial
	@echo "  - Sleeping to not race postgres or the queue to being alive; you're welcome."
	@sleep 10
else
	@echo "  - Skipping $@ outside of CI; set FORCE=true if you want this to happen automatically."
endif
ifeq ($(CI),true)
	@echo "::endgroup::"
endif
.PHONY: deploy//partial

deploy//web: deploy//down
ifeq ($(CI),true)
	@echo "::group::make $@"
endif
	$(call header,$@)
ifeq ($(shell [[ $(CI) == "true" || $(FORCE) == "true" ]] && echo "true"),true)
	cd $(MAKEPATH)/deploy; $(MAKE) CI_FROM_REF=$(CI_FROM_REF) CI_TO_REF=$(CI_TO_REF) web
	$(MAKEPATH)/ci/scripts/readiness-check.sh
else
	@echo "  - Skipping $@ outside of CI; set FORCE=true if you want this to happen automatically."
endif
ifeq ($(CI),true)
	@echo "::endgroup::"
endif
.PHONY: deploy//web

## deploy//prod: Manually deploy production (requires Tailscale & SSH key)
deploy//prod:
	$(call header,$@)
	$(MAKEPATH)/scripts/deploy-prod.sh
.PHONY: deploy//prod

## prepare: Prepares supporting services for development (warning: deletes local database state)
prepare:
	$(MAKE) FORCE=true deploy//partial
.PHONY: prepare

## down: Brings down supporting services (warning: deletes local database state)
down:
	$(MAKE) FORCE=true deploy//down
.PHONY: down

## list: Prints a comprehensive list of Makefile targets
#
# Thanks to: https://stackoverflow.com/a/26339924
list:
	@LC_ALL=C $(MAKE) -pRrq -f $(THIS_FILE) : 2>/dev/null | awk -v RS= -F: '/(^|\n)# Files(\n|$$)/,/(^|\n)# Finished Make data base/ {if ($$1 !~ "^[#.]") {print $$1}}' | sort | egrep -v -e '^[^[:alnum:]]' -e '^$@$$'
.PHONY: list
