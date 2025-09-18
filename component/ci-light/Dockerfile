FROM alpine:3 AS buck2

RUN apk add --no-cache curl zstd libc6-compat
RUN ARCH=$(uname -m); \
    case "$ARCH" in \
        x86_64) BUCK2_ARCH="x86_64";; \
        aarch64) BUCK2_ARCH="aarch64";; \
        *) echo "unsupported arch $ARCH" && exit 1;; \
    esac; \
    curl -L \
        "https://github.com/facebook/buck2/releases/download/latest/buck2-${BUCK2_ARCH}-unknown-linux-gnu.zst" \
        -o /tmp/buck2.zst && \
    unzstd /tmp/buck2.zst -c > /bin/buck2 && chmod +x /bin/buck2

FROM rust:1-slim-bookworm

ARG BUILDEVENTS_VERSION=v0.17.0
RUN apt-get update && apt-get install -y \
    curl \
    bash \
    git \
    gh \
    openssh-client \
    python3 \
    ca-certificates \
    gnupg \
    lsb-release \
    b3sum \
    && curl -fsSL https://download.docker.com/linux/debian/gpg | gpg --dearmor -o /usr/share/keyrings/docker-archive-keyring.gpg \
    && echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/docker-archive-keyring.gpg] https://download.docker.com/linux/debian $(lsb_release -cs) stable" | tee /etc/apt/sources.list.d/docker.list > /dev/null \
    && apt-get update \
    && apt-get install -y docker-ce-cli docker-buildx-plugin docker-compose-plugin \
    && rm -rf /var/lib/apt/lists/*

RUN set -eux; \
    ARCH="$(arch)"; \
    case "${ARCH}" in \
        x86_64) BUILDEVENTS_ARCH="linux-amd64" ;; \
        aarch64) BUILDEVENTS_ARCH="linux-arm64" ;; \
        *) echo "Unsupported architecture: ${ARCH}" && exit 1 ;; \
    esac && \
    curl -L -o /usr/local/bin/buildevents \
        "https://github.com/honeycombio/buildevents/releases/download/${BUILDEVENTS_VERSION}/buildevents-${BUILDEVENTS_ARCH}" && \
    chmod +x /usr/local/bin/buildevents

COPY --from=buck2 /bin/buck2 /usr/local/bin/buck2

RUN rustup component add clippy rustfmt
