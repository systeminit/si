ARG PGBOUNCER_PROM_EXPORTER_VERSION
FROM prometheuscommunity/pgbouncer-exporter:${PGBOUNCER_PROM_EXPORTER_VERSION}
WORKDIR /app
COPY ./docker-entrypoint.sh /app/
COPY ./pgbouncer-conn-string-exporter.sh /app/
ENTRYPOINT ["/app/docker-entrypoint.sh"]