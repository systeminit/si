include ../build/typescript.mk
COMPONENT:=si-web-app
CONTAINER = si-web-app
.DEFAULT_GOAL := build

build: node_modules
	echo "floopy"
	$(MAKE) -C ../si-registry build_js
	npm run build
