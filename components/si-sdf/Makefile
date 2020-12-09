include ../build/rust.mk
COMPONENT = si-sdf
.DEFAULT_GOAL := build

build_release//cli:
	cargo build --bin si --release --verbose

container//cli:
	env BUILDKIT_PROGRESS=plain DOCKER_BUILDKIT=1 docker build \
		-f $(CURDIR)/../${COMPONENT}/Dockerfile.cli \
		-t systeminit/si-cli:latest \
		-t systeminit/si-cli:$(RELEASE) \
		-t si-cli:$(RELEASE) \
		$(CURDIR)/../../

release//cli: container//cli
	docker push systeminit/si-cli:latest
	docker push systeminit/si-cli:$(RELEASE)
