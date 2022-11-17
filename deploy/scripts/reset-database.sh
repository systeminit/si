#!/usr/bin/env bash
apt update
apt install -y postgresql-client
# TODO(victor): At some point we need to start managing the db credentials as secrets
export PGPASSWORD="bugbear"
psql -U si -d si -h postgres -c " DROP SCHEMA public CASCADE; CREATE SCHEMA public;"