#!/bin/bash

set -euo pipefail

SB_ID="${1:-0}" # Default to sb_id=0
JAILER_BINARY="/usr/bin/jailer"
JAILER_NS="jailer-$SB_ID"
JAIL="/srv/jailer/firecracker/$SB_ID/root"

if ! test -f "$JAIL/rootfs.ext4"; then
  echo "Files missing from $JAIL. Has the machine configuration been completed?"
else
  echo "Starting jailer $SB_ID..."
  # TODO(johnrwatson): We don't use proper cgroup isolation, we probably want this in the future
  "${JAILER_BINARY}" \
    --cgroup-version 2 \
    --id $SB_ID \
    --exec-file \
    /usr/bin/firecracker \
    --uid 10000$SB_ID \
    --gid 10000 \
    --netns /var/run/netns/$JAILER_NS \
    --new-pid-ns \
    -- \
    --config-file ./firecracker.conf
fi
