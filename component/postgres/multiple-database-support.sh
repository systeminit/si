#!/bin/bash
# Solution ported from:
# https://developpaper.com/docker-is-a-solution-to-create-multiple-databases-when-starting-postgresql/

main() {
  set -euo pipefail
  if [[ -n "${DEBUG:-}" ]]; then set -v; fi
  if [[ -n "${TRACE:-}" ]]; then set -xv; fi

  if [[ -z "${POSTGRES_MULTIPLE_DBS:-}" ]]; then
    return 0
  fi
  if [[ -z "${POSTGRES_USER:-}" ]]; then
    die "Missing required environment variable: POSTGRES_USER"
  fi
  if [[ -z "${POSTGRES_PASSWORD:-}" ]]; then
    die "Missing required environment variable: POSTGRES_PASSWORD"
  fi

  echo "--- Creating multiple databases; triggered by POSTGRES_MULTIPLE_DBS"
  # shellcheck shell=sh disable=SC2207
  dbs=($(echo "$POSTGRES_MULTIPLE_DBS" | tr ',' '\n'))
  local db
  for db in "${dbs[@]}"; do
    create_db "$db" || die "failed to create database '$db'"
  done
}

create_db() {
  local db="$1"

  echo "  - Creating user='$db' and database='$db'"
  psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" <<-EOF
	CREATE DATABASE $db;
	GRANT ALL PRIVILEGES ON DATABASE $db to $POSTGRES_USER;
	EOF
}

die() {
  printf -- "\nxxx %s\n\n" "$1" >&2
  exit 1
}

main "$@" || exit 1
