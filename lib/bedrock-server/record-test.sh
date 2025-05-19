#!/bin/bash

set -eo pipefail

echo "This script is brutal to your local stack, cancel in 5s if you aren't sure you want to do this"
#sleep 5

echo "Purging NATS Rebaser Streams"
nats --server 0.0.0.0 stream purge REBASER_REQUESTS --force
nats --server 0.0.0.0 stream purge REBASER_TASKS --force
nats --server 0.0.0.0 stream purge VERITECH_REQUESTS --force
nats --server 0.0.0.0 stream purge PINGA_JOBS --force
nats --server 0.0.0.0 stream purge LAYERDB_EVENTS --force
nats --server 0.0.0.0 stream purge AUDIT_LOGS --force
nats --server 0.0.0.0 stream purge DEAD_LETTER_QUEUES --force
nats --server 0.0.0.0 stream purge PENDING_EVENTS --force
nats --server 0.0.0.0 stream purge REBASER_REQUESTS_AUDIT --force || echo "hasn't been configured"

echo "Creating a Snapshot of PG DB as a new Restore Point for start of test"

DATETIME=$(date -u +"%Y-%m-%dT%H-%M-%SZ")
BASE_DIR='src/profiles/rebaser/datasources'
DATABASE_SNAPSHOT_SUBDIR='database_restore_points/measure_rebase'
NATS_SEQUENECE_SUBDIR='nats_sequences/measure_rebase'

SNAPSHOT_DIR="$BASE_DIR/$DATETIME/$DATABASE_SNAPSHOT_SUBDIR"
SEQUENCE_DIR="$BASE_DIR/$DATETIME/$NATS_SEQUENECE_SUBDIR"

mkdir -p "$SNAPSHOT_DIR" "$SEQUENCE_DIR"

echo "Snapshot directories created at $SNAPSHOT_DIR and $SEQUENCE_DIR"

PGPASSWORD=bugbear pg_dumpall --globals-only -h 0.0.0.0 -p 5432 -U si > "$SNAPSHOT_DIR/globals.sql"

# Dump whole schema per DB
for DB in si_layer_db si; do
  echo "Dumping full schema of $DB"
  PGPASSWORD=bugbear pg_dump --no-owner --no-privileges --schema=public \
  -h 0.0.0.0 -p 5432 -U si -d "$DB" -f "$SNAPSHOT_DIR/${DB}_public_schema.sql"
done

echo "Schema-level snapshots saved"

# Record from NATS
echo "Starting to watch the rebaser audit stream for 120s"
echo "START THE TEST YOU WISH TO RECORD"
nats --server 0.0.0.0 consumer next REBASER_REQUESTS_AUDIT my-observer --count 10 --timeout 120s > recorded_messages.txt

python3 convert_recorded_messages_to_json.py

mv sequence.json "$SEQUENCE_DIR/sequence.json"
mv recorded_messages.txt "$SEQUENCE_DIR/recorded_messages.txt"
