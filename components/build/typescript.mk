include $(CURDIR)/../build/docker.mk

RELEASE := $(shell date +%Y%m%d%H%M%S)

.PHONY: build build_release clean test run node_modules

node_modules: package.json
	@echo "--- [$(shell basename ${CURDIR})] $@"
	npm install

build: node_modules
	@echo "--- [$(shell basename ${CURDIR})] $@"
	env NODE_ENV=production npm run build

build_release: build

test: node_modules
	@echo "--- [$(shell basename ${CURDIR})] $@"
	npm run test

run: start

start: node_modules
	@echo "--- [$(shell basename ${CURDIR})] $@"
	npm run start

watch: node_modules
	@echo "--- [$(shell basename ${CURDIR})] $@"
	npm run watch

clean:
	@echo "--- [$(shell basename ${CURDIR})] $@"
	rm -rf ./node_modules ./dist ./lib

test_container:
	@echo "--- [$(shell basename ${CURDIR})] $@"
	docker run -t --network=host --volume=$(CURDIR)/../../:/src docker.pkg.github.com/systeminit/si/si-base:latest /bin/bash -c "cd / && if [[ ! -d /src/target ]]; then tar zxf /build-cache/cargo-cache.tgz; fi && if [[ ! -d /src/components/si-web-ui/node_modules ]]; then tar zxf /build-cache/npm-cache.tgz; fi && . /root/.cargo/env && cd /src/components/$(COMPONENT) && make test"
