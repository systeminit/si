ARG GOLANG_VERSION
ARG ALPINE_VERSION
ARG NATS_CLI_VERSION
FROM golang:${GOLANG_VERSION} AS ssm-builder

ARG SSM_PLUGIN_VERSION

RUN set -ex && apk add --no-cache make git gcc libc-dev curl bash zip && \
    curl -sLO https://github.com/aws/session-manager-plugin/archive/${SSM_PLUGIN_VERSION}.tar.gz && \
    mkdir -p /go/src/github.com && \
    tar xzf ${SSM_PLUGIN_VERSION}.tar.gz && \
    mv session-manager-plugin-${SSM_PLUGIN_VERSION} /go/src/github.com/session-manager-plugin && \
    cd /go/src/github.com/session-manager-plugin && \
    make release

FROM alpine:${ALPINE_VERSION} AS nats-builder

RUN apk add --no-cache curl unzip
RUN curl -sL https://github.com/nats-io/natscli/releases/download/${NATS_CLI_VERSION}/nats-${NATS_CLI_VERSION:1}-linux-amd64.zip -o nats.zip && \
    unzip nats.zip -d /tmp && \
    mkdir -p /nats-bin && \
    mv /tmp/nats-${NATS_CLI_VERSION:1}-linux-amd64/nats /nats-bin/

FROM alpine:${ALPINE_VERSION}

RUN set -eux; \
    apk add --no-cache jq aws-cli bash \
    rm -rf /var/cache/apk/*

COPY ./scripts/ /usr/local/bin/si/
RUN chmod +x /usr/local/bin/si/*

COPY --from=ssm-builder /go/src/github.com/session-manager-plugin/bin/linux_amd64_plugin/session-manager-plugin /usr/local/bin/
COPY --from=nats-builder /nats-bin/nats /usr/local/bin/nats

ENV PATH="/usr/local/bin/si:${PATH}"

ENTRYPOINT ["bash", "-c"]
