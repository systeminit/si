.PHONY: sync deps package_sync

PATH := $(PATH):$(HOME)/.cargo/bin
SHELL := env PATH=$(PATH) /bin/bash
USE_PACMAN=$(wildcard /usr/bin/pacman)
USE_APT=$(wildcard /usr/bin/apt-get)
RUST_EXISTS=$(wildcard $(HOME)/.cargo/bin/cargo)
NODE_VERSION=v13.11.0
NODE_DISTRO=linux-x64

package_update:
ifeq ($(USE_PACMAN),/usr/bin/pacman)
	sudo pacman -Sy --noconfirm 
endif
ifeq ($(USE_APT),/usr/bin/apt-get)
	sudo env DEBIAN_FRONTEND=noninteractive apt-get update -y
endif

package_sync: package_update
ifeq ($(USE_PACMAN),/usr/bin/pacman)
	sudo pacman -Syu --noconfirm 
endif
ifeq ($(USE_APT),/usr/bin/apt-get)
	sudo env DEBIAN_FRONTEND=noninteractive apt-get upgrade -y
endif

package_curl:
ifeq ($(USE_PACMAN),/usr/bin/pacman)
	sudo pacman -S --needed --noconfirm curl
endif
ifeq ($(USE_APT),/usr/bin/apt-get)
	sudo env DEBIAN_FRONTEND=noninteractive apt-get install -y curl
endif

package_rust: package_curl package_compilers
ifneq ($(RUST_EXISTS),$(HOME)/.cargo/bin/cargo)
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > /tmp/rustup.sh
	chmod a+x /tmp/rustup.sh
	/tmp/rustup.sh -y
	cargo install --force cargo-watch
endif

package_compilers:
ifeq ($(USE_PACMAN),/usr/bin/pacman)
	sudo pacman -S --needed --noconfirm base-devel cmake clang
endif
ifeq ($(USE_APT),/usr/bin/apt-get)
	sudo env DEBIAN_FRONTEND=noninteractive apt-get install -y build-essential cmake clang 
endif

build_deps: package_update package_curl package_compilers package_rust runtime_deps
ifeq ($(USE_PACMAN),/usr/bin/pacman)
	sudo pacman -S --needed --noconfirm parallel tmux
endif
ifeq ($(USE_APT),/usr/bin/apt-get)
	sudo env DEBIAN_FRONTEND=noninteractive apt-get install -y parallel tmux
endif

runtime_deps: package_update
ifeq ($(USE_PACMAN),/usr/bin/pacman)
	sudo pacman -S --needed --noconfirm openssl libev libevent nodejs npm
endif
ifeq ($(USE_APT),/usr/bin/apt-get)
	sudo env DEBIAN_FRONTEND=noninteractive apt-get install -y openssl libssl-dev libev-dev libevent-dev
			curl -sSf https://nodejs.org/dist/$(NODE_VERSION)/node-$(NODE_VERSION)-$(NODE_DISTRO).tar.xz -o /tmp/node-$(NODE_VERSION)-$(NODE_DISTRO).tar.xz && \
      sudo mkdir -p /usr/local/lib/nodejs && \
			sudo tar -xJvf /tmp/node-$(NODE_VERSION)-$(NODE_DISTRO).tar.xz -C /usr/local/lib/nodejs && \
      sudo ln -sf /usr/local/lib/nodejs/node-$(NODE_VERSION)-$(NODE_DISTRO)/bin/node /usr/bin/node && \
      sudo ln -sf /usr/local/lib/nodejs/node-$(NODE_VERSION)-$(NODE_DISTRO)/bin/npm /usr/bin/npm && \
      sudo ln -sf /usr/local/lib/nodejs/node-$(NODE_VERSION)-$(NODE_DISTRO)/bin/npx /usr/bin/npx
endif

