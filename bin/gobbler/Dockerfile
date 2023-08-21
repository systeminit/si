# hadolint ignore=DL3007
FROM nixos/nix:latest AS builder
ARG BIN=gobbler

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

FROM alpine:3 AS final
ARG BIN=gobbler

# hadolint ignore=DL3018
RUN set -eux; \
    apk add --no-cache runuser; \
    adduser -D app; \
    for dir in /run /etc /usr/local/etc /home/app/.config; do \
        mkdir -pv "$dir/$BIN"; \
    done;

WORKDIR /run/$BIN
COPY --from=builder /tmp/nix-store-closure /nix/store
COPY --from=builder /tmp/local-bin/* /usr/local/bin/

ENTRYPOINT [ \
  "/sbin/runuser", "-u", "app", "--", "/usr/local/bin/gobbler" \
]
