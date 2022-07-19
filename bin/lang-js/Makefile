include ../../mk/typescript.mk

.DEFAULT_GOAL := build

build: node_modules
	@echo "--- [$(shell basename ${CURDIR})] $@"
	env NODE_ENV=production npm run package
.PHONY: build

lint:
	@echo 'skipping lint'
PHONY: lint
