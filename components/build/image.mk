include $(CURDIR)/../build/container.mk

image:
	@echo "--- [$(shell basename ${CURDIR})] $@"
	./bin/build-image.sh

publish:
	@echo "--- [$(shell basename ${CURDIR})] $@"
	./bin/build-image.sh --push

release:
	@echo "--- [$(shell basename ${CURDIR})] $@"
	./bin/build-image.sh --ci
