#!/bin/sh
eval $(cat /app/pgbouncer-conn-string-exporter.sh)
/bin/pgbouncer_exporter