#!/usr/bin/env sh
# shellcheck shell=sh disable=SC2039

main() {
  set -eu
  if [ -n "${DEBUG:-}" ]; then set -v; fi
  if [ -n "${TRACE:-}" ]; then set -xv; fi

  local name="db"
  local image="systeminit/couchbase:latest"

  if is_running "$name"; then
    echo "--- Container '$name' is running, you're welcome"
    return
  fi

  if created "$name"; then
    echo "--- Starting stopped container '$name'"
    docker container start "$name"
  else
    if ! image_built "$image"; then
      "${0%/*}/build.sh"
    fi

    echo "--- Creating and running container '$name'"
    docker container run \
      --name "$name" \
      --detach \
      --publish 8091-8096:8091-8096 \
      --publish 11210-11211:11210-11211 \
      "$image"
  fi
}

created() {
  [ -n "$(docker container ls --filter "name=^$1" --all --quiet)" ]
}

image_built() {
  [ -n "$(docker image ls --quiet "$1")" ]
}

is_running() {
  [ -n "$(
    docker container ls --filter "name=^$1" --filter "status=running" --quiet
  )" ]
}

main "$@" || exit 1
