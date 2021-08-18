include ../build/typescript.mk
include ../build/image.mk
COMPONENT = si-veritech
CONTAINER_NAME := veritech
.DEFAULT_GOAL := build
.PHONY: registry registry format run-image

registry:
	@echo "--- [$(shell basename ${CURDIR})] $@"
	$(MAKE) -C ../si-registry build

format:
	@echo "--- [$(shell basename ${CURDIR})] $@"
	npm run lint:fix
