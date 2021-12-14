run-container: ## Runs the component in a container (uses ./script/run-container.sh)
	@echo "--- [$(shell basename ${CURDIR})] $@"
	./script/run-container.sh
.PHONY: run-container

stop-container: ## Stops a running container for the component
	@echo "--- $@"
	docker container stop $(CONTAINER_NAME)
.PHONY: stop

tail-container: ## Tails the logs of the container for the component
	@echo "--- $@"
	docker container logs -f $(CONTAINER_NAME)
.PHONY: tail

clean-container: ### Stops and removes the container for the component
	@if [ -n "$$(docker container ls --filter "name=^$(CONTAINER_NAME)" \
		--filter "status=running" --quiet)" ]; \
	then \
		$(MAKE) stop-container; \
	fi
	@if [ -n "$$(docker container ls --filter "name=^$(CONTAINER_NAME)" \
		--all --quiet)" ]; \
	then \
		echo "  - Removing container $(CONTAINER_NAME)"; \
		docker container rm $(CONTAINER_NAME); \
	fi
.PHONY: clean-container
