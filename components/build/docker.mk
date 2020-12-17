.PHONY: container_arch

container_arch:
	@echo "--- [$(shell basename ${CURDIR})] $@"
	docker run -it --rm -v `pwd`:/src archlinux:latest bash
