include ../build/typescript.mk
COMPONENT = si-entity
.DEFAULT_GOAL := build

registry:
	@echo "--- [$(shell basename ${CURDIR})] $@"
	$(MAKE) -C ../si-registry build
.PHONY: registry

build: registry
	@echo "--- [$(shell basename ${CURDIR})] $@"
	$(MAKE) node_modules
	env NODE_ENV=production npm run build

watch:
	@echo "--- [$(shell basename ${CURDIR})] $@"
	npm run watch

format:
	@echo "--- [$(shell basename ${CURDIR})] $@"
	npm run lint:fix
