# hadolint ignore=DL3007
FROM nixos/nix:latest AS builder
ARG APP=web

COPY . /workdir
WORKDIR /workdir

RUN set -eux; \
    nix \
        --extra-experimental-features "nix-command flakes impure-derivations ca-derivations" \
        --option filter-syscalls false \
        build \
        ".#$APP";

RUN mkdir -p /tmp/nix-store-closure /tmp/local-bin
# hadolint ignore=SC2046
RUN cp -R $(nix-store --query --requisites result/) /tmp/nix-store-closure
# hadolint ignore=SC2046
RUN ln -snf $(nix-store --query result/)/bin/* /tmp/local-bin/

FROM alpine:3 AS final
ARG APP=web

# hadolint ignore=DL3018
RUN set -eux; \
    apk add --no-cache runuser; \
    for dir in /run /etc /usr/local/etc; do \
        mkdir -pv "$dir/$APP"; \
    done; \
    mkdir -pv /var/log/nginx; \
    ln -snf /var/log/nginx "/run/$APP/log";


WORKDIR /run/$APP
COPY --from=builder /tmp/nix-store-closure /nix/store
COPY --from=builder /tmp/local-bin/* /usr/local/bin/

ENTRYPOINT [ \
  "/usr/local/bin/web" \
]
