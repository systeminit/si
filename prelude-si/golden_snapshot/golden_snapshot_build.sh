#!/bin/bash
# Build a golden snapshot for fast Firecracker VM restore
# This boots a VM, waits for cyclone to be ready, then creates a snapshot
#
# Required: KVM access (/dev/kvm must be accessible)
# Required: firecracker binary in PATH or at /usr/bin/firecracker
#
# Arguments:
#   $1 - git_metadata file (JSON with version info)
#   $2 - platform_arch (aarch64 or x86_64)
#   $3 - build_metadata_out file path
#   $4 - output tarball path
#   $5 - rootfs ext4 image path
#   $6 - kernel image path

set -euo pipefail

git_metadata="$1"
platform_arch="$2"
build_metadata_out="$3"
tarball_out="$4"
rootfs_input="$5"
kernel_input="$6"

echo "-------------------------------------"
echo "Info: Initiating golden snapshot build"
echo "Artifact Version: $(jq -r '.canonical_version' <"$git_metadata")"
echo "Platform: $platform_arch"
echo "Output File: $tarball_out"
echo "Rootfs: $rootfs_input"
echo "Kernel: $kernel_input"
echo "-------------------------------------"

# Check for KVM access
if [[ ! -e /dev/kvm ]]; then
  echo "Error: /dev/kvm not found. KVM is required to create golden snapshots."
  echo "This build must run on a KVM-capable host (bare metal or nested virt enabled)."
  exit 1
fi

if [[ ! -r /dev/kvm ]] || [[ ! -w /dev/kvm ]]; then
  echo "Error: Cannot access /dev/kvm. Check permissions."
  exit 1
fi

# Find firecracker binary
FIRECRACKER=""
for path in /usr/bin/firecracker /usr/local/bin/firecracker firecracker; do
  if command -v "$path" &>/dev/null; then
    FIRECRACKER="$path"
    break
  fi
done

if [[ -z "$FIRECRACKER" ]]; then
  echo "Error: firecracker binary not found in PATH or standard locations"
  exit 1
fi

echo "Info: Using firecracker at: $FIRECRACKER"

BUCKROOT="$BUCK_SCRATCH_PATH"
WORKDIR="$BUCKROOT/golden-snapshot-build"
SNAPSHOT_DIR="$WORKDIR/snapshot"
SOCKET="$WORKDIR/fc.sock"
LOG="$WORKDIR/fc.log"

# IMPORTANT: The vsock path is stored in the snapshot. When restored inside a jail
# (chrooted to /srv/jailer/firecracker/{id}/root/), firecracker will create the
# vsock at this path RELATIVE to the chroot. Using /v.sock means it will be at
# /srv/jailer/firecracker/{id}/root/v.sock - same as cold boot.
VSOCK="/v.sock"

cleanup() {
  set +e
  pkill -f "firecracker --api-sock $SOCKET" 2>/dev/null || true
  rm -f "$SOCKET" "$VSOCK"
  # Don't remove WORKDIR - buck2 manages it
}
trap cleanup EXIT

mkdir -p "$SNAPSHOT_DIR" "$WORKDIR"

# Copy rootfs and kernel to work directory (we'll modify rootfs)
ROOTFS="$WORKDIR/rootfs.ext4"
KERNEL="$WORKDIR/kernel.bin"

cp "$rootfs_input" "$ROOTFS"
cp "$kernel_input" "$KERNEL"

# Make rootfs writable (it may be read-only from buck)
chmod u+w "$ROOTFS"

echo "Info: Starting Firecracker for cold boot..."
rm -f "$LOG" "$SOCKET"
"$FIRECRACKER" --api-sock "$SOCKET" --log-path "$LOG" --level Debug &
FC_PID=$!
sleep 0.5

if ! kill -0 $FC_PID 2>/dev/null; then
  echo "Error: Firecracker failed to start"
  cat "$LOG" 2>/dev/null || true
  exit 1
fi

echo "Info: Configuring VM..."

# Configure boot source
curl -s --unix-socket "$SOCKET" -X PUT 'http://localhost/boot-source' \
  -H 'Content-Type: application/json' \
  -d "{
        \"kernel_image_path\": \"$KERNEL\",
        \"boot_args\": \"panic=1 pci=off nomodules reboot=k tsc=reliable quiet i8042.nokbd i8042.noaux 8250.nr_uarts=0 ipv6.disable=1\"
    }"

# Configure root drive
curl -s --unix-socket "$SOCKET" -X PUT 'http://localhost/drives/rootfs' \
  -H 'Content-Type: application/json' \
  -d "{
        \"drive_id\": \"rootfs\",
        \"path_on_host\": \"$ROOTFS\",
        \"is_root_device\": true,
        \"is_read_only\": false
    }"

# Configure machine
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
echo "Info: Starting VM (cold boot)..."
START_TIME=$(date +%s%3N)

curl -s --unix-socket "$SOCKET" -X PUT 'http://localhost/actions' \
  -H 'Content-Type: application/json' \
  -d '{"action_type": "InstanceStart"}'

# Wait for cyclone to be ready
# The VM boots Alpine, starts OpenRC, which starts the cyclone service
# cyclone binds to vsock port 52
# TODO: Actually probe vsock port 52 for readiness instead of sleeping
echo "Info: Waiting for VM to boot and cyclone to initialize..."
sleep 5

BOOT_TIME=$(($(date +%s%3N) - START_TIME))
echo "Info: VM booted in ${BOOT_TIME}ms"

# Pause the VM before snapshotting
echo "Info: Pausing VM..."
curl -s --unix-socket "$SOCKET" -X PATCH 'http://localhost/vm' \
  -H 'Content-Type: application/json' \
  -d '{"state": "Paused"}'

# Create snapshot
echo "Info: Creating snapshot..."
SNAP_START=$(date +%s%3N)

SNAP_RESULT=$(curl -s --unix-socket "$SOCKET" -X PUT 'http://localhost/snapshot/create' \
  -H 'Content-Type: application/json' \
  -d "{
        \"snapshot_type\": \"Full\",
        \"snapshot_path\": \"$SNAPSHOT_DIR/vmstate\",
        \"mem_file_path\": \"$SNAPSHOT_DIR/memory\"
    }")

if [[ -n "$SNAP_RESULT" ]] && echo "$SNAP_RESULT" | grep -q "fault_message"; then
  echo "Error: Snapshot creation failed: $SNAP_RESULT"
  cat "$LOG" 2>/dev/null || true
  exit 1
fi

SNAP_TIME=$(($(date +%s%3N) - SNAP_START))
echo "Info: Snapshot created in ${SNAP_TIME}ms"

# Kill the original VM
kill $FC_PID 2>/dev/null || true
wait $FC_PID 2>/dev/null || true

# The rootfs now contains the boot-time changes (cyclone initialized, services started)
# Copy it to snapshot dir
cp "$ROOTFS" "$SNAPSHOT_DIR/rootfs.ext4"

# Also include the kernel for completeness (restore needs it)
cp "$KERNEL" "$SNAPSHOT_DIR/kernel.bin"

echo "Info: Snapshot files:"
ls -lh "$SNAPSHOT_DIR/"

# Create tarball
echo "Info: Creating tarball..."
tar -cvf "$tarball_out" -C "$SNAPSHOT_DIR" \
  vmstate \
  memory \
  rootfs.ext4 \
  kernel.bin

echo "Info: Tarball created: $(ls -lh "$tarball_out")"

# Generate build metadata
cat <<EOF >"$build_metadata_out"
{
  "family": "cyclone",
  "variant": "golden-snapshot",
  "version": "$(jq -r '.canonical_version' <"$git_metadata")",
  "arch": "$platform_arch",
  "os": "linux",
  "commit": "$(jq -r '.commit_hash' <"$git_metadata")",
  "b3sum": "$(b3sum --no-names "$tarball_out")"
}
EOF

echo "--- golden snapshot build complete."
