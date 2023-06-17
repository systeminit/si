#!/usr/bin/env bash

main() {
  set -eu
  if [ -n "${DEBUG:-}" ]; then set -v; fi
  if [ -n "${TRACE:-}" ]; then set -xv; fi

  local version img name
  version="${VERSION:-latest}"
  img="${IMG:-systeminit/module-index}"
  name="${NAME:-module-index}"

  if [ -z "$(docker image ls --quiet "$img:$version")" ]; then
    echo "  - Image '$img:$version' not found, building"
    "${0%/*}/build-image.sh"
  fi

  if [ -n "$(docker container ls --filter "name=^$name" --filter "status=running" --quiet)" ]; then
    echo "  - Container $name is already running"
  elif [ -n "$(docker container ls --filter "name=^$name" --all --quiet)" ]; then
    echo "  - Starting stopped container $name"
    docker container start "$name"
  else
    local gateway
    echo "  - Determining gateway address for service discovery"
    gateway="$(docker run --rm busybox ip route \
      | awk '/^default via/ { print $3 }')"
    echo "  - Creating and starting container $name"
    cd "${0%/*}/.."

    set -x
    exec docker run --detach \
      --publish 5157:5157 \
      --add-host "postgres:$gateway" \
      --env SI_PINGA__PG__HOSTNAME=postgres \
      --env OTEL_EXPORTER_OTLP_ENDPOINT=http://otelcol:4317 \
      --name "$name" \
      "$@" \
      "$img:$version"
  fi
}

main "$@" || exit 1
