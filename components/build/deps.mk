.PHONY: sync deps package_sync

PATH := $(PATH):$(HOME)/.cargo/bin
SHELL := env PATH=$(PATH) /bin/bash
USE_PACMAN=$(wildcard /usr/bin/pacman)
USE_APT=$(wildcard /usr/bin/apt-get)
RUST_EXISTS=$(wildcard $(HOME)/.cargo/bin/cargo)

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

package_rust: package_curl
ifneq ($(RUST_EXISTS),$(HOME)/.cargo/bin/cargo)
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > /tmp/rustup.sh
	chmod a+x /tmp/rustup.sh
	/tmp/rustup.sh -y
endif

build_deps: package_update package_curl package_rust runtime_deps
ifeq ($(USE_PACMAN),/usr/bin/pacman)
	sudo pacman -S --needed --noconfirm base-devel cmake clang parallel tmux
endif
ifeq ($(USE_APT),/usr/bin/apt-get)
	sudo env DEBIAN_FRONTEND=noninteractive apt-get install -y build-essential cmake clang parallel tmux
endif

runtime_deps: package_update
ifeq ($(USE_PACMAN),/usr/bin/pacman)
	sudo pacman -S --needed --noconfirm openssl libev libevent nodejs npm
endif
ifeq ($(USE_APT),/usr/bin/apt-get)
	sudo env DEBIAN_FRONTEND=noninteractive apt-get install -y openssl libev-dev libevent-dev nodejs npm
endif

