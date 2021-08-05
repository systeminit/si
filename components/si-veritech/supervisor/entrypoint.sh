#!/usr/bin/env sh

/usr/bin/supervisord

if [ "${1:-}" = "--" ]; then
  shift
fi

exec "$@"
