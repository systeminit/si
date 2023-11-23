#!/bin/bash

set -euo pipefail

SB_ID="${1:-0}" # Default to sb_id=0

JAILER_BINARY="/usr/bin/jailer"
FC_BINARY="/usr/bin/jailer"
RO_DRIVE="/firecracker-data/rootfs.ext4"
KERNEL="/firecracker-data/image-kernel.bin"
TAP_DEV="fc-${SB_ID}-tap0"

KERNEL_BOOT_ARGS="panic=1 pci=off nomodules reboot=k tsc=reliable quiet i8042.nokbd i8042.noaux 8250.nr_uarts=0 ipv6.disable=1"
#KERNEL_BOOT_ARGS="console=ttyS0 reboot=k panic=1 pci=off nomodules i8042.nokbd i8042.noaux ipv6.disable=1"

API_SOCKET="/srv/jailer/firecracker/$SB_ID/root/run/firecracker.socket"
CURL=(curl --silent --show-error --header "Content-Type: application/json" --unix-socket "${API_SOCKET}" --write-out "HTTP %{http_code}")

# Create a user and group to run the execution via for one micro-vm
# TODO(johnrwatson): There is a edge case where this will clash with an already existing user + will fail
# root@ip-10-1-29-58:/firecracker-data# useradd -M -u 10000$SB_ID jailer-$SB_ID
# useradd: user 'jailer-20359' already exists
useradd -M -u 10000$SB_ID jailer-$SB_ID
usermod -L jailer-$SB_ID

# This was created earlier on the machine provisioning
# groupadd -g 10000 jailer-processes
usermod -a -G jailer-processes jailer-$SB_ID
usermod -a -G root jailer-$SB_ID
usermod -a -G kvm jailer-$SB_ID

curl_put() {
    local URL_PATH="$1"
    local OUTPUT RC
    OUTPUT="$("${CURL[@]}" -X PUT --data @- "http://localhost/${URL_PATH#/}" 2>&1)"
    RC="$?"
    if [ "$RC" -ne 0 ]; then
        echo "Error: curl PUT ${URL_PATH} failed with exit code $RC, output:"
        echo "$OUTPUT"
        return 1
    fi
    # Error if output doesn't end with "HTTP 2xx"
    if [[ "$OUTPUT" != *HTTP\ 2[0-9][0-9] ]]; then
        echo "Error: curl PUT ${URL_PATH} failed with non-2xx HTTP status code, output:"
        echo "$OUTPUT"
        return 1
    fi
}

logfile="/srv/jailer/firecracker/$SB_ID/root/fc-sb${SB_ID}-log"
metricsfile="/srv/jailer/firecracker/$SB_ID/root/fc-sb${SB_ID}-metrics"

mkdir -p /srv/jailer/firecracker/$SB_ID/root/
touch "$logfile"
touch "$metricsfile"

# Simlink in the rootfs and kernel files
# TODO(johnrwatson): Figure out how to avoid this, it's a total waste of resources
cp $KERNEL /srv/jailer/firecracker/$SB_ID/root/image-kernel.bin
cp $RO_DRIVE /srv/jailer/firecracker/$SB_ID/root/rootfs.ext4

chown -R jailer-$SB_ID:jailer-$SB_ID /srv/jailer/firecracker/$SB_ID/root/

# Create network namespace for jailer incantation
ip netns add jailer-$SB_ID

# Setup TAP device that uses proxy ARP
MASK_LONG="255.255.255.252"
MASK_SHORT="/30"
FC_IP="10.0.0.1"  # Intentionally hardcoded to make cross-microvm communication 
TAP_IP="10.0.0.2" # more difficult & to simplify rootfs creation/configuration
FC_MAC="$(printf '02:FC:00:00:%02X:%02X' $((SB_ID / 256)) $((SB_ID % 256)))"
ip netns exec jailer-$SB_ID ip link del "$TAP_DEV" 2> /dev/null || true
ip netns exec jailer-$SB_ID ip tuntap add dev "$TAP_DEV" mode tap

# Disable ipv6, enable Proxy ARP
ip netns exec jailer-$SB_ID sysctl -w net.ipv4.conf.${TAP_DEV}.proxy_arp=1 > /dev/null
ip netns exec jailer-$SB_ID sysctl -w net.ipv6.conf.${TAP_DEV}.disable_ipv6=1 > /dev/null

# Add IP to TAP for micro-vm
ip netns exec jailer-$SB_ID ip addr add "${TAP_IP}${MASK_SHORT}" dev "$TAP_DEV"
ip netns exec jailer-$SB_ID ip link set dev "$TAP_DEV" up

NET_LINK_MAIN_IP="$(printf '100.65.%s.%s' $(((4 * SB_ID + 1) / 256)) $(((4 * SB_ID + 1) % 256)))"
NET_LINK_JAILER_IP="$(printf '100.65.%s.%s' $(((4 * SB_ID + 2) / 256)) $(((4 * SB_ID + 2) % 256)))"

# Set up IP link into default namespace for external routing
ip link add veth-main$1 type veth peer name veth-jailer$1
ip link set veth-jailer$1 netns jailer-$1
ip addr add $NET_LINK_MAIN_IP/30 dev veth-main$1
ip netns exec jailer-$1 ip addr add $NET_LINK_JAILER_IP/30 dev veth-jailer$1

# Bring the veth link up for external routing
ip link set dev veth-main$1 up
ip netns exec jailer-$1 ip link set dev veth-jailer$1 up
ip netns exec jailer-$1 ip route add default via $NET_LINK_MAIN_IP
ip netns exec jailer-$1 ip route

# NAT within the namespace to route return traffic to TAP device of firecracker process for inbound traffic
ip netns exec jailer-$1 iptables -t nat -A POSTROUTING -o veth-jailer$1 -j MASQUERADE
ip netns exec jailer-$1 iptables -A FORWARD -m conntrack --ctstate RELATED,ESTABLISHED -j ACCEPT
ip netns exec jailer-$1 iptables -A FORWARD -i fc-$1-tap0 -o veth-jailer$1 -j ACCEPT

KERNEL_BOOT_ARGS="${KERNEL_BOOT_ARGS} ip=${FC_IP}::${TAP_IP}:${MASK_LONG}::eth0:off"

# Start Firecracker API server
rm -f "$API_SOCKET" || echo "socket missing"

# Example FC invocation directly
#"${FC_BINARY}" --api-sock "$API_SOCKET" --id "${SB_ID}" --boot-timer >> "$logfile" &

# TODO(johnrwatson): We don't use proper cgroup isolation, we probably want this in the future
"${JAILER_BINARY}" --cgroup-version 2 --id $SB_ID --exec-file /usr/bin/firecracker --uid 10000$SB_ID --gid 10000 --netns /var/run/netns/jailer-$SB_ID --new-pid-ns -- --boot-timer >> "$logfile" &

sleep 0.015s

# Wait for API server to start
while [ ! -e "$API_SOCKET" ]; do
    echo "FC $SB_ID still not ready..."
    sleep 0.01s
done

curl_put '/logger' <<EOF
{
  "level": "Debug",
  "log_path": "fc-sb${SB_ID}-log",
  "show_level": false,
  "show_log_origin": false
}
EOF

# TODO(johnrwatson): We don't get metrics at the minute, probably want it in the future
#curl_put '/metrics' <<EOF
#{
#  "metrics_path": "$metricsfile"
#}
#EOF

curl_put '/machine-config' <<EOF
{
  "vcpu_count": 1,
  "mem_size_mib": 512
}
EOF

curl_put '/boot-source' <<EOF
{
  "kernel_image_path": "image-kernel.bin",
  "boot_args": "$KERNEL_BOOT_ARGS"
}
EOF

curl_put '/drives/1' <<EOF
{
  "drive_id": "1",
  "path_on_host": "rootfs.ext4",
  "is_root_device": true,
  "is_read_only": false
}
EOF

curl_put '/network-interfaces/1' <<EOF
{
  "iface_id": "1",
  "guest_mac": "$FC_MAC",
  "host_dev_name": "$TAP_DEV"
}
EOF

curl_put '/vsock' <<EOF
{
      "guest_cid": 3,
      "uds_path": "./v.sock"
  }
EOF

curl_put '/actions' <<EOF
{
  "action_type": "InstanceStart"
}
EOF