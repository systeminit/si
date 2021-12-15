#!/usr/bin/env bash
docker run --rm busybox ip route | awk '/^default via/ { print $3 }'