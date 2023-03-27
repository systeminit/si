ARG BASE_VERSION

FROM otel/opentelemetry-collector:$BASE_VERSION AS otelcol

# We're going to base our image on a non-scratch image which should allow us to
# `docker exec` into a running instance for debugging
FROM alpine:3 AS final
COPY --from=otelcol /otelcol /bin/otelcol
COPY component/otelcol/config.yaml /etc/otelcol/config.yaml

ENTRYPOINT ["/bin/otelcol"]
CMD ["--config", "/etc/otelcol/config.yaml"]
