#!/usr/bin/env bash
set -euo pipefail
if [ "${1:-}" = "--verbose" -o "${1:-}" = "-V" ]; then
  set -x
fi

COMPOSE_FILE="${BUCK_DEFAULT_RUNTIME_RESOURCES}/component/deploy/docker-compose.yml"
docker compose -f ${COMPOSE_FILE} --profile si down || true
