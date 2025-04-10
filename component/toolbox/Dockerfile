FROM golang:1.15.3-alpine AS ssm-builder

ARG VERSION=1.2.279.0

RUN set -ex && apk add --no-cache make git gcc libc-dev curl bash zip && \
    curl -sLO https://github.com/aws/session-manager-plugin/archive/${VERSION}.tar.gz && \
    mkdir -p /go/src/github.com && \
    tar xzf ${VERSION}.tar.gz && \
    mv session-manager-plugin-${VERSION} /go/src/github.com/session-manager-plugin && \
    cd /go/src/github.com/session-manager-plugin && \
    make release

FROM alpine:latest AS nats-builder

RUN apk add --no-cache curl unzip
RUN curl -sL https://github.com/nats-io/natscli/releases/download/v0.2.1/nats-0.2.1-linux-amd64.zip -o nats.zip && \
    unzip nats.zip -d /tmp && \
    mkdir -p /nats-bin && \
    mv /tmp/nats-0.2.1-linux-amd64/nats /nats-bin/

FROM alpine

RUN set -eux; \
    apk add --no-cache jq aws-cli bash \
    rm -rf /var/cache/apk/*

COPY ./scripts/ /usr/local/bin/si/
RUN chmod +x /usr/local/bin/si/*

COPY --from=ssm-builder /go/src/github.com/session-manager-plugin/bin/linux_amd64_plugin/session-manager-plugin /usr/local/bin/
COPY --from=nats-builder /nats-bin/nats /usr/local/bin/nats

ENV PATH="/usr/local/bin/si:${PATH}"

ENTRYPOINT ["bash", "-c"]
