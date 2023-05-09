ARG BASE_VERSION

FROM postgres:${BASE_VERSION}
ARG BASE_VERSION

# RUN apk add --no-cache rsyslog

RUN set -eux; \
    apt-get update; \
    apt-get install -y --no-install-recommends \
        ca-certificates \
        curl \
        busybox \
        procps \
    ; \
    rm -rf /var/lib/apt/lists/*

ARG TARGETPLATFORM
RUN set -eux; \
    case "$TARGETPLATFORM" in \
        linux/arm64) export PGA_BINARY="pganalyze-collector-linux-arm64" ;; \
        linux/amd64) export PGA_BINARY="pganalyze-collector-linux-amd64" ;; \
    esac; \
    \
    curl -L -o /usr/bin/pganalyze-collector https://github.com/pganalyze/collector/releases/latest/download/${PGA_BINARY} \
    && chmod +x /usr/bin/pganalyze-collector

RUN addgroup pganalyze \
    && adduser --gecos "" --disabled-password pganalyze --ingroup pganalyze

COPY multiple-database-support.sh /docker-entrypoint-initdb.d/
COPY entrypoint-wrapper.sh /usr/local/bin/entrypoint-wrapper.sh
COPY pganalyze-collector.conf.sample /etc/pganalyze-collector.conf.sample
COPY postgresql-additions.conf /etc/postgresql-additions.conf

RUN chmod +x /usr/local/bin/entrypoint-wrapper.sh \
    && mkdir -p /etc/postgresql \
    && cp /usr/share/postgresql/postgresql.conf.sample /etc/postgresql/postgresql.conf \
    && cat /etc/postgresql-additions.conf >> /etc/postgresql/postgresql.conf

ENTRYPOINT ["/usr/local/bin/entrypoint-wrapper.sh"]
CMD ["postgres"]
