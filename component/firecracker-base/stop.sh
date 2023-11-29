#!/bin/bash

set -euo pipefail

SB_ID="${1:-null}"

JAILER_BINARY="/usr/bin/jailer"
TAP_DEV="fc-${SB_ID}-tap0"

# Kill the firecracker process
ps aux | grep "firecracke[r] --id $SB_ID" | awk '{ print $2 }' | xargs kill -9 || true

# Remove TAP device
ip link del "$TAP_DEV" 2> /dev/null || true

# Remove veth devices
ip link del veth-main$SB_ID 2> /dev/null || true
ip link del veth-jailer$SB_ID 2> /dev/null || true

# Remove iptables rules
ip netns exec jailer-$SB_ID iptables -t nat -D POSTROUTING -o veth-jailer$SB_ID -j MASQUERADE
ip netns exec jailer-$SB_ID iptables -D FORWARD -m conntrack --ctstate RELATED,ESTABLISHED -j ACCEPT
ip netns exec jailer-$SB_ID iptables -D FORWARD -i fc-$SB_ID-tap0 -o veth-jailer$SB_ID -j ACCEPT

# Remove network namespace
ip netns del jailer-$SB_ID

# Remove user and group
userdel jailer-$SB_ID

# Remove directories and files
rm -rf /srv/jailer/firecracker/$SB_ID