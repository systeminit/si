#!/usr/bin/env bash

main() {
  set -eu
  if [ -n "${DEBUG:-}" ]; then set -v; fi
  if [ -n "${TRACE:-}" ]; then set -xv; fi

  local version img name
  version="${VERSION:-latest}"
  img="${IMG:-systeminit/web}"
  name="${NAME:-web}"

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
    echo "  - Creating and starting container $name"
    cd "${0%/*}/.."

    set -x
    exec docker run --detach \
      --publish 8080:80 \
      --name "$name" \
      "$@" \
      "$img:$version"
  fi
}

main "$@" || exit 1
