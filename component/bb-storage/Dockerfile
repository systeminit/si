FROM ghcr.io/buildbarn/bb-storage:20250408t112116z-1d733a3 AS bb
FROM alpine:latest

COPY config.yml /app/config.yml
COPY docker-entrypoint.sh /app/entrypoint.sh
COPY --from=bb /app/bb_storage /app/bb_storage

ENTRYPOINT ["/app/entrypoint.sh", "/app/bb_storage", "/app/config.yml"]
