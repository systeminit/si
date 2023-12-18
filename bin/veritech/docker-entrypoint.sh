#!/bin/sh
# shellcheck disable=SC3043
set -eu

main() {
  write_docker_credentials

  exec /usr/local/bin/.veritech "$@"
}

write_docker_credentials() {
  ensure_env_var DOCKER_AUTHENTICATION "${DOCKER_AUTHENTICATION:-}"

  local auth_key="https://index.docker.io/v1/"

  mkdir -p "$HOME/.docker"
  chmod 0700 "$HOME/.docker"
  cat <<-EOF >"$HOME/.docker/config.json"
	{"auths":{"$auth_key":{"auth":"${DOCKER_AUTHENTICATION:-}"}}}
	EOF
  chmod 0600 "$HOME/.docker/config.json"

  # Remove environment variables from veritech's environment
  unset DOCKER_AUTHENTICATION
}

ensure_env_var() {
  local name="$1"
  local value="$2"

  if [ -z "$value" ]; then
    echo "xxx" >&2
    echo "xxx Missing required environment variable: '$name', aborting" >&2
    echo "xxx" >&2
    exit 1
  fi
}

main "$@"
