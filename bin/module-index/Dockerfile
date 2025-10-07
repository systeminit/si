ARG BASE_VERSION

FROM alpine:3 AS buck2

RUN apk add --no-cache curl zstd libc6-compat
RUN ARCH=$(uname -m); \
    case "$ARCH" in \
        x86_64) BUCK2_ARCH="x86_64";; \
        aarch64) BUCK2_ARCH="aarch64";; \
        *) echo "unsupported arch $ARCH" && exit 1;; \
    esac; \
    curl -L \
        "https://github.com/facebook/buck2/releases/download/latest/buck2-${BUCK2_ARCH}-unknown-linux-musl.zst" \
        -o /tmp/buck2.zst && \
    unzstd /tmp/buck2.zst -c > /bin/buck2 && chmod +x /bin/buck2

FROM alpine:3 AS builder
ARG BIN
ARG SI_RBE_TOKEN

COPY . /workdir
WORKDIR /workdir

COPY --from=buck2 /bin/buck2 /usr/local/bin/buck2

RUN buck2 build bin/$BIN:$BIN \
    --config rbe.enabled=true \
    --out /tmp/local-bin/$BIN

FROM debian:${BASE_VERSION} AS final
ARG BIN
ENV BIN=${BIN}

# hadolint ignore=DL3008,DL3009
RUN set -eux; \
    apt-get update; \
    apt-get install -y --no-install-recommends \
        runit \
        passwd \
        adduser \
        ca-certificates; \
    rm -rf /var/lib/apt/lists/*; \
    adduser --disabled-password --gecos '' app; \
    for dir in /run /etc /usr/local/etc /home/app/.config; do \
        mkdir -pv "$dir/$BIN"; \
    done;

WORKDIR /run/$BIN
COPY --from=builder /tmp/local-bin/* /usr/local/bin/
COPY --from=builder /workdir/config/keys/prod.jwt_signing_public_key.pem /run/module-index

USER app
ENTRYPOINT "/usr/local/bin/$BIN --jwt-public-key /run/module-index/prod.jwt_signing_public_key.pem
