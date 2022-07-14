MAKEPATH := $(shell dirname $(realpath $(lastword $(MAKEFILE_LIST))))
REPOPATH := $(shell dirname $(MAKEPATH))
CI_FROM_REF := main
CI_TO_REF := HEAD

.DEFAULT_GOAL := prod

boostrap:
	$(MAKEPATH)/scripts/bootstrap.sh

# Run this target alongside local developer builds of web, sdf, etc.
# This target does not compose down beforehand.
partial: init
	GATEWAY=$(shell $(MAKEPATH)/scripts/gateway.sh) \
		docker-compose \
			-f $(MAKEPATH)/docker-compose.yml \
			-f $(MAKEPATH)/docker-compose.env.yml \
		up -d nats pg otel faktory

# Run interactively and compose down beforehand. This target is meant to be run iteratively for
# local development and testing.
dev: down init
	GATEWAY=$(shell $(MAKEPATH)/scripts/gateway.sh) \
		docker-compose \
			-f $(MAKEPATH)/docker-compose.yml \
			-f $(MAKEPATH)/docker-compose.env.yml \
		up

# Run in a detached state and do not compose down beforehand.
# This is the target to run in production.
prod: init
	GATEWAY=$(shell $(MAKEPATH)/scripts/gateway.sh) \
		docker-compose \
			-f $(MAKEPATH)/docker-compose.yml \
			-f $(MAKEPATH)/docker-compose.env.yml \
		up -d

web: init
	# REPOPATH=$(REPOPATH) $(MAKEPATH)/scripts/check-for-artifacts-before-mounting.sh
	$(MAKEPATH)/scripts/generate-ci-yml.sh $(CI_FROM_REF) $(CI_TO_REF)
	GATEWAY=$(shell $(MAKEPATH)/scripts/gateway.sh) REPOPATH=$(REPOPATH) \
		docker-compose \
			-f $(MAKEPATH)/docker-compose.yml \
			-f $(MAKEPATH)/docker-compose.env.yml \
			-f $(MAKEPATH)/docker-compose.ci.yml \
		up -d

ci: init
	REPOPATH=$(REPOPATH) $(MAKEPATH)/scripts/check-for-artifacts-before-mounting.sh
	GATEWAY=$(shell $(MAKEPATH)/scripts/gateway.sh) REPOPATH=$(REPOPATH) \
		docker-compose \
			-f $(MAKEPATH)/docker-compose.yml \
			-f $(MAKEPATH)/docker-compose.env.yml \
			-f $(MAKEPATH)/docker-compose.ci.yml \
		up -d

down:
	-docker-compose -f $(MAKEPATH)/docker-compose.yml \
		down

init:
	if [ ! -f $(MAKEPATH)/docker-compose.env.yml ]; then \
		echo "version: \"3\"" > $(MAKEPATH)/docker-compose.env.yml; \
	fi
	docker-compose -f $(MAKEPATH)/docker-compose.yml pull
