#!/usr/bin/env bash

main() {
  set -eu
  if [ -n "${DEBUG:-}" ]; then set -v; fi
  if [ -n "${TRACE:-}" ]; then set -xv; fi

  local version img name
  version="${VERSION:-stable}"
  img="${IMG:-systeminit/postgres}"
  name="${CONTAINER_NAME:-postgres}"

  : "${POSTGRES_PASSWORD=bugbear}"
  : "${PGPASSWORD=bugbear}"
  : "${POSTGRES_USER=si}"
  : "${POSTGRES_DB=si}"
  : "${POSTGRES_MULTIPLE_DBS=si_test,si_test_dal,si_test_sdf_server,si_auth,si_auth_test,si_module_index}"

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
      --env "POSTGRES_PASSWORD=$POSTGRES_PASSWORD" \
      --env "PGPASSWORD=$PGPASSWORD" \
      --env "POSTGRES_USER=$POSTGRES_USER" \
      --env "POSTGRES_DB=$POSTGRES_DB" \
      --env "POSTGRES_MULTIPLE_DBS=$POSTGRES_MULTIPLE_DBS" \
      --publish 5432:5432 \
      --name "$name" \
      "$@" \
      "$img:$version" \
      -c log_error_verbosity=VERBOSE \
      -c log_statement=all \
      -c shared_buffers=4GB \
      -c wal_buffers=64MB \
      -c effective_cache_size=32GB
  fi
}

main "$@" || exit 1
