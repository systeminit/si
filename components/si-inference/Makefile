include ../build/typescript.mk
COMPONENT = si-inference
.DEFAULT_GOAL := build

build:
	@echo "--- [$(shell basename ${CURDIR})] $@"
	$(MAKE) node_modules
	env NODE_ENV=production npm run build

watch:
	@echo "--- [$(shell basename ${CURDIR})] $@"
	npm run watch

format:
	@echo "--- [$(shell basename ${CURDIR})] $@"
	npm run lint:fix
