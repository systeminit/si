ARG BASE_VERSION
FROM debian:${BASE_VERSION}

ARG USER_UID
ARG USER_GID

# Install base system requirements
# hadolint ignore=DL3008
RUN set -eux; \
    apt-get update; \
    apt-get install -y --no-install-recommends \
        busybox \
        ca-certificates \
        curl \
        git \
        gnupg \
        openssh-client \
        sudo \
        xz-utils \
    ; \
    rm -rf /var/lib/apt/lists/*

# Install buildevents
ARG BUILDEVENTS_VERSION=v0.17.0
RUN set -eux; \
    ARCH="$(dpkg --print-architecture)"; \
    case "${ARCH}" in \
        amd64) BUILDEVENTS_ARCH="linux-amd64" ;; \
        arm64) BUILDEVENTS_ARCH="linux-arm64" ;; \
        *) echo "Unsupported architecture: ${ARCH}" && exit 1 ;; \
    esac && \
    curl -L -o /usr/local/bin/buildevents \
        "https://github.com/honeycombio/buildevents/releases/download/${BUILDEVENTS_VERSION}/buildevents-${BUILDEVENTS_ARCH}" && \
    chmod +x /usr/local/bin/buildevents

# Setup workspace and git configuration
RUN set -eux; \
    mkdir -p /workspace; \
    git config --global --add safe.directory /workspace

# Install Nix as root with daemon mode
RUN set -eux; \
    curl -L https://nixos.org/nix/install | sh -s -- --daemon; \
    mkdir -p /root/.config/nix; \
    { \
        echo 'extra-nix-path = nixpkgs=flake:nixpkgs'; \
        echo 'experimental-features = nix-command flakes impure-derivations ca-derivations'; \
        echo 'auto-optimise-store = true'; \
        echo 'bash-prompt-prefix = (nix:$name)\040'; \
    } >"/root/.config/nix/nix.conf"; \
    mkdir -p "/root/.ssh"; \
    chmod 0700 "/root/.ssh"; \
    ssh-keyscan github.com >>"/root/.ssh/known_hosts"

WORKDIR /workspace

# Copy flake configuration and prebuild Nix environment
COPY flake.nix flake.lock rust-toolchain ./

RUN set -eux; \
    . /etc/profile.d/nix.sh; \
    nix develop .#ci --build; \
    CI_TOOLS=$(nix build --no-link --print-out-paths .#ci-tools); \
    cp -rs "$CI_TOOLS/bin/"* /usr/local/bin/ 2>/dev/null || true; \
    cp -rs "$CI_TOOLS/share" /usr/local/ 2>/dev/null || true

