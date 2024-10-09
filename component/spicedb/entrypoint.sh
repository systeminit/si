#!/usr/bin/env sh
set -eu

spicedb serve >>/tmp/spicedb.log 2>&1 &
tail -f /tmp/spicedb.log &
sleep 3

zed context set example localhost:50051 hobgoblin --insecure
zed schema write schema.zed
zed schema read
zed validate validation.yaml

wait
