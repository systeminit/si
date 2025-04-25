ARG CI_LIGHT_RUST_VERSION
FROM rust:${CI_LIGHT_RUST_VERSION}

ARG BUILDEVENTS_VERSION
RUN apk add --no-cache \
    curl \
    bash \
    git \
    github-cli \
    musl-dev \
    openssh-client \
    python3

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

RUN rustup component add clippy rustfmt

