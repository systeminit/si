include ../../mk/base.mk
include ../../mk/rust.mk

.DEFAULT_GOAL := build
CONTAINER_NAME := council
IMG := systeminit/council
WATCH_TASK := run
WATCH_PATHS := . \
	../../lib/dal \
	../../lib/si-data \
	../../lib/si-data-nats \
	../../lib/si-data-pg \
	../../lib/si-settings \
	../../lib/si-std \
	../../lib/telemetry-application-rs \
	../../lib/veritech-client
