#!/usr/bin/env bash

if [ -z "$(pg_isready --version 2>/dev/null)" ]; then
  # pg_isready isn't always available. :(
  echo -n "'pg_isready' not available; sleeping to give PostgresQL time to start..."
  sleep 10
  echo "DONE"
else
  echo -n "Trying PG"
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
