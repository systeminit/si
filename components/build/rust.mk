include $(CURDIR)/../build/docker.mk

RELEASE := $(shell date +%Y%m%d%H%M%S)
WATCH_PATHS ?= .
WATCH_TASK ?= run
RUST_LOG ?= info

.PHONY: build build_release test run start release container

build:
	cargo build

build_release:
	cargo build --release

test:
	env RUST_BACKTRACE=1 RUST_LOG=debug cargo test -- --nocapture

run:
	cargo run

start: run

watch:
	env RUST_LOG=$(RUST_LOG) cargo watch $(foreach path,$(WATCH_PATHS),-w $(path)) -x $(WATCH_TASK)

test_container:
	docker run -t --network=host --volume=$(CURDIR)/../../:/src docker.pkg.github.com/systeminit/si/si-base:latest /bin/bash -c "cd / && if [[ ! -d /src/target ]]; then tar zxf /build-cache/cargo-cache.tgz; fi && if [[ ! -d /src/components/si-web-ui/node_modules ]]; then tar zxf /build-cache/npm-cache.tgz; fi && . /root/.cargo/env && cd /src/components/$(COMPONENT) && make test"

# -t docker.pkg.github.com/systeminit/si/${COMPONENT}-service:latest \
# -t docker.pkg.github.com/systeminit/si/${COMPONENT}-service:$(RELEASE) \

container:
	env BUILDKIT_PROGRESS=plain DOCKER_BUILDKIT=1 docker build \
		-f $(CURDIR)/../build/Dockerfile-rust \
		-t ${COMPONENT}-service:latest \
		-t ${COMPONENT}-service:$(RELEASE) \
		-t 835304779882.dkr.ecr.us-east-2.amazonaws.com/si/${COMPONENT}-service:latest \
		-t 835304779882.dkr.ecr.us-east-2.amazonaws.com/si/${COMPONENT}-service:$(RELEASE) \
		--build-arg component=${COMPONENT} \
		$(CURDIR)/../../

release: container
	docker push 835304779882.dkr.ecr.us-east-2.amazonaws.com/si/${COMPONENT}-service:latest
	docker push 835304779882.dkr.ecr.us-east-2.amazonaws.com/si/${COMPONENT}-service:$(RELEASE)

