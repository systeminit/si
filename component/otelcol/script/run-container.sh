#!/usr/bin/env bash

main() {
  set -eu
  if [ -n "${DEBUG:-}" ]; then set -v; fi
  if [ -n "${TRACE:-}" ]; then set -xv; fi

  local version img name
  version="${VERSION:-0.74.0}"
  img="${IMG:-otelcol}"
  name="${NAME:-otelcol}"

  if [ -z "$(docker image ls --quiet "$img:$version")" ]; then
    echo "  - Building image '$img:$version'"
    cd "${0%/*}/.."
    docker build --build-arg "VERSION=$version" -t "$img:$version" .
  fi

  if [ -n "$(docker container ls --filter "name=^$name" --filter "status=running" --quiet)" ]; then
    echo "  - Container $name is already running"
  elif [ -n "$(docker container ls --filter "name=^$name" --all --quiet)" ]; then
    echo "  - Starting stopped container $name"
    docker container start "$name"
  else
    echo "  - Creating and starting container $name"
    cd "${0%/*}/.."
    if [ -z "${HONEYCOMB_TOKEN:-}" ]; then
      echo "xxx HONEYCOMB_TOKEN must be set" >&2
      exit 1
    fi
    if [ -z "${HONEYCOMB_DATASET:-}" ]; then
      echo "xxx HONEYCOMB_DATASET must be set" >&2
      exit 1
    fi

    set -x
    exec docker run --detach \
      --env "HONEYCOMB_TOKEN=$HONEYCOMB_TOKEN" \
      --env "HONEYCOMB_DATASET=$HONEYCOMB_DATASET" \
      --publish 4317:4317 \
      --publish 55679:55679 \
      --name "$name" \
      "$@" \
      "$img:$version"
  fi
}

main "$@" || exit 1
