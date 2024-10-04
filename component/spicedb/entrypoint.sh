#!/usr/bin/env sh
set -eu

spicedb serve >>/tmp/spicedb.log 2>&1 &
tail -f /tmp/spicedb.log &
sleep 3

zed context set example localhost:50051 hobgoblin --insecure
zed schema write schema.zed
zed schema read
zed relationship create document:1 writer user:1
zed permission check document:1 view user:1

wait
