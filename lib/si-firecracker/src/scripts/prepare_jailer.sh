#!/bin/bash

set -euo pipefail

# retry to passed command every 5 seconds for up to 30 seconds
function retry() {
  n=0
  until [ "$n" -ge 30 ]; do
    $1 && break
    n=$((n + 1))
    sleep 1
  done
}

########## ############################# #########
##########              Vars             #########
########## ############################# #########

SB_ID="${1:-0}" # Default to sb_id=0

DATA_DIR="/firecracker-data"
JAILER_DIR="/srv/jailer/firecracker"
JAILER_BINARY="/usr/bin/jailer"

ROOTFS="rootfs.ext4"
KERNEL="image-kernel.bin"
SCRIPTS="scripts"

RO_DRIVE="$DATA_DIR/$ROOTFS"
KERNEL_IMG="$DATA_DIR/$KERNEL"

TAP_DEV="fc-${SB_ID}-tap0"
FC_MAC="$(printf '02:FC:00:00:%02X:%02X' $((SB_ID / 256)) $((SB_ID % 256)))"
JAILER_NS="jailer-$SB_ID"

########## ############################# #########
##########           User Prep           #########
########## ############################# #########

# Create a user and group to run the execution via for one micro-vm
function user_prep() {
  useradd -M -u 500$SB_ID $JAILER_NS
  usermod -L $JAILER_NS

  # This group was created earlier on the machine provisioning
  usermod -a -G jailer-processes $JAILER_NS
  usermod -a -G root $JAILER_NS
  usermod -a -G kvm $JAILER_NS
}

if ! id 500$SB_ID >/dev/null 2>&1; then
  retry user_prep
fi

########## ############################# #########
##########          Disk Prep            #########
########## ############################# #########

JAIL="$JAILER_DIR/$SB_ID/root"
mkdir -p "$JAIL/"
rm -rf "$JAIL/{dev,run}"

touch $JAIL/logs
touch $JAIL/metrics
[[ -e $DATA_DIR/$SCRIPTS ]] && cp $DATA_DIR/$SCRIPTS $JAIL

function kernel_prep() {
  cp $KERNEL_IMG "$JAIL/$KERNEL"
  # TODO(scott): make this work. First attempt yielded a
  # kernel loader InvalidElfMagicNumber error
  # OVERLAY="kernel-overlay-$SB_ID"
  # OVERLAY_FILE=$JAIL/$OVERLAY
  # touch $OVERLAY_FILE
  # truncate --size=5368709120 $OVERLAY_FILE
  # OVERLAY_LOOP=$(losetup --find --show $OVERLAY_FILE)
  # OVERLAY_SZ=$(blockdev --getsz $OVERLAY_LOOP)
  # echo "0 $OVERLAY_SZ snapshot /dev/mapper/rootfs $OVERLAY_LOOP P 8" | dmsetup create $OVERLAY
  # touch $JAIL/$KERNEL
  # mount --bind /dev/mapper/$OVERLAY $JAIL/$KERNEL
}

function rootfs_prep() {
  # Here we create a device-per-jail to act as a unique
  # CoW layer. These cannot be shared because we are required
  # to bind mount these into the jail dir due to chroot shenanigans.
  # Bind mounted permissions propagate, so jails would conflict.
  OVERLAY_FILE=$JAIL/$OVERLAY
  touch $OVERLAY_FILE
  truncate --size=10737418240 $OVERLAY_FILE
  OVERLAY_LOOP=$(losetup --find --show $OVERLAY_FILE)
  OVERLAY_SZ=$(blockdev --getsz $OVERLAY_LOOP)
  echo "0 $OVERLAY_SZ snapshot /dev/mapper/rootfs $OVERLAY_LOOP P 8" | dmsetup create $OVERLAY
  touch $JAIL/$ROOTFS
  mount --bind /dev/mapper/$OVERLAY $JAIL/$ROOTFS
}

if ! test -f "$JAIL/$KERNEL"; then
  retry kernel_prep
fi

OVERLAY="rootfs-overlay-$SB_ID"

if ! dmsetup info $OVERLAY &>/dev/null; then
  retry rootfs_prep
fi

chown -R jailer-$SB_ID:jailer-$SB_ID $JAIL/

########## ############################# #########
##########          Network Prep         #########
########## ############################# #########

# Create network namespace for jailer incantation
if ! test -f /run/netns/$JAILER_NS; then
  ip netns add $JAILER_NS

  MASK_LONG="255.255.255.252"
  MASK_SHORT="/30"
  FC_IP="10.0.0.1"  # Intentionally hardcoded to make cross-microvm communication
  TAP_IP="10.0.0.2" # more difficult & to simplify rootfs creation/configuration
  NET_LINK_MAIN_IP="$(printf '100.65.%s.%s' $(((4 * SB_ID + 1) / 256)) $(((4 * SB_ID + 1) % 256)))"
  NET_LINK_JAILER_IP="$(printf '100.65.%s.%s' $(((4 * SB_ID + 2) / 256)) $(((4 * SB_ID + 2) % 256)))"
  VETH_DEV="veth-jailer$SB_ID"

  # Setup TAP device that uses proxy ARP
  ip netns exec $JAILER_NS ip link del "$TAP_DEV" 2>/dev/null || true
  ip netns exec $JAILER_NS ip tuntap add dev "$TAP_DEV" mode tap

  # Disable ipv6, enable Proxy ARP
  ip netns exec $JAILER_NS sysctl -w net.ipv4.conf.${TAP_DEV}.proxy_arp=1 >/dev/null
  ip netns exec $JAILER_NS sysctl -w net.ipv6.conf.${TAP_DEV}.disable_ipv6=1 >/dev/null

  # Add IP to TAP for micro-vm
  ip netns exec $JAILER_NS ip addr add "${TAP_IP}${MASK_SHORT}" dev "$TAP_DEV"
  ip netns exec $JAILER_NS ip link set dev "$TAP_DEV" up

  # Set up IP link into default namespace for external routing
  ip link add veth-main$SB_ID type veth peer name $VETH_DEV
  ip link set $VETH_DEV netns $JAILER_NS
  ip addr add $NET_LINK_MAIN_IP/30 dev veth-main$SB_ID
  ip netns exec $JAILER_NS ip addr add $NET_LINK_JAILER_IP/30 dev $VETH_DEV

  # Bring the veth link up for external routing
  ip link set dev veth-main$SB_ID up
  ip netns exec $JAILER_NS ip link set dev $VETH_DEV up
  ip netns exec $JAILER_NS ip route replace default via $NET_LINK_MAIN_IP

  # NAT within the namespace to route return traffic to TAP device of firecracker process for inbound traffic
  ip netns exec $JAILER_NS iptables -t nat -A POSTROUTING -o $VETH_DEV -j MASQUERADE
  ip netns exec $JAILER_NS iptables -A FORWARD -m conntrack --ctstate RELATED,ESTABLISHED -j ACCEPT
  ip netns exec $JAILER_NS iptables -A FORWARD -i $TAP_DEV -o $VETH_DEV -j ACCEPT
  ip netns exec $JAILER_NS iptables -A OUTPUT -d 169.254.169.254 -j DROP

fi

########## ############################# #########
##########        Firecracker Prep       #########
########## ############################# #########

{
  cat <<EOF
{
  "boot-source": {
    "kernel_image_path": "./$KERNEL",
    "boot_args": "panic=1 pci=off nomodules reboot=k tsc=reliable quiet i8042.nokbd i8042.noaux 8250.nr_uarts=0 ipv6.disable=1"
  },
  "drives": [
    {
      "drive_id": "1",
      "is_root_device": true,
      "is_read_only": false,
      "path_on_host": "./rootfs.ext4"
    }
EOF

  if [ -e $JAIL/$SCRIPTS ]; then

    cat <<EOF
    ,{
      "drive_id": "2",
      "is_root_device": false,
      "is_read_only": true,
      "path_on_host": "./scripts"
    }
EOF
  fi

  cat <<EOF
  ],
  "machine-config": {
    "vcpu_count": 1,
    "mem_size_mib": 512
  },
  "network-interfaces": [{
    "iface_id": "1",
    "guest_mac": "$FC_MAC",
    "host_dev_name": "$TAP_DEV"
  }],
  "vsock":{
      "guest_cid": 3,
      "uds_path": "./v.sock"
  },
  "logger": {
      "level": "Debug",
      "log_path": "./logs",
      "show_level": false,
      "show_log_origin": false
  },
  "metrics": {
    "metrics_path": "./metrics"
  }
}
EOF
} >$JAIL/firecracker.conf
