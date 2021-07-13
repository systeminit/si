VERSION := 0.29.0
CONFIG := otel-config.yaml
IMG := systeminit/otelcol
NAME := otelcol

run: build-if-missing ## Runs the OpenTelemtry Collector container
	@echo "--- $@"
	@if [ -n "$$(docker container ls --filter "name=^$(NAME)" \
		--filter "status=running" --quiet)" ]; \
	then \
		echo "  - Container $(NAME) is already running"; \
	elif [ -n "$$(docker container ls --filter "name=^$(NAME)" \
		--all --quiet)" ]; \
	then \
		echo "  - Starting stopped container $(NAME)"; \
		docker container start $(NAME); \
	else \
		echo "  - Creating and starting container $(NAME)"; \
		if [ -z "$${HONEYCOMB_TOKEN:-}" ]; then \
			echo "xxx HONEYCOMB_TOKEN must be set" >&2; \
			exit 1; \
		fi; \
		if [ -z "$${HONEYCOMB_DATASET:-}" ]; then \
			echo "xxx HONEYCOMB_DATASET must be set" >&2; \
			exit 1; \
		fi; \
		docker run --detach \
			--env "HONEYCOMB_TOKEN=$$HONEYCOMB_TOKEN" \
			--env "HONEYCOMB_DATASET=$$HONEYCOMB_DATASET" \
			--publish 4317:4317 \
			--publish 55679:55679 \
			--name $(NAME) \
			$(IMG); \
	fi
.PHONY: run

build: ## Builds the sources
	@echo "--- $@"
	docker build \
		--build-arg VERSION=$(VERSION) \
		--build-arg CONFIG=$(CONFIG) \
		-t $(IMG) \
		.
.PHONY: build

build-if-missing:
	@if [ -z "$$(docker image ls --quiet $(IMG))" ]; then $(MAKE) build; fi
.PHONY: build-if-missing

stop: ## Stops the OpenTelemetry Collector container
	@echo "--- $@"
	docker container stop $(NAME)
.PHONY: stop

tail: ## Tails the OpenTelemetry Collector container logs
	@echo "--- $@"
	docker container logs -f $(NAME)
.PHONY: tail

clean-container: ## Cleans up the OpenTelemetry Collector container
	@if [ -n "$$(docker container ls --filter "name=^$(NAME)" \
		--filter "status=running" --quiet)" ]; \
	then \
		$(MAKE) stop; \
	fi
	@if [ -n "$$(docker container ls --filter "name=^$(NAME)" \
		--all --quiet)" ]; \
	then \
		echo "  - Removing container $(NAME)"; \
		docker container rm $(NAME); \
	fi
.PHONY: clean-container

clean-image: clean-container ## Cleans up the OpenTelemetry Collector image
	@if [ -n "$$(docker image ls --quiet $(IMG))" ]; then \
		echo "  - Removing image $(IMG)"; \
		docker image rm $(IMG); \
	fi
.PHONY: clean-image

clean: clean-container clean-image ## Cleans up containers and images
.PHONY: clean
