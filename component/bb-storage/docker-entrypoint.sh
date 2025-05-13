#!/bin/sh

mkdir -p /storage/ac/persistent_state /storage/cas/persistent_state
mkdir -p /tmp/ac/ /tmp/cas/
touch /tmp/ac/blocks /tmp/cas/blocks

echo "Starting bb-storage..."
exec "$@"
