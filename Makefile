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

include ./components/build/deps.mk

# order is important as earlier components are needed to build later components
COMPONENTS = \
	components/si-inference \
	components/si-entity \
	components/si-registry \
	components/si-veritech \
	components/si-web-app \
	components/si-data \
	components/si-model \
	components/si-model-test \
	components/si-settings \
	components/si-sdf
RELEASEABLE_COMPONENTS = \
	web \
	veritech \
	sdf \
	nats \
	si-web-app
RUNNABLE_COMPONENTS = \
	bin/veritech \
	bin/sdf \
	components/si-web-app
BUILDABLE = $(patsubst %,build//%,$(COMPONENTS))
TESTABLE = $(patsubst %,test//%,$(COMPONENTS))
CLEANABLE = $(patsubst %,clean//%,$(COMPONENTS))
RELEASEABLE = $(patsubst %,release-%,$(RELEASEABLE_COMPONENTS))
PROMOTABLE = $(patsubst %,promote-%,$(RELEASEABLE_COMPONENTS))
IMAGEABLE = $(patsubst %,image//%,$(RELEASEABLE_COMPONENTS))
WATCHABLE = $(patsubst %,watch//%,$(RUNNABLE_COMPONENTS))
BUILDABLE_REGEX = $(shell echo $(COMPONENTS) | tr " " "|")
RELEASEABLE_REGEX = $(shell echo $(RELEASEABLE_COMPONENTS) | tr " " "|")

.DEFAULT_GOAL := prepare

.PHONY: $(BUILDABLE) $(TESTABLE) $(RELEASEABLE) $(IMAGEABLE) image release

test//components/si-sdf//RDEPS: test//components/si-settings test//components/si-model test//components/si-data test//components/si-model-test

test//components/si-model//RDEPS: test//components/si-settings test//components/si-data test//components/si-model-test

test//components/si-veritech//RDEPS: test//components/si-registry test//components/si-entity

test//components/si-entity//RDEPS: test//components/si-registry

test//components/si-web-app//RDEPS: test//components/si-registry test//components/si-entity

%//RDEPS:
	@ echo "*** No dependencies for $@ ***"

$(BUILDABLE):
	@ pushd $(patsubst build//%,%,$@); $(MAKE)

build: $(BUILDABLE)

$(TESTABLE): % : %//RDEPS
ifdef TEST_IN_CONTAINERS
	@ pushd $(patsubst test//%,%,$@); $(MAKE) test_container RELEASE=$(RELEASE)
else
	@ pushd $(patsubst test//%,%,$@); $(MAKE) test
endif

test: $(TESTABLE)

$(IMAGEABLE):
	cd $(patsubst image//%,%,$@) && $(MAKE) image

$(RELEASEABLE):
	cd bin/$(patsubst release-%,%,$@) && $(MAKE) release

$(PROMOTABLE):
	cd bin/$(patsubst promote-%,%,$@) && $(MAKE) promote

release-postgres:
	cd component/postgres && $(MAKE) release

release-nats:
	cd component/nats && $(MAKE) release

release-otelcol:
	cd component/otelcol && $(MAKE) release

release-web:
	cd app/web && $(MAKE) release

promote-postgres:
	cd component/postgres && $(MAKE) promote

promote-nats:
	cd component/nats && $(MAKE) promote

promote-otelcol:
	cd component/otelcol && $(MAKE) promote

promote-web:
	cd app/web && $(MAKE) promote

$(WATCHABLE):
	@ pushd $(patsubst watch//%,%,$@); $(MAKE) watch

watch:
	@ echo $(RUNNABLE_COMPONENTS) | tr ' ' '\n' | parallel --tag --jobs 0 --linebuffer -r make watch//{}

tmux: tmux//windows

tmux//windows: tmux_session tmux_windows
	@ echo "*** Starting magical tmux (windows) good times ***"

tmux//panes: tmux_session tmux_panes
	@ echo "*** Starting magical tmux (panes) good times ***"

tmux_session:
ifdef TMUX
	@ echo Not starting a new session, as you are in one.
else
	@ echo "*** Starting new tmux session ***"
	@ tmux -2 new-session -d -s si
	@ tmux send-keys "make dev_deps" C-m
	@ echo "tmux attach -t si"
endif

tmux_windows:
	@ for x in $(RUNNABLE_COMPONENTS); do tmux new-window -a -n $$(echo $$x | cut -f 2 -d '/') && tmux send-keys "make watch//$$x" C-m; done

tmux_panes:
	@ for x in $(RUNNABLE_COMPONENTS); do tmux split-window -v && tmux send-keys "make watch//$$x" C-m; done

build_release//cli:
	@echo "--- [$(shell basename ${CURDIR})] $@"
	@ pushd ./components/si-sdf; $(MAKE) $@

container//cli:
	@echo "--- [$(shell basename ${CURDIR})] $@"
	@cd ./components/si-sdf; $(MAKE) $@

deploy//internal: release
	@echo "--- [$(shell basename ${CURDIR})] $@"
	@cd components/aws-si-internal; env RELEASE=$(RELEASE) pulumi up -y

release: $(RELEASEABLE)
	@echo "--> You have released the System Initative! <--"

image: $(IMAGEABLE)

$(CLEANABLE):
	@cd $(patsubst clean//%,%,$@); $(MAKE) clean

clean: $(CLEANABLE)

force_clean:
	@echo "--- [$(shell basename ${CURDIR})] $@"
	sudo rm -rf ./components/*/node_modules
	sudo rm -rf ./target

run-dev-deps:
	@echo "--- [$(shell basename ${CURDIR})] $@"
	cd ./components/postgres && $(MAKE) run-container
	cd ./components/nats && $(MAKE) run-container
	cd ./components/otelcol && $(MAKE) run-container
.PHONY: run-dev-deps

run-containers: run-dev-deps
	@echo "--- [$(shell basename ${CURDIR})] $@"
	cd ./components/si-veritech && $(MAKE) run-container
	cd ./components/si-sdf && $(MAKE) run-container
	cd ./components/si-web-app && $(MAKE) run-container
.PHONY: run-containers

run-test-deps: run-dev-deps
.PHONY: run-test-deps

stop-dev-deps:
	@echo "--- [$(shell basename ${CURDIR})] $@"
	cd ./components/postgres && $(MAKE) stop-container
	cd ./components/nats && $(MAKE) stop-container
	cd ./components/otelcol && $(MAKE) stop-container

stop-containers: stop-dev-deps
	@echo "--- [$(shell basename ${CURDIR})] $@"
	cd ./components/si-veritech && $(MAKE) stop-container
	cd ./components/si-sdf && $(MAKE) stop-container
	cd ./components/si-web-app && $(MAKE) stop-container
.PHONY: stop-containers

clean-dev-deps:
	@echo "--- [$(shell basename ${CURDIR})] $@"
	cd ./components/postgres && $(MAKE) clean-container
	cd ./components/nats && $(MAKE) clean-container
	cd ./components/otelcol && $(MAKE) clean-container

clean-containers: clean-dev-deps
	@echo "--- [$(shell basename ${CURDIR})] $@"
	cd ./components/si-veritech && $(MAKE) clean-container
	cd ./components/si-sdf && $(MAKE) clean-container
	cd ./components/si-web-app && $(MAKE) clean-container
.PHONY: clean-containers

# TODO(nick): The below targets are to be used during the transition period between the
# Vue 2 to Vue 3 rewrite. These targets should be merged into existing ones once the transition is
# complete.

down:
	-cd $(MAKEPATH)/deploy && $(MAKE) down
.PHONY: down

prepare: down
	cd $(MAKEPATH)/bin/lang-js; npm install
	cd $(MAKEPATH)/bin/lang-js; npm run package
	cd $(MAKEPATH)/deploy && $(MAKE) partial
.PHONY: prepare

backend: down
	cd $(MAKEPATH)/bin/lang-js; npm install
	cd $(MAKEPATH)/bin/lang-js; npm run package
	cd $(MAKEPATH)/deploy && $(MAKE) backend
.PHONY: backend

veritech-run:
	cd $(MAKEPATH); cargo build
	cd $(MAKEPATH); cargo build --bin cyclone
	cd $(MAKEPATH)/bin/veritech; cargo run
.PHONY: veritech-run

sdf-run:
	cd $(MAKEPATH); cargo build
	cd $(MAKEPATH); cargo run --bin sdf -- --disable-opentelemetry
.PHONY: sdf-run

app-run:
	cd $(MAKEPATH)/app/web; npm install
	cd $(MAKEPATH)/app/web; npm run vite-clean
	cd $(MAKEPATH)/app/web; npm run dev
.PHONY: app-run

cargo-clean:
	cd $(MAKEPATH); cargo clean
.PHONY: clean-cargo

deploy-prod:
	@$(MAKEPATH)/scripts/deploy-prod.sh
.PHONY: deploy-prod
