ARG BASE_VERSION

FROM postgres:${BASE_VERSION} as build_common
ARG BASE_VERSION

RUN set -eux; \
    grep -E '^deb\s' /etc/apt/sources.list.d/pgdg.list \
        | sed 's,^deb \(.*\)$,deb-src \1,' >> /etc/apt/sources.list.d/pgdg.list; \
    apt-get update; \
    apt-get install -y --no-install-recommends \
        ca-certificates \
        curl \
    ; \
    apt-get build-dep -y "postgresql-$PG_MAJOR=$PG_VERSION"; \
    apt-get install -y --no-install-recommends "postgresql-server-dev-$PG_MAJOR"

FROM build_common as build_pldebugger
ARG BASE_VERSION

ARG PLDEBUGGER_VERSION=1.5

RUN set -eux; \
    cd /usr/src; \
    curl -LO "https://github.com/EnterpriseDB/pldebugger/archive/refs/tags/v$PLDEBUGGER_VERSION.tar.gz"; \
    tar xf "v$PLDEBUGGER_VERSION.tar.gz"; \
    cd "pldebugger-$PLDEBUGGER_VERSION"; \
    make USE_PGXS=1; \
    make install USE_PGXS=1 DESTDIR=/tmp/tmproot; \
    tar cpf /pldebugger.tar -C /tmp/tmproot .

FROM build_common as build_plprofiler
ARG BASE_VERSION

ARG PLPROFILER_VERSION=REL4_2

RUN set -eux; \
    cd /usr/src; \
    curl -LO "https://github.com/bigsql/plprofiler/archive/refs/tags/$PLPROFILER_VERSION.tar.gz"; \
    tar xf "$PLPROFILER_VERSION.tar.gz"; \
    cd "plprofiler-$PLPROFILER_VERSION"; \
    make install USE_PGXS=1 DESTDIR=/tmp/tmproot; \
    tar cpf /plprofiler.tar -C /tmp/tmproot .

FROM build_common as build_plpgsql_check
ARG BASE_VERSION

ARG PLPGSQL_CHECK_VERSION=2.2.2

RUN set -eux; \
    cd /usr/src; \
    curl -LO "https://github.com/okbob/plpgsql_check/archive/refs/tags/v$PLPGSQL_CHECK_VERSION.tar.gz"; \
    tar xf "v$PLPGSQL_CHECK_VERSION.tar.gz"; \
    cd "plpgsql_check-$PLPGSQL_CHECK_VERSION"; \
    make USE_PGXS=1; \
    make install USE_PGXS=1 DESTDIR=/tmp/tmproot; \
    tar cpf /plpgsql_check.tar -C /tmp/tmproot .

FROM postgres:${BASE_VERSION} as final
ARG BASE_VERSION

COPY --from=build_pldebugger /pldebugger.tar /pldebugger.tar
COPY --from=build_plprofiler /plprofiler.tar /plprofiler.tar
COPY --from=build_plpgsql_check /plpgsql_check.tar /plpgsql_check.tar

RUN set -eux; \
    apt-get update; \
    apt-get install -y --no-install-recommends \
        busybox \
        ca-certificates \
        cron \
        curl \
        logrotate \
        procps \
    ; \
    rm -rf /var/lib/apt/lists/*; \
    tar xpf /pldebugger.tar -C /; \
    tar xpf /plprofiler.tar -C /; \
    tar xpf /plpgsql_check.tar -C /; \
    rm -f /pldebugger.tar /plprofiler.tar /plpgsql_check.tar

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
