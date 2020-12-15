#!/usr/bin/env sh
# shellcheck shell=sh disable=SC2039

main() {
  set -eu
  if [ -n "${DEBUG:-}" ]; then set -v; fi
  if [ -n "${TRACE:-}" ]; then set -xv; fi

  local image="systeminit/pg:latest"

  if image_built "$image"; then
    echo "--- Removing preexisting image '$image'"
    docker image rm "$image"
  fi

  # Change to directory containing Dockerfile context
  cd "${0%/*}"

  echo "--- Building image '$image'"
  docker image build --tag "$image" .
}

image_built() {
  [ -n "$(docker image ls --quiet "$1")" ]
}

main "$@" || exit 1
