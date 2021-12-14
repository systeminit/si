#!/usr/bin/env bash
set -e

function error-and-exit {
    if [ "$1" = "" ]; then
        error-and-exit "must provide argument <error-message>"
    fi
    echo "error: $1"
    exit 1
}

function verify-fedora {
    if [ "$(uname -s)" != "Linux" ]; then
        error-and-exit "OS is not Linux-based"
    fi
    if [ ! -f /etc/os-release ]; then
        error-and-exit "cannot determine Linux distro"
    fi
    if [ "$(grep '^ID=' /etc/os-release | sed 's/^ID=//' | tr -d '"')" != "fedora" ]; then
        error-and-exit "Linux distro must be fedora"
    fi
    if [ "$(uname -m)" != "x86_64" ]; then
        error-and-exit "bootstrapper has only been validated on x86_64"
    fi
}

# Source: https://docs.docker.com/engine/install/fedora/
function install-docker {
    dnf -y install dnf-plugins-core
    dnf config-manager --add-repo https://download.docker.com/linux/fedora/docker-ce.repo
    dnf install -y docker-ce docker-ce-cli containerd.io docker-compose
    systemctl start docker
    systemctl enable docker
    docker run hello-world
}

verify-fedora
determine-user
if [ ! "$(command -v docker)" ] || [ ! "$(command -v docker-compose)" ]; then
    install-docker
fi