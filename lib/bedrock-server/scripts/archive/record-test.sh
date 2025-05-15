#!/bin/bash

set -eo pipefail

echo "This script is brutal to your local stack, cancel in 5s if you aren't sure you want to do this"
#sleep 5

echo "Purging NATS Streams"
nats --server 0.0.0.0 stream purge REBASER_REQUESTS --force || echo "hasn't been configured"
nats --server 0.0.0.0 stream purge REBASER_TASKS --force || echo "hasn't been configured"
nats --server 0.0.0.0 stream purge VERITECH_REQUESTS --force || echo "hasn't been configured"
nats --server 0.0.0.0 stream purge PINGA_JOBS --force || echo "hasn't been configured"
nats --server 0.0.0.0 stream purge LAYERDB_EVENTS --force || echo "hasn't been configured"
nats --server 0.0.0.0 stream purge AUDIT_LOGS --force || echo "hasn't been configured"
nats --server 0.0.0.0 stream purge DEAD_LETTER_QUEUES --force || echo "hasn't been configured" 
nats --server 0.0.0.0 stream purge PENDING_EVENTS --force || echo "hasn't been configured"

echo "Purging Audit Streams"
nats --server 0.0.0.0 stream purge REBASER_REQUESTS_AUDIT --force || echo "hasn't been configured"

echo "Creating mirror streams for rebaser"
nats --server 0.0.0.0 stream add REBASER_REQUESTS_AUDIT \
  --mirror REBASER_REQUESTS \
  --description "Passive copy of REBASER_REQUESTS for recording" \
  --storage file \
  --retention limits \
  --defaults \
  --max-msgs 1000 \
  --discard old \
  --dupe-window 2m \
  --no-deny-delete \
  --no-deny-purge || echo "Already exists"
#nats --server 0.0.0.0 consumer add REBASER_REQUESTS_AUDIT my-observer --deliver all --ack none --replay instant --pull --filter='' --no-headers-only || echo "no configuration to do"

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
echo "Watching the rebaser audit stream until a 60s quiet period occurs"
echo "START THE TEST YOU WISH TO RECORD"
nats --server 0.0.0.0 consumer next REBASER_REQUESTS_AUDIT my-observer --count 1000 --timeout 60s > recorded_messages.txt 2>/dev/null || true

python3 convert_recorded_messages_to_json.py

mv sequence.json "$SEQUENCE_DIR/sequence.json"
mv recorded_messages.txt "$SEQUENCE_DIR/recorded_messages.txt"
