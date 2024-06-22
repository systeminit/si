#!/bin/bash

set -euo pipefail

SB_ID="${1:-0}"

# Kill the firecracker process if it exists
pkill -f "firecracker --id $SB_ID" || true

# Remove directories and files
JAIL="/srv/jailer/firecracker/${SB_ID}/root"
DISK="${JAIL}/rootfs.ext4"
OVERLAY="rootfs-overlay-${SB_ID}"
OVERLAY_FILE="${JAIL}/rootfs-overlay-${SB_ID}"

# Unmount disk if mounted
while mountpoint -q "$DISK"; do
  umount -dl "$DISK"
done

# Remove device mapper overlay
while dmsetup info "$OVERLAY" &> /dev/null; do
  dmsetup remove --force --retry "$OVERLAY"
done

# Detach loop devices related to the specific SB_ID
# Note the ) at the end to ensure we don't match -1 with -10
if losetup -a | grep "$OVERLAY)" &> /dev/null; then
  losetup -d $(losetup -j "$OVERLAY_FILE" -O NAME | sed -n 2p)
fi
rm -rf "/srv/jailer/firecracker/$SB_ID"
