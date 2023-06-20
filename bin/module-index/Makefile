include ../../mk/base.mk
include ../../mk/rust.mk

.DEFAULT_GOAL := build
CONTAINER_NAME := module-index
IMG := systeminit/module-index
WATCH_TASK := run
WATCH_PATHS := . \
	../../lib/module-index-server \
	../../lib/si-settings \
	../../lib/telemetry-application-rs
