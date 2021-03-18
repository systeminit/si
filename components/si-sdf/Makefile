include ../build/rust.mk
COMPONENT = si-sdf
.DEFAULT_GOAL := build
WATCH_PATHS := . ../si-model ../si-settings ../si-data

build_release//cli:
	@echo "--- [$(shell basename ${CURDIR})] $@"
	cargo build --bin si --release --verbose

container//cli:
	@echo "--- [$(shell basename ${CURDIR})] $@"
	env BUILDKIT_PROGRESS=plain DOCKER_BUILDKIT=1 docker build \
		-f $(CURDIR)/../${COMPONENT}/Dockerfile.cli \
		-t systeminit/si-cli:latest \
		-t systeminit/si-cli:$(RELEASE) \
		-t si-cli:$(RELEASE) \
		$(CURDIR)/../../

release//cli: container//cli
	@echo "--- [$(shell basename ${CURDIR})] $@"
	docker push systeminit/si-cli:latest
	docker push systeminit/si-cli:$(RELEASE)
