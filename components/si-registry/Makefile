include ../build/typescript.mk
COMPONENT = si-registry
.DEFAULT_GOAL := build

build: node_modules
	@echo "--- [$(shell basename ${CURDIR})] $@"
	npm run build

watch: build start
	@ echo "Watching.."

format:
	@echo "--- [$(shell basename ${CURDIR})] $@"
	npm run lint:fix
