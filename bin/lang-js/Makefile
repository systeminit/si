include ../../mk/base.mk
include ../../mk/typescript.mk

.DEFAULT_GOAL := build

build: node_modules
	$(call header,$@)
	env NODE_ENV=production npm run package
