#!/usr/bin/env sh
# shellcheck shell=sh disable=SC2039

main() {
  set -eu
  if [ -n "${DEBUG:-}" ]; then set -v; fi
  if [ -n "${TRACE:-}" ]; then set -xv; fi

  local image="systeminit/couchbase:latest"

  if image_built "$image"; then
    echo "--- Removing preexisting image '$image'"
    docker image rm "$image"
  fi

  # Change to directory containing Dockerfile context
  cd "${0%/*}"

  echo "--- Building image '$image'"
  cat <<-EOF >.env
	# Couchbase Memory Configuration
	export DATA_RAM_QUOTA=${DATA_RAM_QUOTA:-2048}
	export INDEX_RAM_QUOTA=${INDEX_RAM_QUOTA:-512}
	export FULL_TEXT_RAM_QUOTA=${FULL_TEXT_RAM_QUOTA:-512}
	export BUCKET_SI_RAM_QUOTA=${BUCKET_SI_RAM_QUOTA:-512}
	export BUCKET_SI_INTEGRATION_RAM_QUOTA=${BUCKET_SI_INTEGRATION_RAM_QUOTA:-512}
	EOF
  cat .env
  docker image build --tag "$image" .
}

image_built() {
  [ -n "$(docker image ls --quiet "$1")" ]
}

main "$@" || exit 1
