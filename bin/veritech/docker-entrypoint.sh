#!/bin/sh
# shellcheck disable=SC3043
set -eu

main() {
  write_aws_credentials
  write_docker_credentials

  exec /usr/local/bin/.veritech "$@"
}

write_aws_credentials() {
  ensure_env_var AWS_ACCESS_KEY_ID "${AWS_ACCESS_KEY_ID:-}"
  ensure_env_var AWS_SECRET_ACCESS_KEY "${AWS_SECRET_ACCESS_KEY:-}"

  mkdir -p "$HOME/.aws"
  chmod 0755 "$HOME/.aws"
  cat <<-EOF >"$HOME/.aws/credentials"
	[default]
	aws_access_key_id = ${AWS_ACCESS_KEY_ID:-}
	aws_secret_access_key = ${AWS_SECRET_ACCESS_KEY:-}
	EOF
  chmod 0600 "$HOME/.aws/credentials"

  # Remove environment variables from veritech's environment
  unset AWS_ACCESS_KEY_ID AWS_SECRET_ACCESS_KEY
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
