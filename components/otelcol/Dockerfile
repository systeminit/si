ARG VERSION
FROM otel/opentelemetry-collector:$VERSION

ARG VERSION
ARG CONFIG

COPY $CONFIG /etc/otel/config.yaml

# 4317 - OTLP receiver gRPC port
# 55679 - zPages port, an HTTP endpoint that provides live data for debugging
EXPOSE 4317 55679
