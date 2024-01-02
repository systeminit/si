#!/usr/bin/env bash
set -eu

# We need to propagate SIGINT & SIGTERM to both the docker-entrypoint.sh AND to
# syslogd.
trap 'kill -TERM $(jobs -p); wait' SIGINT SIGTERM

# Postgres is logging to syslog, which is logging to `stdout` so logs also show
# up via `docker logs`.
/bin/busybox syslogd -n -L -O - &

/usr/local/bin/docker-entrypoint.sh \
  "$@" \
  -c config_file=/etc/postgresql/postgresql.conf \
  -c ssl=on \
  -c ssl_cert_file=/var/lib/postgresql/server.crt \
  -c ssl_key_file=/var/lib/postgresql/server.key &

if [ -n "${PGANALYZE:-}" ]; then
  echo '--- pganalyze enabled'
  echo '  - Preparing /etc/pganalyze-collector.conf'
  sed \
    -e "s^@@API_KEY@@^${PGA_API_KEY}^" \
    -e "s^@@DB_PASSWORD@@^${PGA_DB_PASSWORD}^" \
    -e "s^@@DB_HOST@@^${PGA_DB_HOST}^" \
    -e "s^@@SYSTEM_ID@@^${PGA_SYSTEM_ID:-development}^" \
    /etc/pganalyze-collector.conf.sample >/etc/pganalyze-collector.conf
  echo '  - Starting pganalyze-collector'
  su - pganalyze -c '/usr/bin/pganalyze-collector --syslog' &
fi

wait
