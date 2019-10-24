include $(CURDIR)/../build/docker.mk

.PHONY: build build_release test run

build:
	cargo build

build_release:
	cargo build --release

test:
	cargo test

run:
	cargo run

start: run
