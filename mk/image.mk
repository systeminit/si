include $(CURDIR)/../../mk/container.mk

image: ## Builds a container image (uses ./script/build-image.sh)
	@echo "--- [$(shell basename ${CURDIR})] $@"
	env BASE_VERSION=${BASE_VERSION} IMG=${IMG} ./script/build-image.sh
.PHONY: image

publish: ## Builds and pushes the image to the repository (uses ./script/build-image.sh)
	@echo "--- [$(shell basename ${CURDIR})] $@"
	env BASE_VERSION=${BASE_VERSION} IMG=${IMG} ./script/build-image.sh --push $(PUBLISH_ARGS)
.PHONY: publish

release: ## Builds and pushes the image and tags to the repository (uses ./script/build-image.sh)
	@echo "--- [$(shell basename ${CURDIR})] $@"
	env BASE_VERSION=${BASE_VERSION} IMG=${IMG} ./script/build-image.sh --ci
.PHONY: release

promote: ## Tags and pushes the current Git revision image as 'stable'  (uses ./script/promote-image.sh)
	@echo "--- [$(shell basename ${CURDIR})] $@"
	./script/promote-image.sh ${IMG} "$${SHA:-$$( git show -s --format=%H)}" stable
.PHONY: promote
