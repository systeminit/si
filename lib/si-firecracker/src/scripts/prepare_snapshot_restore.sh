#!/bin/bash
# Prepare a jail for snapshot restore
# This is a faster alternative to prepare_jailer.sh that skips cold boot
#
# Usage: prepare_snapshot_restore.sh <jail_id>

set -euo pipefail

function retry() {
  n=0
  until [ "$n" -ge 30 ]; do
    $1 && break
    n=$((n + 1))
    sleep 1
  done
}

########## Variables ##########

SB_ID="${1:-0}"

DATA_DIR="/firecracker-data"
JAILER_DIR="/srv/jailer/firecracker"
SNAPSHOT_DIR="$DATA_DIR/golden-snapshot"

ROOTFS="rootfs.ext4"
KERNEL="image-kernel.bin"

TAP_DEV="fc-${SB_ID}-tap0"
FC_MAC="$(printf '02:FC:00:00:%02X:%02X' $((SB_ID / 256)) $((SB_ID % 256)))"
JAILER_NS="jailer-$SB_ID"

# Fixed size: 10GB = 20971520 sectors
LOOP_SZ=20971520

########## User Prep ##########

function user_prep() {
  useradd -M -u 500$SB_ID $JAILER_NS
  usermod -L $JAILER_NS
  usermod -a -G jailer-processes $JAILER_NS
  usermod -a -G root $JAILER_NS
  usermod -a -G kvm $JAILER_NS
}

if ! id 500$SB_ID >/dev/null 2>&1; then
  retry user_prep
fi

########## Disk Prep ##########

JAIL="$JAILER_DIR/$SB_ID/root"
mkdir -p "$JAIL/"
rm -rf "$JAIL/{dev,run}"

touch "$JAIL/logs"
touch "$JAIL/metrics"

# Create /tmp for vsock socket (snapshot stores /tmp/fc-vsock-test.sock path)
mkdir -p "$JAIL/tmp"

########## Overlay Prep ##########

OVERLAY="rootfs-overlay-$SB_ID"

function overlay_prep() {
  # Create overlay file (10GB sparse file for CoW)
  truncate -s 10737418240 "$JAIL/overlay-file"

  LOOP=$(losetup --find --show "$JAIL/overlay-file")

  # Store loop device for cleanup
  echo "$LOOP" >"$JAIL/loop-device"

  # Create dm-snapshot using the golden base
  echo "0 $LOOP_SZ snapshot /dev/mapper/rootfs-overlay-golden $LOOP P 8" | dmsetup create $OVERLAY

  touch "$JAIL/$ROOTFS"
  mount --bind /dev/mapper/$OVERLAY "$JAIL/$ROOTFS"
}

if ! dmsetup info $OVERLAY &>/dev/null; then
  retry overlay_prep
fi

########## Snapshot Files ##########

# Hardlink snapshot files instead of copying - massive I/O savings!
# Firecracker uses MAP_PRIVATE for memory, so each VM gets copy-on-write in RAM.
# The files are never modified on disk, so hardlinks are safe and efficient.
# Hardlinks work inside chroot (unlike symlinks) because they share the inode.
ln -f "$SNAPSHOT_DIR/vmstate" "$JAIL/vmstate" 2>/dev/null || ln "$SNAPSHOT_DIR/vmstate" "$JAIL/vmstate"
ln -f "$SNAPSHOT_DIR/memory" "$JAIL/memory" 2>/dev/null || ln "$SNAPSHOT_DIR/memory" "$JAIL/memory"
ln -f "$SNAPSHOT_DIR/$KERNEL" "$JAIL/$KERNEL" 2>/dev/null || ln "$SNAPSHOT_DIR/$KERNEL" "$JAIL/$KERNEL"

# The snapshot stores absolute paths to drives. Create the directory structure
# inside the jail so the paths resolve correctly after chroot.
GOLDEN_PATH_IN_JAIL="$JAIL/firecracker-data/golden-snapshot"
mkdir -p "$GOLDEN_PATH_IN_JAIL"
# Symlink to the rootfs in jail root (avoids second bind mount that complicates cleanup)
ln -sf /rootfs.ext4 "$GOLDEN_PATH_IN_JAIL/$ROOTFS"
ln -f "$SNAPSHOT_DIR/$KERNEL" "$GOLDEN_PATH_IN_JAIL/$KERNEL" 2>/dev/null || true

########## Chown ##########

chown -R jailer-$SB_ID:jailer-$SB_ID "$JAIL/"

########## Network Prep ##########

if ! test -f /run/netns/$JAILER_NS; then
  ip netns add $JAILER_NS

  MASK_LONG="255.255.255.252"
  MASK_SHORT="/30"
  FC_IP="10.0.0.1"
  TAP_IP="10.0.0.2"
  NET_LINK_MAIN_IP="$(printf '100.65.%s.%s' $(((4 * SB_ID + 1) / 256)) $(((4 * SB_ID + 1) % 256)))"
  NET_LINK_JAILER_IP="$(printf '100.65.%s.%s' $(((4 * SB_ID + 2) / 256)) $(((4 * SB_ID + 2) % 256)))"
  VETH_DEV="veth-jailer$SB_ID"

  ip netns exec $JAILER_NS ip link del "$TAP_DEV" 2>/dev/null || true
  ip netns exec $JAILER_NS ip tuntap add dev "$TAP_DEV" mode tap

  ip netns exec $JAILER_NS sysctl -w net.ipv4.conf.${TAP_DEV}.proxy_arp=1 >/dev/null
  ip netns exec $JAILER_NS sysctl -w net.ipv6.conf.${TAP_DEV}.disable_ipv6=1 >/dev/null

  ip netns exec $JAILER_NS ip addr add "${TAP_IP}${MASK_SHORT}" dev "$TAP_DEV"
  ip netns exec $JAILER_NS ip link set dev "$TAP_DEV" up

  ip link add veth-main$SB_ID type veth peer name $VETH_DEV
  ip link set $VETH_DEV netns $JAILER_NS
  ip addr add $NET_LINK_MAIN_IP/30 dev veth-main$SB_ID
  ip netns exec $JAILER_NS ip addr add $NET_LINK_JAILER_IP/30 dev $VETH_DEV

  ip link set dev veth-main$SB_ID up
  ip netns exec $JAILER_NS ip link set dev $VETH_DEV up
  ip netns exec $JAILER_NS ip route replace default via $NET_LINK_MAIN_IP

  ip netns exec $JAILER_NS iptables -t nat -A POSTROUTING -o $VETH_DEV -j MASQUERADE
  ip netns exec $JAILER_NS iptables -A FORWARD -m conntrack --ctstate RELATED,ESTABLISHED -j ACCEPT
  ip netns exec $JAILER_NS iptables -A FORWARD -i $TAP_DEV -o $VETH_DEV -j ACCEPT
  ip netns exec $JAILER_NS iptables -A OUTPUT -d 169.254.169.254 -j DROP
fi
