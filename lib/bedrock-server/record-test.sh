#!/bin/bash

set -eo pipefail

echo "This script is brutal to your local stack, cancel in 5s if you aren't sure you want to do this"
#sleep 5

echo "Purging NATS Rebaser Streams"
nats --server 0.0.0.0 stream purge REBASER_REQUESTS --force
nats --server 0.0.0.0 stream purge REBASER_TASKS --force
nats --server 0.0.0.0 stream purge REBASER_REQUESTS_AUDIT --force

echo "Restarting Docker Container for Postgres"
docker stop dev-db-1
docker start dev-db-1

echo "Creating a Snapshot of PG DB as a new Restore Point for start of test"

# Use ISO format (e.g., 2025-05-16T12-30-45Z) for UTC datetime
DATETIME=$(date -u +"%Y-%m-%dT%H-%M-%SZ")
BASE_DIR='src/profiles/rebaser/datasources'
DATABASE_SNAPSHOT_SUBDIR='database_restore_points/measure_rebase'
NATS_SEQUENECE_SUBDIR='nats_sequences/measure_rebase'

SNAPSHOT_DIR="$BASE_DIR/$DATETIME/$DATABASE_SNAPSHOT_SUBDIR"
SEQUENCE_DIR="$BASE_DIR/$DATETIME/$NATS_SEQUENECE_SUBDIR"

# Create the new directory
mkdir -p "$SNAPSHOT_DIR" "$SEQUENCE_DIR"

echo "Snapshot directories created at $SNAPSHOT_DIR and $SEQUENCE_DIR"

sleep 5

# Dump the databases into SQL restore points
PGPASSWORD=bugbear pg_dump -h 0.0.0.0 -p 7432 -U si -d si_layer_db -f "$SNAPSHOT_DIR/si_layer_db_backup.sql"
PGPASSWORD=bugbear pg_dump -h 0.0.0.0 -p 7432 -U si -d si -f "$SNAPSHOT_DIR/si_db_backup.sql"

echo "Database snapshots saved in $SNAPSHOT_DIR"

# Start to Watch the Stream
echo "Starting to watch the rebaser audit stream for 120s"
echo "START THE TEST YOU WISH TO RECORD"
nats --server 0.0.0.0 consumer next REBASER_REQUESTS_AUDIT my-observer --count 10 --timeout 120s > recorded_messages.txt

# Converting Recorded messages into ./sequence.json, please place this in the relevant test location
python3 convert_recorded_messages_to_json.py # outputs sequence.json

# Move the associated sequence and raw messages beside the snapshots for clean grouping
mv sequence.json $SEQUENCE_DIR/sequence.json
mv recorded_messages.txt $SEQUENCE_DIR/recorded_messages.txt

echo "Test Recording can be found in $DATETIME recording folder, nice job."