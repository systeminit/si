include ../build/typescript.mk
COMPONENT:=si-web-app
.DEFAULT_GOAL := build

build: node_modules
	$(MAKE) -C ../si-registry build
	npm run build
