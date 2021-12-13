#!/usr/bin/env bash
set -e

SI_USER="unknown"
SI_OS="unknown"
SI_LINUX="unknown"
SI_WSL2="unknown"
SI_ARCH="unknown"

function error-and-exit {
    if [ "$1" = "" ]; then
        error-and-exit "must provide argument <error-message>"
    fi
    echo "error: $1"
    exit 1
}

function determine-user {
    if [ $EUID -eq 0 ]; then
        error-and-exit "must run as non-root"
    fi
    SI_USER=$(whoami)
    sudo -v
}

function set-config {
    # Since arm64 has many names, we standardize on one to reduce verbosity within this script.
    SI_ARCH="$(uname -m)"
    if [ "$SI_ARCH" = "aarch64" ]; then
        SI_ARCH="arm64"
    fi

    if [ "$(uname -s)" = "Darwin" ]; then
        SI_OS="darwin"
        SI_LINUX="false"
        SI_WSL2="false"
    elif [ "$(uname -s)" = "Linux" ]; then
        SI_LINUX="true"

        if [ -f /etc/os-release ]; then
            SI_OS=$(grep '^ID=' /etc/os-release | sed 's/^ID=//' | tr -d '"')
        else
            error-and-exit "file \"/etc/os-release\" not found"
        fi

        SI_WSL2="false"
        if [ -f /proc/sys/kernel/osrelease ] && [ $(grep "WSL2" /proc/sys/kernel/osrelease) ]; then
            SI_WSL2="true"
        fi
    else
        error-and-exit "detected OS is neither Darwin nor Linux"
    fi
}

function arch-bootstrap {
    sudo pacman -Syu --noconfirm base-devel make git
}

function darwin-bootstrap {
    brew update
    brew upgrade
    brew cleanup
    brew install bash make git
}

function fedora-bootstrap {
    sudo dnf upgrade -y --refresh
    sudo dnf autoremove -y
    sudo dnf install -y @development-tools make git lld
}

function perform-bootstrap {
    if [ "$SI_OS" = "darwin" ] && ( [ "$SI_ARCH" = "x86_64" ] || [ "$SI_ARCH" = "arm64" ] ); then
        darwin-bootstrap
    elif [ "$SI_OS" = "arch" ] && [ "$SI_ARCH" = "x86_64" ]; then
        arch-bootstrap
    elif [ "$SI_OS" = "fedora" ] && [ "$SI_ARCH" = "x86_64" ]; then
        fedora-bootstrap
    else
        error-and-exit "detected distro \"$SI_OS\" and architecture \"$SI_ARCH\" combination have not yet been validated"
    fi
}

function check-binaries {
    for BINARY in "cargo" "node" "npm" "docker"; do
        if ! [ "$(command -v ${BINARY})" ]; then
            error-and-exit "\"$BINARY\" must be installed and in PATH"
        fi
    done
}

function check-node {
    # Check added due to vercel/pkg requirement: https://github.com/vercel/pkg/issues/838
    if [ "$(node -pe process.release.lts)" = "undefined" ] && [ "$SI_OS" = "darwin"] && [ "$SI_ARCH" = "arm64"]; then
        error-and-exit "must use an LTS release of node for \"$SI_OS\" on \"$SI_ARCH\""
    fi
}

# We use empty echo payloads instead of newline characters for readability.
function print-success {
    echo ""
    echo "Success! Ready to build System Initiative with your config 🦀:"
    echo ""
    echo "user  : $SI_USER"
    echo "os    : $SI_OS"
    echo "arch  : $SI_ARCH"
    echo "linux : $SI_LINUX"
    echo "wsl2  : $SI_WSL2"
    echo ""
}

determine-user
set-config
perform-bootstrap
check-binaries
check-node
print-success
