#!/usr/bin/env sh
set -eu

config="${SI_OTEL_COL__CONFIG_PATH-/etc/otelcol/config.yaml}"

sed \
  -i "s/SI_OTEL_COL__HONEYCOMB_API_KEY/${SI_OTEL_COL__HONEYCOMB_API_KEY:-}/g" \
  "$config"

exec /bin/otelcol --config "$config"
