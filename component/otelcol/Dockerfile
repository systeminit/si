ARG OTELCOL_VERSION
ARG ALPINE_VERSION

FROM otel/opentelemetry-collector-contrib:$OTELCOL_VERSION AS otelcol

ENV SI_OTEL_COL__CONFIG_PATH=/etc/otelcol/config.yaml
ENV SI_OTEL_COL__HONEYCOMB_API_KEY=""

# We're going to base our image on a non-scratch image which should allow us to
# `docker exec` into a running instance for debugging
FROM alpine:${ALPINE_VERSION} AS final
WORKDIR /app
COPY --from=otelcol /otelcol-contrib /bin/otelcol
COPY ./*config.yaml /etc/otelcol/
COPY ./docker-entrypoint.sh /app/docker-entrypoint.sh

ENTRYPOINT ["/app/docker-entrypoint.sh"]
