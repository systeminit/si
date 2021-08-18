include ../build/typescript.mk
include ../build/image.mk

COMPONENT:=si-web-app
CONTAINER_NAME := web
.DEFAULT_GOAL := build

registry:
	@echo "--- [$(shell basename ${CURDIR})] $@"
	$(MAKE) -C ../si-registry build
.PHONY: registry

build: registry
	@echo "--- [$(shell basename ${CURDIR})] $@"
	$(MAKE) node_modules
	env NODE_ENV=production npm run build

lint: node_modules
	@echo "--- [$(shell basename ${CURDIR})] $@"
	npm run lint

lint_no_fix: node_modules
	@echo "--- [$(shell basename ${CURDIR})] $@"
	npm run lint-no-fix
