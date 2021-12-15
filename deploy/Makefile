MAKEPATH := $(shell dirname $(realpath $(lastword $(MAKEFILE_LIST))))
DOCKER_CONFIG := $(HOME)/.docker/config.json
GATEWAY_DEFAULT := "I like my butt"

.DEFAULT_GOAL := prod

boostrap:
	$(MAKEPATH)/scripts/bootstrap.sh

# Run this target alongside local developer builds of web, sdf, etc.
partial: down smoke-test check-env
	DOCKER_CONFIG=$(DOCKER_CONFIG) GATEWAY=$(shell $(MAKEPATH)/scripts/gateway.sh) \
		docker-compose -f $(MAKEPATH)/docker-compose.yml -f $(MAKEPATH)/docker-compose.env.yml \
		up nats pg otel

# Run interactively and compose down beforehand. This target is meant to be run iteratively for
# local development and testing.
dev: down smoke-test check-env
	DOCKER_CONFIG=$(DOCKER_CONFIG) GATEWAY=$(shell $(MAKEPATH)/scripts/gateway.sh) \
		docker-compose -f $(MAKEPATH)/docker-compose.yml -f $(MAKEPATH)/docker-compose.env.yml \
		up

# Run in a detached state and do not compose down beforehand.
# This is the target to run in production.
prod: smoke-test check-env
	DOCKER_CONFIG=$(DOCKER_CONFIG) GATEWAY=$(shell $(MAKEPATH)/scripts/gateway.sh) \
		docker-compose -f $(MAKEPATH)/docker-compose.yml -f $(MAKEPATH)/docker-compose.env.yml \
		up -d

down:
	-DOCKER_CONFIG=$(DOCKER_CONFIG) GATEWAY=$(GATEWAY_DEFAULT) \
		docker-compose -f $(MAKEPATH)/docker-compose.yml \
		down

# Smoke test the private repository pull first.
smoke-test:
	DOCKER_CONFIG=$(DOCKER_CONFIG) GATEWAY=$(GATEWAY_DEFAULT) \
		docker-compose -f $(MAKEPATH)/docker-compose.yml \
		pull

check-env:
	@if [ ! -f $(MAKEPATH)/docker-compose.env.yml ]; then \
		echo "Missing docker-compose.env.yml - must be included for use!"; \
		echo "A sample file appears below, for your convenience. ;)"; \
		echo "---"; \
		echo "sdf:"; \
		echo "  volumes:"; \
		echo "    - /etc/jwt_secret_key.bin:/run/sdf/jwt_secret_key.bin"; \
		echo "otel:"; \
		echo "  environment:"; \
		echo "    - HONEYCOMB_TOKEN=the_token"; \
		echo "    - HONEYCOMB_DATASET=the_dataset"; \
		echo "---"; \
		exit 1; \
	fi
