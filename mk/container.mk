## run-container: Runs the component in a container (uses ./script/run-container.sh)
default--run-container:
	$(call header,$@)
	./script/run-container.sh
.PHONY: default--run-container

## stop-container: Stops a running container for the component
default--stop-container:
	$(call header,$@)
	docker container stop $(CONTAINER_NAME)
.PHONY: default--stop-container

## tail-container: Tails the logs of the container for the component
default--tail-container:
	$(call header,$@)
	docker container logs -f $(CONTAINER_NAME)
.PHONY: default--tail-container

## clean-container: Stops and removes the container for the component
default--clean-container:
	$(call header,$@)
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
.PHONY: default--clean-container

# Thanks to:
# https://newbedev.com/make-file-warning-overriding-commands-for-target
%: default--%
	@true
