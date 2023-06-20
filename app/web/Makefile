include ../../mk/base.mk
include ../../mk/typescript.mk

.DEFAULT_GOAL := build
CONTAINER_NAME := web
IMG := systeminit/web

## clean-vite: Cleans Vite cache
clean-vite:
	$(call header,$@)
	npm run vite:clean
.PHONY: clean-vite
