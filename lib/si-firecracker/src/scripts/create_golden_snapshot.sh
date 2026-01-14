#!/bin/bash
# Create a golden snapshot for fast VM restore
# This boots a VM, waits for cyclone to be ready, then creates a snapshot
# Run once after firecracker-setup.sh, before starting the pool
# Usage: sudo create_golden_snapshot.sh

set -e

SNAPSHOT_DIR="/firecracker-data/golden-snapshot"
DATA_DIR="/firecracker-data"
KERNEL="image-kernel.bin"
ROOTFS="rootfs.ext4"
SOCKET="/tmp/fc-golden-snapshot.sock"
# IMPORTANT: The vsock path is stored in the snapshot. When restored inside a jail
# (chrooted to /srv/jailer/firecracker/{id}/root/), firecracker will create the
# vsock at this path RELATIVE to the chroot. Using /v.sock means it will be at
# /srv/jailer/firecracker/{id}/root/v.sock - same as cold boot.
VSOCK="/v.sock"
LOG="/tmp/fc-golden-snapshot.log"

log() { echo "[$(date +%T)] $1"; }
err() { echo "[$(date +%T)] ERROR: $1" >&2; }

cleanup() {
    pkill -f "firecracker --api-sock $SOCKET" 2>/dev/null || true
    rm -f "$SOCKET" "$VSOCK"
}
trap cleanup EXIT

log "Creating golden snapshot..."

# Check that the base rootfs device-mapper exists
if ! dmsetup info rootfs &> /dev/null; then
    err "Device mapper 'rootfs' not found!"
    err "Run: /firecracker-data/firecracker-setup.sh -r first"
    exit 1
fi

mkdir -p "$SNAPSHOT_DIR"
cleanup

# Create a golden overlay for snapshot
OVERLAY="rootfs-overlay-golden"
if dmsetup info $OVERLAY &> /dev/null; then
    log "Removing existing golden overlay..."
    umount "$SNAPSHOT_DIR/$ROOTFS" 2>/dev/null || true
    dmsetup remove $OVERLAY 2>/dev/null || true
    # Also remove the loop device
    LOOP_FILE="$SNAPSHOT_DIR/overlay-file"
    if [[ -f "$LOOP_FILE" ]]; then
        LOOP_DEV=$(losetup -j "$LOOP_FILE" | cut -d: -f1)
        if [[ -n "$LOOP_DEV" ]]; then
            losetup -d "$LOOP_DEV" 2>/dev/null || true
        fi
    fi
fi

OVERLAY_FILE="$SNAPSHOT_DIR/overlay-file"
rm -f "$OVERLAY_FILE"
touch "$OVERLAY_FILE"
truncate --size=10737418240 "$OVERLAY_FILE"
OVERLAY_LOOP=$(losetup --find --show "$OVERLAY_FILE")
OVERLAY_SZ=$(blockdev --getsz $OVERLAY_LOOP)
echo "0 $OVERLAY_SZ snapshot /dev/mapper/rootfs $OVERLAY_LOOP P 8" | dmsetup create $OVERLAY

# Bind mount the overlay to a file path firecracker can use
touch "$SNAPSHOT_DIR/$ROOTFS"
mount --bind /dev/mapper/$OVERLAY "$SNAPSHOT_DIR/$ROOTFS"
log "Created golden rootfs overlay"

# Copy kernel
cp "$DATA_DIR/$KERNEL" "$SNAPSHOT_DIR/$KERNEL"

# Start firecracker
log "Starting Firecracker for cold boot..."
rm -f "$LOG"
firecracker --api-sock "$SOCKET" --log-path "$LOG" --level Debug &
FC_PID=$!
sleep 0.5

if ! kill -0 $FC_PID 2>/dev/null; then
    err "Firecracker failed to start"
    cat "$LOG"
    exit 1
fi

# Configure VM
log "Configuring VM..."

curl -s --unix-socket "$SOCKET" -X PUT 'http://localhost/boot-source' \
    -H 'Content-Type: application/json' \
    -d "{
        \"kernel_image_path\": \"$SNAPSHOT_DIR/$KERNEL\",
        \"boot_args\": \"panic=1 pci=off nomodules reboot=k tsc=reliable quiet i8042.nokbd i8042.noaux 8250.nr_uarts=0 ipv6.disable=1\"
    }"

curl -s --unix-socket "$SOCKET" -X PUT 'http://localhost/drives/rootfs' \
    -H 'Content-Type: application/json' \
    -d "{
        \"drive_id\": \"rootfs\",
        \"path_on_host\": \"$SNAPSHOT_DIR/$ROOTFS\",
        \"is_root_device\": true,
        \"is_read_only\": false
    }"

curl -s --unix-socket "$SOCKET" -X PUT 'http://localhost/machine-config' \
    -H 'Content-Type: application/json' \
    -d '{
        "vcpu_count": 1,
        "mem_size_mib": 512
    }'

# Add vsock - the path will be stored in the snapshot
curl -s --unix-socket "$SOCKET" -X PUT 'http://localhost/vsock' \
    -H 'Content-Type: application/json' \
    -d "{
        \"guest_cid\": 3,
        \"uds_path\": \"$VSOCK\"
    }"

# Start the VM
log "Starting VM (cold boot)..."
START_TIME=$(date +%s%3N)

curl -s --unix-socket "$SOCKET" -X PUT 'http://localhost/actions' \
    -H 'Content-Type: application/json' \
    -d '{"action_type": "InstanceStart"}'

# Wait for cyclone to be ready
# In production this would check vsock port 52, but for now we just wait
log "Waiting for VM to boot and cyclone to initialize..."
sleep 5

BOOT_TIME=$(( $(date +%s%3N) - START_TIME ))
log "VM booted in ${BOOT_TIME}ms"

# Pause the VM
log "Pausing VM..."
curl -s --unix-socket "$SOCKET" -X PATCH 'http://localhost/vm' \
    -H 'Content-Type: application/json' \
    -d '{"state": "Paused"}'

# Create snapshot
log "Creating snapshot..."
SNAP_START=$(date +%s%3N)

SNAP_RESULT=$(curl -s --unix-socket "$SOCKET" -X PUT 'http://localhost/snapshot/create' \
    -H 'Content-Type: application/json' \
    -d "{
        \"snapshot_type\": \"Full\",
        \"snapshot_path\": \"$SNAPSHOT_DIR/vmstate\",
        \"mem_file_path\": \"$SNAPSHOT_DIR/memory\"
    }")

if [[ -n "$SNAP_RESULT" ]] && echo "$SNAP_RESULT" | grep -q "fault_message"; then
    err "Snapshot creation failed: $SNAP_RESULT"
    cat "$LOG"
    exit 1
fi

SNAP_TIME=$(( $(date +%s%3N) - SNAP_START ))
log "Snapshot created in ${SNAP_TIME}ms"

# Show what we created
log "Snapshot files:"
ls -lh "$SNAPSHOT_DIR/"

# Kill the original VM
kill $FC_PID 2>/dev/null || true
wait $FC_PID 2>/dev/null || true
rm -f "$SOCKET"

log "Golden snapshot created successfully!"
log "  vmstate: $SNAPSHOT_DIR/vmstate"
log "  memory:  $SNAPSHOT_DIR/memory"
log "  rootfs:  $SNAPSHOT_DIR/$ROOTFS (overlay)"
