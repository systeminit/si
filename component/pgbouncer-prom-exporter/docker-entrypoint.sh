#!/bin/sh
eval $(cat /etc/pgbouncer-prom-exporter/pgbouncer-conn-string-exporter.sh)
/bin/pgbouncer_exporter