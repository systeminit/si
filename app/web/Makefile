include ../build/typescript.mk
include ../build/image.mk

COMPONENT:=si-webapp
CONTAINER_NAME := web
.DEFAULT_GOAL := build

build: node_modules
	@echo "--- [$(shell basename ${CURDIR})] $@"
	env NODE_ENV=production npm run build

lint: node_modules
	@echo "--- [$(shell basename ${CURDIR})] $@"
	npm run eslint

run: node_modules
	@echo "--- [$(shell basename ${CURDIR})] $@"
	npm run lint