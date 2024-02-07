#!/bin/bash

set -euo pipefail

SB_ID="${1:-null}"

# Kill the firecracker process if it exists
FIRECRACKER_PID=$(pgrep -f "firecracker --id $SB_ID") || true
if [ -n "${FIRECRACKER_PID}" ]; then
  kill -9 $FIRECRACKER_PID
fi

# Remove directories and files
DISK="/srv/jailer/firecracker/$SB_ID/root/rootfs.ext4"
OVERLAY="rootfs-overlay-$SB_ID"

until ! $(mount | grep -q $DISK); do
  umount -dl "$DISK"
done

until ! dmsetup status "$OVERLAY" &> /dev/null; do
  dmsetup remove --force --retry $OVERLAY
done

# We are not currently creating these.
# umount /srv/jailer/firecracker/$SB_ID/root/image-kernel.bin || true
# dmsetup remove kernel-overlay-$SB_ID || true

# TODO(scott): figure out a better way to do this.
# this will only detach devices removed from device-mapper
# but it still feels bad
losetup --detach-all
rm -rf /srv/jailer/firecracker/$SB_ID
