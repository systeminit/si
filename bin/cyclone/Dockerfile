###########################################################################
# Builder Stage: cyclone
###########################################################################
# hadolint ignore=DL3007
FROM nixos/nix:2.28.2 AS builder-cyclone
ARG BIN=cyclone
ARG SI_RBE_TOKEN

COPY . /workdir
WORKDIR /workdir

RUN set -eux; \
    nix \
        --extra-experimental-features "nix-command flakes impure-derivations ca-derivations" \
        --option filter-syscalls false \
        build \
        --impure \
        ".#$BIN";

RUN mkdir -p /tmp/nix-store-closure /tmp/local-bin
# hadolint ignore=SC2046
RUN cp -R $(nix-store --query --requisites result/) /tmp/nix-store-closure
# hadolint ignore=SC2046
RUN ln -snf $(nix-store --query result/)/bin/* /tmp/local-bin/

# Add wrapper/entrypoint for `$BIN` to exec `.$BIN`
RUN set -eux; \
    mv -v "/tmp/local-bin/$BIN" "/tmp/local-bin/.$BIN"; \
    cp -pv /workdir/bin/$BIN/docker-entrypoint.sh "/tmp/local-bin/$BIN";

###########################################################################
# Builder Stage: lang-js
###########################################################################
# hadolint ignore=DL3007
FROM nixos/nix:2.28.2 AS builder-lang-js
ARG BIN=lang-js

COPY . /workdir
WORKDIR /workdir

RUN set -eux; \
    nix \
        --extra-experimental-features "nix-command flakes impure-derivations ca-derivations" \
        --option filter-syscalls false \
        build \
        ".#$BIN";

RUN mkdir -p /tmp/nix-store-closure /tmp/local-bin
# hadolint ignore=SC2046
RUN cp -R $(nix-store --query --requisites result/) /tmp/nix-store-closure
# hadolint ignore=SC2046
RUN ln -snf $(nix-store --query result/)/bin/* /tmp/local-bin/

###########################################################################
# Final Stage
###########################################################################
FROM alpine:3.18 AS final
ARG BIN=cyclone

# hadolint ignore=DL3018
RUN set -eux; \
    apk add --no-cache runuser; \
    adduser -D app; \
    for dir in /run /etc /usr/local/etc /home/app/.config; do \
        mkdir -pv "$dir/$BIN"; \
    done;

WORKDIR /run/$BIN
COPY --from=builder-cyclone /tmp/nix-store-closure /nix/store
COPY --from=builder-cyclone /tmp/local-bin/* /usr/local/bin/
COPY --from=builder-lang-js /tmp/nix-store-closure /nix/store
COPY --from=builder-lang-js /tmp/local-bin/* /usr/local/bin/

ENTRYPOINT [ \
  "/sbin/runuser", "-u", "app", "--", "/usr/local/bin/cyclone" \
]
