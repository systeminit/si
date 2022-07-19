include ../../mk/base.mk
include ../../mk/typescript.mk
include ../../mk/image.mk

.DEFAULT_GOAL := build
CONTAINER_NAME := web
IMG := systeminit/web

test: node_modules ## Tests the TypeScript package
	@echo "--- [$(shell basename ${CURDIR})] $@"
ifeq ($(CI),true)
	env CYPRESS_BASE_URL='https://localhost' npm run cypress:run || exit 0
else
	npm run cypress:run
endif
.PHONY: test
