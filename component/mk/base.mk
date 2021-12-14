help: ## Prints help information
	@printf -- "\033[1;36;40mmake %s\033[0m\n" "$@"
	@echo
	@echo "USAGE:"
	@echo "    make [TARGET]"
	@echo
	@echo "TARGETS:"
	@grep -hE '^[a-zA-Z0-9_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk '\
		BEGIN { FS = ":.*?## " }; \
		{ printf "    \033[1;36;40m%-20s\033[0m %s\n", $$1, $$2 }'
.PHONY: help
