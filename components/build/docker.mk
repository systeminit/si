.PHONY: container_arch

container_arch:
	docker run -it --rm -v `pwd`:/src archlinux:latest bash
