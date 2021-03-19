include ../build/typescript.mk
COMPONENT = si-veritech
CONTAINER = si-veritech
.DEFAULT_GOAL := build
.PHONY: registry

registry: 
	@echo "--- [$(shell basename ${CURDIR})] $@"
	$(MAKE) -C ../si-registry build

format:
	@echo "--- [$(shell basename ${CURDIR})] $@"
	npm run lint:fix
