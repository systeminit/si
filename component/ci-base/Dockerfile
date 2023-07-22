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
    ; \
    rm -rf /var/lib/apt/lists/*

RUN set -eux; \
    useradd --create-home --shell /bin/bash --uid "${USER_UID}" ci; \
    echo 'ci ALL=(ALL:ALL) NOPASSWD: ALL' >/etc/sudoers.d/ci; \
    mkdir -p /workdir; \
    chown -R ci:ci /workdir

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
    nix-env -iA nixpkgs.nixFlakes; \
    grep 'Nix installer' "$HOME/.profile" >>"$HOME/.bashrc"; \
    git config --global --add safe.directory /workdir

WORKDIR /workdir

COPY flake.nix flake.lock rust-toolchain ./
COPY docker-entrypoint.sh /

RUN set -eux; \
    . "$HOME/.nix-profile/etc/profile.d/nix.sh"; \
    nix develop --build; \
    rm -f flake.nix flake.lock rust-toolchain

ENTRYPOINT ["/docker-entrypoint.sh"]
