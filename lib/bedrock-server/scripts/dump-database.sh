#!/bin/bash

set -eo pipefail

database=$1
uuid=$2
variant=$3

BASE_DIR='./recordings/rebaser/datasources'
DATABASE_SNAPSHOT_SUBDIR='database_restore_points'

SNAPSHOT_DIR="$BASE_DIR/$uuid/$DATABASE_SNAPSHOT_SUBDIR/$variant"

mkdir -p "$SNAPSHOT_DIR"

PGPASSWORD=bugbear pg_dumpall --globals-only -h 0.0.0.0 -p 5432 -U si > "$SNAPSHOT_DIR/globals.sql"

echo "Dumping full schema of $database as uuid $uuid"
PGPASSWORD=bugbear pg_dump --no-owner --no-privileges --schema=public -h 0.0.0.0 -p 5432 -U si -d "$database" -f "$SNAPSHOT_DIR/${database}_public_schema.sql"
