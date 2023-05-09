#!/usr/bin/env bash
set -euo pipefail
if [ "${1:-}" = "--verbose" -o "${1:-}" = "-V" ]; then
  set -x
fi

COMPOSE_FILE="${BUCK_DEFAULT_RUNTIME_RESOURCES}/component/deploy/docker-compose.yml"
LOCAL_PG_COMPOSE_FILE="${BUCK_DEFAULT_RUNTIME_RESOURCES}/component/deploy/docker-compose.local-postgres.yml"

# TODO(nick,fletcher): don't build local PG and host on docker hub instead.
POSTGRES_DIR="${BUCK_DEFAULT_RUNTIME_RESOURCES}/component/postgres/postgres"

# Get the local gateway and pull all images.
LOCAL_GATEWAY=$(docker run --rm busybox ip route | awk '/^default via/ { print $3 }')
docker compose -f ${COMPOSE_FILE} pull

# Run local PG if we are on aarch64. Otherwise, use the docker container.
if [ $(uname -m) = "arm64" ] || [ $(uname -m) = "aarch64" ]; then
  REPO_PORT=31337
  REPO_TAG=localhost:${REPO_PORT}/systeminit/postgres

  if [[ "$(docker image inspect ${REPO_TAG} --format='image exists' 2>/dev/null)" == "" ]]; then
    pushd ${POSTGRES_DIR}
    env BASE_VERSION=14.5-bullseye IMG=systeminit/postgres ./script/build-image.sh
    echo "tagging"
    docker tag systeminit/postgres:latest ${REPO_TAG}
    popd
  fi

  env GATEWAY=${LOCAL_GATEWAY} docker compose -f ${COMPOSE_FILE} -f ${LOCAL_PG_COMPOSE_FILE} up --detach
else
  env GATEWAY=${LOCAL_GATEWAY} docker compose -f ${COMPOSE_FILE} up --detach
fi

# Check if PG is ready!
if [ -z "$(pg_isready --version 2>/dev/null)" ]; then
  echo -n "'pg_isready' not available; sleeping to give PostgresQL time to start..."
  sleep 10
  echo "done"
else
  echo -n "trying PG"
  while :; do
    echo -n "."
    pg_isready -h localhost --timeout=1 -U si 2>&1 >/dev/null
    if [ "$?" = "0" ]; then
      break
    fi
    sleep 1
  done
  echo "ready"
fi
