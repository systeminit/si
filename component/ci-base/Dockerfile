ARG BASE_VERSION
FROM debian:${BASE_VERSION}

ARG USER_UID
ARG USER_GID

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
    rm -rf /var/lib/apt/lists/*; \
    \
    install -m 0755 -d /etc/apt/keyrings; \
    curl -fsSL https://download.docker.com/linux/debian/gpg \
        | gpg --dearmor -o /etc/apt/keyrings/docker.gpg;\
    chmod a+r /etc/apt/keyrings/docker.gpg; \
    echo "deb [arch="$(dpkg --print-architecture)" signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/debian "$(. /etc/os-release && echo "$VERSION_CODENAME")" stable" \
        | tee /etc/apt/sources.list.d/docker.list > /dev/null; \
    apt-get update; \
    apt-get install -y --no-install-recommends \
        docker-ce \
        docker-ce-cli \
        containerd.io \
        docker-buildx-plugin \
        docker-compose-plugin \
        wget \
    ; \
    rm -rf /var/lib/apt/lists/*

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

# Install Cypress Dependencies
RUN apt-get update; \
    apt-get install -y --no-install-recommends \
        xvfb \
        libgtk2.0-0 \
        libnss3 \
        libatk-bridge2.0-0 \
        libgtk-3-0 \
        libgdm-dev \
        libgbm1 \
        libwayland-server0 \
        libasound2 \
    ; \
    rm -rf /var/lib/apt/lists/*

RUN set -eux; \
    useradd --create-home --shell /bin/bash --uid "${USER_UID}" ci; \
    echo 'ci ALL=(ALL:ALL) NOPASSWD: ALL' >/etc/sudoers.d/ci; \
    mkdir -p /workdir; \
    chown -R ci:ci /workdir

RUN groupadd -f docker && usermod -aG docker ci

ENV USER=ci
USER ci:ci

# hadolint ignore=DL3004,DL4006
RUN set -eux; \
    sudo install -d -m755 -o $(id -u) -g $(id -g) /nix; \
    curl \
            --proto '=https' \
            --tlsv1.2 -sSf \
            -L https://nixos.org/nix/install \
        | sh -s -- --no-daemon; \
    mkdir -p "$HOME/.config/nix"; \
    { \
        echo 'extra-nix-path = nixpkgs=flake:nixpkgs'; \
        echo 'experimental-features = nix-command flakes impure-derivations ca-derivations'; \
        echo 'auto-optimise-store = true'; \
        echo 'bash-prompt-prefix = (nix:$name)\040'; \
    } >"$HOME/.config/nix/nix.conf"; \
    . "$HOME/.nix-profile/etc/profile.d/nix.sh"; \
    nix-env -iA nixpkgs.nixVersions.stable; \
    grep 'Nix installer' "$HOME/.profile" >>"$HOME/.bashrc"; \
    \
    git config --global --add safe.directory /workdir; \
    mkdir -p "$HOME/.ssh"; \
    chmod 0700 "$HOME/.ssh"; \
    ssh-keyscan github.com >>"$HOME/.ssh/known_hosts";

WORKDIR /workdir

COPY flake.nix flake.lock rust-toolchain ./
COPY docker-entrypoint.sh /

RUN set -eux; \
    . "$HOME/.nix-profile/etc/profile.d/nix.sh"; \
    nix develop --build; \
    rm -f flake.nix flake.lock rust-toolchain

ENTRYPOINT ["/docker-entrypoint.sh"]
