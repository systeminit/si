run-container:
	@echo "--- [$(shell basename ${CURDIR})] $@"
	./bin/run-container.sh
.PHONY: run-container

stop-container:
	@echo "--- $@"
	docker container stop $(CONTAINER_NAME)
.PHONY: stop

tail-container:
	@echo "--- $@"
	docker container logs -f $(CONTAINER_NAME)
.PHONY: tail

clean-container:
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
