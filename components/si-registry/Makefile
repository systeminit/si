include ../build/typescript.mk
COMPONENT = si-registry
CONTAINER = si-veritech
.DEFAULT_GOAL := build

build_js: node_modules
	npm run build:js

watch: build_js start
	@ echo "Watching.."

format: 
	npm run format