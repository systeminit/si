include $(CURDIR)/../../mk/container.mk

## image: Builds a container image (uses ./script/build-image.sh)
default--image:
	@echo "--- [$(shell git rev-parse --show-prefix | sed s,/$$,,)] $@"
	env BASE_VERSION=${BASE_VERSION} IMG=${IMG} ./script/build-image.sh
.PHONY: default--image

## publish: Builds and pushes the image to the repository (uses ./script/build-image.sh)
default--publish:
	@echo "--- [$(shell git rev-parse --show-prefix | sed s,/$$,,)] $@"
	env BASE_VERSION=${BASE_VERSION} IMG=${IMG} ./script/build-image.sh --push $(PUBLISH_ARGS)
.PHONY: default--publish

## release: Builds and pushes the image and tags to the repository (uses ./script/build-image.sh)
default--release:
	@echo "--- [$(shell git rev-parse --show-prefix | sed s,/$$,,)] $@"
	env BASE_VERSION=${BASE_VERSION} IMG=${IMG} ./script/build-image.sh --ci
.PHONY: default--release

## promote: Tags and pushes the current Git revision image as 'stable'  (uses ./script/promote-image.sh)
default--promote:
	@echo "--- [$(shell git rev-parse --show-prefix | sed s,/$$,,)] $@"
	./script/promote-image.sh ${IMG} "$${SHA:-$$( git show -s --format=%H)}" stable
.PHONY: default--promote
