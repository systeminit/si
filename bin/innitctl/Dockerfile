# hadolint ignore=DL3007
FROM nixos/nix:2.28.2 AS builder
ARG BIN=innitctl
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

FROM alpine:3 AS final
ARG BIN=innitctl

# hadolint ignore=DL3018
RUN set -eux; \
    apk add --no-cache runuser; \
    adduser -D app; \
    for dir in /run /etc /usr/local/etc /home/app/.config; do \
        mkdir -pv "$dir/$BIN"; \
    done; \
    mkdir -p /output && \
    chown -R app:app /output && \
    chmod -R 775 /output

WORKDIR /run/$BIN
COPY --from=builder /tmp/nix-store-closure /nix/store
COPY --from=builder /tmp/local-bin/* /usr/local/bin/
COPY --chown=app:app ./configs /configs

ENTRYPOINT ["/usr/local/bin/innitctl"]
