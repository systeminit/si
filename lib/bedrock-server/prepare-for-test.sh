#!/bin/bash

set -eo pipefail

RESTORE_POINT=$1 # e.g. 2025-05-17T08-40-38Z

echo "This script is brutal to your local nats and postgres stack, cancel in 5s if you aren't sure you want to do this"
#sleep 5

echo "Purging NATS Rebaser Streams"
nats --server 0.0.0.0 stream purge REBASER_REQUESTS --force
nats --server 0.0.0.0 stream purge REBASER_TASKS --force
nats --server 0.0.0.0 stream purge REBASER_REQUESTS_AUDIT --force

echo "Restoring to a Snapshot of PG DB in preparation for a test"

echo "Restarting Docker Container for Postgres"
docker stop dev-db-1
docker start dev-db-1

sleep 5

BASE_DIR='src/profiles/rebaser/datasources/'
DATABASE_SNAPSHOT_SUBDIR='database_restore_points/measure_rebase'
NATS_SEQUENECE_SUBDIR='nats_sequences/measure_rebase'

SNAPSHOT_DIR="$BASE_DIR/$RESTORE_POINT/$DATABASE_SNAPSHOT_SUBDIR"
SEQUENCE_DIR="$BASE_DIR/$RESTORE_POINT/$NATS_SEQUENECE_SUBDIR"

# For si_layer_db database

PGPASSWORD=bugbear psql -h 0.0.0.0 -p 7432 -U si -d postgres <<EOF
SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = 'si_layer_db' AND pid <> pg_backend_pid();
DROP DATABASE IF EXISTS si_layer_db;
CREATE DATABASE si_layer_db;
EOF

PGPASSWORD=bugbear psql -h 0.0.0.0 -p 7432 -U si -d si_layer_db -f $SNAPSHOT_DIR/si_layer_db_backup.sql

# For si database

PGPASSWORD=bugbear psql -h 0.0.0.0 -p 7432 -U si -d postgres <<EOF
SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = 'si' AND pid <> pg_backend_pid();
DROP DATABASE IF EXISTS si;
CREATE DATABASE si;
EOF

PGPASSWORD=bugbear psql -h 0.0.0.0 -p 7432 -U si -d si -f $SNAPSHOT_DIR/si_db_backup.sql

echo "Stack is ready to receive a test"