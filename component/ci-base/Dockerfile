ARG BASE_VERSION
FROM alpine:${BASE_VERSION}

# hadolint ignore=DL3018
RUN set -eux; \
    apk add --no-cache \
        bash \
        curl \
        docker \
        git \
        sudo \
        xz \
    ; \
    addgroup -g 2000 ci; \
    adduser -h /home/ci -s /bin/bash -G ci -u 2000 -D ci; \
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
        echo 'experimental-features = nix-command flakes'; \
        echo 'auto-optimise-store = true'; \
        echo 'bash-prompt-prefix = (nix:$name)\040'; \
    } >"$HOME/.config/nix/nix.conf"; \
    . "$HOME/.nix-profile/etc/profile.d/nix.sh"; \
    nix-env -iA nixpkgs.nixFlakes; \
    git config --global --add safe.directory /workdir

WORKDIR /workdir

COPY flake.nix flake.lock rust-toolchain ./
COPY docker-entrypoint.sh /

RUN set -eux; \
    . "$HOME/.nix-profile/etc/profile.d/nix.sh"; \
    nix develop --build; \
    rm -f flake.nix flake.lock rust-toolchain

ENTRYPOINT ["/docker-entrypoint.sh"]
