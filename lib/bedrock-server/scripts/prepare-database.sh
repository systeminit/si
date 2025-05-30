#!/bin/bash

set -eo pipefail

FQ_SQL_PATH=$1
DATABASE=$2

reset_schema() {
  local DATABASE=$1
  PGPASSWORD=bugbear psql -h 0.0.0.0 -p 5432 -U si -d "$DATABASE" <<EOF
DROP SCHEMA IF EXISTS public CASCADE;
CREATE SCHEMA public;
GRANT ALL ON SCHEMA public TO si;
GRANT ALL ON SCHEMA public TO public;
EOF
}

reset_schema "$DATABASE"

echo "Restoring plugins"
PGPASSWORD=bugbear psql -h 0.0.0.0 -p 5432 -U si -d "$DATABASE" -c "CREATE EXTENSION IF NOT EXISTS pgcrypto WITH SCHEMA public;"
PGPASSWORD=bugbear psql -h 0.0.0.0 -p 5432 -U si -d "$DATABASE" -c "CREATE EXTENSION IF NOT EXISTS pg_trgm WITH SCHEMA public;"

echo "Restoring full schema for $DATABASE..."
PGPASSWORD=bugbear psql -h 0.0.0.0 -p 5432 -U si -d "$DATABASE" -f "$FQ_SQL_PATH"

