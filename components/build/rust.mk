include $(CURDIR)/../build/docker.mk

RELEASE := $(shell date +%Y%m%d%H%M%S)

.PHONY: build build_release test run start release container

build:
	cargo build

build_release:
	cargo build --release

test:
	cargo test

run:
	cargo run

start: run

test_container:
	docker run -it --network=host --volume=$(CURDIR)/../../:/src docker.pkg.github.com/systeminit/si/si-base:latest /bin/bash -c "cd / && tar zxf /build-cache/cargo-cache.tgz && tar zxf /build-cache/npm-cache.tgz && . /root/.cargo/env && cd /src/components/$(COMPONENT) && make test"

container:
	env BUILDKIT_PROGRESS=plain DOCKER_BUILDKIT=1 docker build \
		-f $(CURDIR)/../build/Dockerfile-rust \
		-t ${COMPONENT}-service:latest \
		-t ${COMPONENT}-service:$(RELEASE) \
		-t docker.pkg.github.com/systeminit/si/${COMPONENT}-service:latest \
		-t docker.pkg.github.com/systeminit/si/${COMPONENT}-service:$(RELEASE) \
		--build-arg component=${COMPONENT} \
		$(CURDIR)/../../

release: container
	docker push docker.pkg.github.com/systeminit/si/${COMPONENT}-service:$(RELEASE)
	docker push docker.pkg.github.com/systeminit/si/${COMPONENT}-service:latest

