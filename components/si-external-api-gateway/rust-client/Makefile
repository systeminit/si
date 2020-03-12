WATCH_PATHS +=  . ../proto

.PHONY: watch

watch:
	cargo watch $(foreach path,$(WATCH_PATHS),-w $(path)) -x build
