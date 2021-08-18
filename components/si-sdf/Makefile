include ../build/rust.mk
include ../build/image.mk

COMPONENT = si-sdf
CONTAINER_NAME := sdf
.DEFAULT_GOAL := build
WATCH_PATHS := . ../si-model ../si-settings ../si-data

build_release//cli:
	@echo "--- [$(shell basename ${CURDIR})] $@"
	cargo build --bin si --release --verbose

release//cli: container//cli
	@echo "--- [$(shell basename ${CURDIR})] $@"
	docker push systeminit/si-cli:latest
	docker push systeminit/si-cli:$(RELEASE)
