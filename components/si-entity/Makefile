include ../build/typescript.mk
COMPONENT = si-entity
.DEFAULT_GOAL := build

watch:
	@echo "--- [$(shell basename ${CURDIR})] $@"
	npm run watch

format:
	@echo "--- [$(shell basename ${CURDIR})] $@"
	npm run lint:fix
