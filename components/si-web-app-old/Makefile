include ../build/typescript.mk
COMPONENT:=si-web-app
CONTAINER = si-web-app
.DEFAULT_GOAL := build

registry:
	@echo "--- [$(shell basename ${CURDIR})] $@"
	$(MAKE) -C ../si-registry build_js build_types
.PHONY: registry

build: registry
	@echo "--- [$(shell basename ${CURDIR})] $@"
	$(MAKE) node_modules
	env NODE_ENV=production npm run build
