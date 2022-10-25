MAKEPATH := $(shell dirname $(realpath $(lastword $(MAKEFILE_LIST))))
REPOPATH := $(shell dirname $(MAKEPATH))
CI_FROM_REF := main
CI_TO_REF := HEAD

.DEFAULT_GOAL := prod

boostrap:
	$(MAKEPATH)/scripts/bootstrap.sh

# Run this target alongside local developer builds of web, sdf, etc.
# This target does not compose down beforehand.
partial: init-partial
	GATEWAY=$(shell $(MAKEPATH)/scripts/gateway.sh) \
		docker compose \
			-f $(MAKEPATH)/docker-compose.yml \
			-f $(MAKEPATH)/docker-compose.env.yml \
			up --detach

partial-pganalyze: init-partial
	GATEWAY=$(shell $(MAKEPATH)/scripts/gateway.sh) \
		docker compose \
			-f $(MAKEPATH)/docker-compose.yml \
			-f $(MAKEPATH)/docker-compose.env.yml \
			-f $(MAKEPATH)/docker-compose.pganalyze.yml \
			up --detach

partial-local-pg: init-partial
	SI_ROOT="${MAKEPATH}/.." bash ${MAKEPATH}/scripts/push-pg-local.sh 
	GATEWAY=$(shell $(MAKEPATH)/scripts/gateway.sh) \
		docker compose \
			-f $(MAKEPATH)/docker-compose.yml \
			-f $(MAKEPATH)/docker-compose.env.yml \
			-f $(MAKEPATH)/docker-compose.local-postgres.yml \
			up --detach

# Run interactively and compose down beforehand. This target is meant to be run iteratively for
# local development and testing.
dev: down init
	GATEWAY=$(shell $(MAKEPATH)/scripts/gateway.sh) \
		docker compose \
			-f $(MAKEPATH)/docker-compose.yml \
			-f $(MAKEPATH)/docker-compose.env.yml \
			-f $(MAKEPATH)/docker-compose.ci.yml \
			--profile si \
			up

# Run in a detached state and do not compose down beforehand.
# This is the target to run in production.
prod:
	GATEWAY=$(shell $(MAKEPATH)/scripts/gateway.sh) \
		docker compose \
			-f $(MAKEPATH)/docker-compose.yml \
			-f $(MAKEPATH)/docker-compose.env.yml \
			-f $(MAKEPATH)/docker-compose.pganalyze.yml \
			-f $(MAKEPATH)/docker-compose.prod.yml \
			--profile si \
			up --detach

web: init
	# REPOPATH=$(REPOPATH) $(MAKEPATH)/scripts/check-for-artifacts-before-mounting.sh
	$(MAKEPATH)/scripts/generate-ci-yml.sh $(CI_FROM_REF) $(CI_TO_REF)
	GATEWAY=$(shell $(MAKEPATH)/scripts/gateway.sh) REPOPATH=$(REPOPATH) \
		docker compose \
			-f $(MAKEPATH)/docker-compose.yml \
			-f $(MAKEPATH)/docker-compose.env.yml \
			-f $(MAKEPATH)/docker-compose.ci.yml \
			up --detach

ci: init
	REPOPATH=$(REPOPATH) $(MAKEPATH)/scripts/check-for-artifacts-before-mounting.sh
	GATEWAY=$(shell $(MAKEPATH)/scripts/gateway.sh) REPOPATH=$(REPOPATH) \
		docker compose \
			-f $(MAKEPATH)/docker-compose.yml \
			-f $(MAKEPATH)/docker-compose.env.yml \
			-f $(MAKEPATH)/docker-compose.ci.yml \
			up --detach

down:
	-docker compose \
		-f $(MAKEPATH)/docker-compose.yml \
		--profile si \
		down

init-partial:
	if [ ! -f $(MAKEPATH)/docker-compose.env.yml ]; then \
		echo "version: \"3\"" > $(MAKEPATH)/docker-compose.env.yml; \
	fi
	if [ -n "$(SI_SKIP_PULL)" ]; then \
		echo "--- SI_SKIP_PULL set, skipping docker compose pull"; \
	else \
		docker compose \
			-f $(MAKEPATH)/docker-compose.yml \
			pull; \
	fi

init:
	if [ ! -f $(MAKEPATH)/docker-compose.env.yml ]; then \
		echo "version: \"3\"" > $(MAKEPATH)/docker-compose.env.yml; \
	fi
	if [ -n "$(SI_SKIP_PULL)" ]; then \
		echo "--- SI_SKIP_PULL set, skipping docker compose pull"; \
	else \
		docker compose \
			-f $(MAKEPATH)/docker-compose.yml \
			--profile si \
			pull; \
	fi

