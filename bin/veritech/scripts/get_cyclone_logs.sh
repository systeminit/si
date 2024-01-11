#!/bin/bash

set -euo pipefail

SB_ID="${1:-0}" # Default to sb_id=0
JAIL="/srv/jailer/firecracker/$SB_ID/root"
MOUNT=$(mktemp -d)
LOG_FILE="var/log/cyclone.log"

function cleanup {
  sudo umount $MOUNT
  rm -rf $MOUNT
}

trap cleanup EXIT

sudo mount $JAIL/rootfs.ext4 $MOUNT

if test -f $MOUNT/$LOG_FILE ; then
  less $MOUNT/$LOG_FILE
else
  echo "No logs found for $SB_ID".
fi

