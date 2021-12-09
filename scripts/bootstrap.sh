#!/usr/bin/env bash
set -e

SI_OS="unknown"
SI_LINUX="unknown"
SI_WSL2="unknown"
SI_ARCH="unknown"

function arch-bootstrap {
    pacman -Syu --noconfirm base-devel make sudo git
}

function darwin-bootstrap {
    brew update
    brew upgrade
    brew cleanup
    brew install bash make git
}

function fedora-bootstrap {
    echo "error: fedora not yet supported, but will likely be soon"
    exit 1
    dnf upgrade -y --refresh
    dnf autoremove -y
    dnf install -y @development-tools make sudo git
}

function determine-architecture {
    # Since arm64 has many names, we standardize on one to reduce verbosity within this script.
    SI_ARCH="$(uname -m)"
    if [ "$SI_ARCH" = "aarch64" ]; then
        SI_ARCH="arm64"
    fi

    if [ "$SI_ARCH" != "x86_64" ] && [ "$SI_ARCH" != "arm64" ]; then
        echo "error: detected architecture \"$SI_ARCH\" has not yet been validated"
        exit 1
    fi
}

function perform-bootstrap {
    determine-architecture

    if [ "$(uname -s)" = "Darwin" ]; then
        SI_OS="darwin"
        SI_LINUX="false"
        SI_WSL2="false"
        darwin-bootstrap
    elif [ "$(uname -s)" = "Linux" ]; then
        SI_LINUX="true"

        if [ -f /etc/os-release ]; then
            SI_OS=$(grep '^ID=' /etc/os-release | sed 's/^ID=//' | tr -d '"')

            # Arch Linux only officially supports amd64.
            if [ "$SI_OS" = "arch" ] && [ "$SI_ARCH" = "x86_64" ]; then
                arch-bootstrap
            elif [ "$SI_OS" = "fedora" ]; then
                fedora-bootstrap
            else
                echo "error: detected distro \"$SI_OS\" and architecture \"$SI_ARCH\" combination have not yet been validated"
                exit 1
            fi
        else
           echo "error: file \"/etc/os-release\" not found"
           exit 1
        fi

        if [ -f /proc/sys/kernel/osrelease ] && [ $(grep "WSL2" /proc/sys/kernel/osrelease) ]; then
            SI_WSL2="true"
        else
            SI_WSL2="false"
        fi
    else
        echo "error: detected OS is neither Darwin nor Linux"
        exit 1
    fi
}

function check-binaries {
    for BINARY in "cargo" "node" "npm" "docker"; do
        if ! [ "$(command -v ${BINARY})" ]; then
            echo "error: \"$BINARY\" must be installed and in PATH"
            exit 1
        fi
    done
}

function check-node {
    # Check added due to vercel/pkg requirement: https://github.com/vercel/pkg/issues/838
    if [ "$(node -pe process.release.lts)" = "undefined" ] && [ "$SI_OS" = "darwin"] && [ "$SI_ARCH" = "arm64"]; then
        echo "error: must use an LTS release of node for \"$SI_OS\" on \"$SI_ARCH\""
        exit 1
    fi
}

# We use empty echo payloads instead of newline characters for readability.
function print-success {
    echo ""
    echo "Success! Ready to build System Initiative with your config 🦀:"
    echo ""
    echo "os    : $SI_OS"
    echo "arch  : $SI_ARCH"
    echo "linux : $SI_LINUX"
    echo "wsl2  : $SI_WSL2"
    echo ""
}

perform-bootstrap
check-binaries
check-node
print-success
