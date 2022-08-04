define header
	@echo "--- [$(patsubst %/,%,$(shell git rev-parse --show-prefix))] $(patsubst default--%,%,$1)"
endef

## help: Prints help information
help:
	@printf -- "\033[1;36;40mmake %s\033[0m\n" "$@"
	@echo
	@echo "USAGE:"
	@echo "    make [TARGET]"
	@echo
	@echo "TARGETS:"
	@grep -hE '^##\s*[^:]+:.*$$' $(MAKEFILE_LIST) | sort | awk '\
		match($$0, /^##\s*([^:]+):\s*(.*)$$/, m) \
		{ printf "    \033[1;36;40m%-20s\033[0m %s\n", m[1], m[2] }'
	@echo
	@echo "SEE ALSO:"
	@printf -- "    For more information, check out \033[1;36;40mDEVELOPING.md\033[0m.\n"
.PHONY: help
