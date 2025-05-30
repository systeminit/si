#!/bin/bash

set -eo pipefail

RESTORE_POINT=$1 # e.g. 2025-05-17T08-40-38Z

echo "This script is brutal to your local NATS and Postgres stack, cancel in 5s if you aren't sure you want to do this"
docker restart dev-postgres-1
sleep 5

echo "Purging NATS Rebaser Streams"
nats --server 0.0.0.0 stream purge REBASER_REQUESTS --force || echo "hasn't been configured"
nats --server 0.0.0.0 stream purge REBASER_TASKS --force || echo "hasn't been configured"
nats --server 0.0.0.0 stream purge VERITECH_REQUESTS --force || echo "hasn't been configured"
nats --server 0.0.0.0 stream purge PINGA_JOBS --force || echo "hasn't been configured"
nats --server 0.0.0.0 stream purge LAYERDB_EVENTS --force || echo "hasn't been configured"
nats --server 0.0.0.0 stream purge AUDIT_LOGS --force || echo "hasn't been configured"
nats --server 0.0.0.0 stream purge DEAD_LETTER_QUEUES --force || echo "hasn't been configured"
nats --server 0.0.0.0 stream purge PENDING_EVENTS --force || echo "hasn't been configured"

echo "Deleting mirror streams to prevent interference"
nats --server 0.0.0.0 consumer delete REBASER_REQUESTS_AUDIT my-observer --force || echo "hasn't been configured"
nats --server 0.0.0.0 stream delete REBASER_REQUESTS_AUDIT --force || echo "hasn't been configured"

echo "Restoring to a snapshot of PG DB in preparation for a test"

BASE_DIR='src/profiles/rebaser/datasources/'
DATABASE_SNAPSHOT_SUBDIR='database_restore_points/measure_rebase'
NATS_SEQUENECE_SUBDIR='nats_sequences/measure_rebase'

SNAPSHOT_DIR="$BASE_DIR/$RESTORE_POINT/$DATABASE_SNAPSHOT_SUBDIR"
SEQUENCE_DIR="$BASE_DIR/$RESTORE_POINT/$NATS_SEQUENECE_SUBDIR"

# Enable nullglob to avoid errors on unmatched globs
shopt -s nullglob

# Step 1: Restore cluster-wide globals (roles, extensions, etc.)
echo "Restoring global PostgreSQL objects (roles, extensions, etc.)"
PGPASSWORD=bugbear psql -h 0.0.0.0 -p 5432 -U si -d postgres -f "$SNAPSHOT_DIR/globals.sql"

# Step 2: Drop and recreate the public schema for each DB
reset_schema() {
  local DB=$1
  echo "Resetting 'public' schema in $DB..."
  PGPASSWORD=bugbear psql -h 0.0.0.0 -p 5432 -U si -d "$DB" <<EOF
DROP SCHEMA IF EXISTS public CASCADE;
CREATE SCHEMA public;
GRANT ALL ON SCHEMA public TO si;
GRANT ALL ON SCHEMA public TO public;
EOF
}

reset_schema "si_layer_db"
reset_schema "si"

echo "Restoring full schema for si_layer_db..."
PGPASSWORD=bugbear psql -h 0.0.0.0 -p 5432 -U si -d si_layer_db -f "$SNAPSHOT_DIR/si_layer_db_public_schema.sql"

echo "Restoring full schema for si..."
PGPASSWORD=bugbear psql -h 0.0.0.0 -p 5432 -U si -d si -f "$SNAPSHOT_DIR/si_public_schema.sql"

PGPASSWORD=bugbear psql -h 0.0.0.0 -p 5432 -U si -d si_layer_db -c "CREATE EXTENSION IF NOT EXISTS pgcrypto;"
PGPASSWORD=bugbear psql -h 0.0.0.0 -p 5432 -U si -d si -c "CREATE EXTENSION IF NOT EXISTS pgcrypto;"

echo "Stack is ready to receive a test"
