#!/usr/bin/env sh
set -eu

spicedb serve-testing &
sleep 3

zed context set example localhost:50051 hobgoblin --insecure
zed schema write schema.zed
zed schema read
zed validate validation.yaml

wait
