include ../build/typescript.mk
COMPONENT = si-registry
CONTAINER = si-veritech
.DEFAULT_GOAL := build

build_js: node_modules
	@echo "--- [$(shell basename ${CURDIR})] $@"
	npm run build:js

build_types: node_modules
	@echo "--- [$(shell basename ${CURDIR})] $@"
	npm run build:types

watch: build_js start
	@ echo "Watching.."

format:
	@echo "--- [$(shell basename ${CURDIR})] $@"
	npm run format
