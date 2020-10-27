include $(CURDIR)/../build/docker.mk

RELEASE := $(shell date +%Y%m%d%H%M%S)

.PHONY: build build_release clean test run install

install: 
	npm install

node_modules: package.json
	npm install

build: node_modules
	env NODE_ENV=production npm run build

build_release: build

test: node_modules
	npm run test

run: start

start: node_modules
	npm run start

watch: start
	@ echo "Watching.."

clean:
	rm -rf ./node_modules ./dist ./lib

test_container:
	docker run -t --network=host --volume=$(CURDIR)/../../:/src docker.pkg.github.com/systeminit/si/si-base:latest /bin/bash -c "cd / && if [[ ! -d /src/target ]]; then tar zxf /build-cache/cargo-cache.tgz; fi && if [[ ! -d /src/components/si-web-ui/node_modules ]]; then tar zxf /build-cache/npm-cache.tgz; fi && . /root/.cargo/env && cd /src/components/$(COMPONENT) && make test"

container:
	env BUILDKIT_PROGRESS=plain DOCKER_BUILDKIT=1 docker build \
		-f $(CURDIR)/../${COMPONENT}/Dockerfile \
		-t systeminit/${CONTAINER}:latest \
		-t systeminit/${CONTAINER}:$(RELEASE) \
		$(CURDIR)/../../

release: container
	docker push systeminit/${CONTAINER}:latest
	docker push systeminit/${CONTAINER}:$(RELEASE)

