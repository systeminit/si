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

RUN set -eux; \
    curl https://packages.pganalyze.com/collector-install.sh \
        | env PGA_INSTALL_NONINTERACTIVE=1 bash

COPY component/postgres/multiple-database-support.sh /docker-entrypoint-initdb.d/
COPY component/postgres/entrypoint-wrapper.sh /usr/local/bin/entrypoint-wrapper.sh
COPY component/postgres/pganalyze-collector.conf.sample /etc/pganalyze-collector.conf.sample
COPY component/postgres/postgresql-additions.conf /etc/postgresql-additions.conf

RUN chmod +x /usr/local/bin/entrypoint-wrapper.sh \
    && mkdir -p /etc/postgresql \
    && cp /usr/share/postgresql/postgresql.conf.sample /etc/postgresql/postgresql.conf \
    && cat /etc/postgresql-additions.conf >> /etc/postgresql/postgresql.conf

ENTRYPOINT ["/usr/local/bin/entrypoint-wrapper.sh"]
CMD ["postgres"]
