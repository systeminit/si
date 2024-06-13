#!/usr/bin/env bash
set -eu

main() {
  if [ -n "${SI__WEB__SDF_HOST:-}" ]; then
    echo "Info: SI__WEB__SDF_HOST supplied [ $SI__WEB__SDF_HOST ]! Registering in nginx.conf"
    sed -i "s^server sdf:5156^server $SI__WEB__SDF_HOST^g" "$(find /nix/store/ -path '/nix/store/*web/conf/nginx.conf')"
  fi

  if [ -n "${VITE_OTEL_EXPORTER_OTLP_ENDPOINT:-}" ]; then
    echo "Info: VITE_OTEL_EXPORTER_OTLP_ENDPOINT supplied [ $VITE_OTEL_EXPORTER_OTLP_ENDPOINT ]! Setting in web"
    projectEnvVariables=$(find /nix/store -name "projectEnvVar*.js" | head -n 1)
    tmp=$(mktemp)
    envsubst <"$projectEnvVariables" >"$tmp"
    mv "$tmp" "$projectEnvVariables"
    chmod +rx "$projectEnvVariables"
  fi
  exec @@nginx@@ -c @@conf@@ -p @@prefix@@ -g "daemon off;" "$@"
}

main "$@"
