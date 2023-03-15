include ../../mk/base.mk
include ../../mk/rust.mk
include ../../mk/image.mk

.DEFAULT_GOAL := build
CONTAINER_NAME := sdf
IMG := systeminit/sdf
WATCH_TASK := run
WATCH_PATHS := . \
	../../lib/sdf-server \
	../../lib/telemetry-application-rs
