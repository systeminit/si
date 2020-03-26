include $(CURDIR)/../build/docker.mk

RELEASE := $(shell date +%Y%m%d%H%M%S)

.PHONY: build build_release test run install

install:
	npm install

build: install
	npm run build

build_release: build

test: install build
	npm run test

run: start

start: install
	npm run start

watch: start
	@ echo "Watching.."

test_container:
	docker run -t --network=host --volume=$(CURDIR)/../../:/src docker.pkg.github.com/systeminit/si/si-base:latest /bin/bash -c "cd / && if [[ ! -d /src/target ]]; then tar zxf /build-cache/cargo-cache.tgz; fi && if [[ ! -d /src/components/si-web-ui/node_modules ]]; then tar zxf /build-cache/npm-cache.tgz; fi && . /root/.cargo/env && cd /src/components/$(COMPONENT) && make test"

container:
	env BUILDKIT_PROGRESS=plain DOCKER_BUILDKIT=1 docker build \
		-f $(CURDIR)/../build/Dockerfile-typescript \
		-t ${COMPONENT}-service:latest \
		-t ${COMPONENT}-service:$(RELEASE) \
		-t 835304779882.dkr.ecr.us-east-2.amazonaws.com/si/${COMPONENT}-service:latest \
		-t 835304779882.dkr.ecr.us-east-2.amazonaws.com/si/${COMPONENT}-service:$(RELEASE) \
		--build-arg component=${COMPONENT} \
		$(CURDIR)/../../

release: container
	docker push 835304779882.dkr.ecr.us-east-2.amazonaws.com/si/${COMPONENT}-service:latest
	docker push 835304779882.dkr.ecr.us-east-2.amazonaws.com/si/${COMPONENT}-service:$(RELEASE)
