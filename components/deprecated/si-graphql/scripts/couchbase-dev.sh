#!/bin/sh

set -e

echo "==> Starting Couchbase in Docker as 'db'"
docker run -d --name db -p 8091-8096:8091-8096 -p 11210-11211:11210-11211 couchbase:6.5.0-beta

echo "==> Waiting for Couchbase to boot"
sleep 10

echo "==> Initializing Couchbase"
docker exec db /opt/couchbase/bin/couchbase-cli cluster-init -c 127.0.0.1 --cluster-username si --cluster-password bugbear --services data,index,query,fts,analytics --cluster-ramsize 2048 --cluster-index-ramsize 1024 --cluster-eventing-ramsize 1024 --cluster-fts-ramsize 1024 --cluster-analytics-ramsize 1024 --index-storage-setting default

echo "==> Creating bucket 'si'"
docker exec db /opt/couchbase/bin/couchbase-cli bucket-create --cluster 127.0.0.1 --username si --password bugbear --bucket si --bucket-type couchbase --bucket-ramsize 2048 

echo "==> Creating primary index on 'si'"
docker exec db /opt/couchbase/bin/cbq -engine http://localhost:8091 -u si -p bugbear --script "CREATE PRIMARY INDEX ON \`si\`"

