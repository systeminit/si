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

COMPONENTS = components/si-data components/si-account components/si-settings components/si-graphql-api components/si-web-ui
BUILDABLE = $(patsubst %,build//%,$(COMPONENTS))
TESTABLE = $(patsubst %,test//%,$(COMPONENTS))
BUILDABLE_REGEX = $(shell echo $(COMPONENTS) | tr " " "|")
TO_BUILD=$(shell git diff --name-only origin/master...HEAD | grep -E "^($(BUILDABLE_REGEX))" | cut -d "/" -f 1,2 | sort | uniq | tr "\n" " ")

.DEFAULT_GOAL := build

.PHONY: $(BUILDABLE) $(TESTABLE)

test//components/si-data//RDEPS: test//components/si-account

test//components/si-account//RDEPS: test//components/si-graphql-api

test//components/si-settings//RDEPS: test//components/si-data test//components/si-account

%//RDEPS:
	@ echo "*** No dependencies for $@ ***"

$(BUILDABLE): 
	@ pushd $(patsubst build//%,%,$@); $(MAKE) 

build: $(BUILDABLE)

build_from_git: $(patsubst %,build//%,$(TO_BUILD))

$(TESTABLE): % : %//RDEPS
	@ pushd $(patsubst test//%,%,$@); $(MAKE) test

test: $(TESTABLE)

test_from_git: $(patsubst %,test//%,$(TO_BUILD))
