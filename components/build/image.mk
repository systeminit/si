include $(CURDIR)/../build/container.mk

image:
	@echo "--- [$(shell basename ${CURDIR})] $@"
	./script/build-image.sh

publish:
	@echo "--- [$(shell basename ${CURDIR})] $@"
	./script/build-image.sh --push $(PUBLISH_ARGS)

release:
	@echo "--- [$(shell basename ${CURDIR})] $@"
	./script/build-image.sh --ci
