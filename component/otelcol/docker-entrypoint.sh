#!/bin/sh

sed -i "s/SI_OTEL_COL__HONEYCOMB_API_KEY/$SI_OTEL_COL__HONEYCOMB_API_KEY/g" ${SI_OTEL_COL__CONFIG_PATH-/etc/otelcol/config.yaml}

/bin/otelcol --config ${SI_OTEL_COL__CONFIG_PATH-/etc/otelcol/config.yaml}