#!/usr/bin/env sh
# shellcheck shell=sh disable=SC2039

main() {
  set -eu
  if [ -n "${DEBUG:-}" ]; then set -v; fi
  if [ -n "${TRACE:-}" ]; then set -xv; fi

  local name="pg"
  local image="systeminit/pg:latest"

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
      --publish 5432:5432 \
      "$image" \
      -c log_error_verbosity=VERBOSE \
      -c log_statement=all 
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
