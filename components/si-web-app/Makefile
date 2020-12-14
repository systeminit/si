include ../build/typescript.mk
COMPONENT:=si-web-app
CONTAINER = si-web-app
.DEFAULT_GOAL := build

build: node_modules
	$(MAKE) -C ../si-registry build_js
	env NODE_ENV=production npm run build
