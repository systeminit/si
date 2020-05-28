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

include ./components/build/deps.mk

COMPONENTS = components/si-data components/si-account components/si-settings components/si-graphql-api components/si-web-app components/si-external-api-gateway components/si-kubernetes components/si-registry
RELEASEABLE_COMPONENTS = components/si-account components/si-graphql-api components/si-external-api-gateway components/si-kubernetes
RUNNABLE_COMPONENTS = components/si-account components/si-graphql-api components/si-web-app components/si-external-api-gateway components/si-kubernetes components/si-registry
BUILDABLE = $(patsubst %,build//%,$(COMPONENTS))
TESTABLE = $(patsubst %,test//%,$(COMPONENTS))
RELEASEABLE = $(patsubst %,release//%,$(RELEASEABLE_COMPONENTS))
CONTAINABLE = $(patsubst %,container//%,$(RELEASEABLE_COMPONENTS))
WATCHABLE = $(patsubst %,watch//%,$(RUNNABLE_COMPONENTS))
BUILDABLE_REGEX = $(shell echo $(COMPONENTS) | tr " " "|")
RELEASEABLE_REGEX = $(shell echo $(RELEASEABLE_COMPONENTS) | tr " " "|")
TO_BUILD=$(shell git diff --name-only origin/master...HEAD | grep -E "^($(BUILDABLE_REGEX))" | cut -d "/" -f 1,2 | sort | uniq | tr "\n" " ")

GITHUB_SHA := HEAD

TO_RELEASE=$(shell git diff --name-only HEAD^ | grep -E "^($(RELEASEABLE_REGEX))" | cut -d "/" -f 1,2 | sort | uniq | tr "\n" " ")

RELEASE := $(shell date +%Y%m%d%H%M%S)

.DEFAULT_GOAL := build

.PHONY: $(BUILDABLE) $(TESTABLE) $(RELEASEABLE) $(CONTAINABLE)

test//components/si-data//RDEPS: test//components/si-account test//components/si-ssh-key

test//components/si-account//RDEPS: test//components/si-graphql-api

test//components/si-settings//RDEPS: test//components/si-data test//components/si-account

%//RDEPS:
	@ echo "*** No dependencies for $@ ***"

$(BUILDABLE): 
	@ pushd $(patsubst build//%,%,$@); $(MAKE) 

build: $(BUILDABLE)

build_from_git: $(patsubst %,build//%,$(TO_BUILD))

$(TESTABLE): % : %//RDEPS
ifdef TEST_IN_CONTAINERS
	@ pushd $(patsubst test//%,%,$@); $(MAKE) test_container RELEASE=$(RELEASE)
else
	@ pushd $(patsubst test//%,%,$@); $(MAKE) test
endif

test: $(TESTABLE)

test_from_git: $(patsubst %,test//%,$(TO_BUILD))

$(CONTAINABLE): clean
	@ pushd $(patsubst container//%,%,$@); $(MAKE) container RELEASE=$(RELEASE)

$(RELEASEABLE): clean
	@ pushd $(patsubst release//%,%,$@); $(MAKE) release RELEASE=$(RELEASE)

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

container//base: clean
	env BUILDKIT_PROGRESS=plain DOCKER_BUILDKIT=1 docker build \
		-f $(CURDIR)/components/build/Dockerfile-base \
		-t si-base:latest \
		-t si-base:$(RELEASE) \
		-t docker.pkg.github.com/systeminit/si/si-base:latest \
		.

release//base: container//base
	docker push docker.pkg.github.com/systeminit/si/si-base:latest

release_from_git: $(patsubst %,release//%,$(TO_RELEASE))
	@ echo "--> You have (maybe) released the System Initative! <--"
	@ echo Released: $(TO_RELEASE)


release: $(RELEASEABLE)
	@ echo "--> You have released the System Initative! <--"

clean:
	rm -rf ./components/*/node_modules
	rm -rf ./components/*/target
	rm -rf ./target

force_clean:
	sudo rm -rf ./components/*/node_modules
	sudo rm -rf ./target

dev_deps:
	./components/couchbase/run.sh || docker start db; exit 0
	./components/vernemq/run.sh || docker start vernemq; exit 0
	./components/opentelemetry-collector/run.sh || docker start otelcol; exit 0
