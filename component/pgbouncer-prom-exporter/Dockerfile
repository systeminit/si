ARG BASE_VERSION
FROM prometheuscommunity/pgbouncer-exporter:${BASE_VERSION}
WORKDIR /app
COPY ./docker-entrypoint.sh /app/
COPY ./pgbouncer-conn-string-exporter.sh /app/
ENTRYPOINT ["/app/docker-entrypoint.sh"]