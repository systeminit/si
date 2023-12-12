#!/bin/bash

POOL_SIZE=${1:-100}
NATS=${2:-nats.si-tools-prod.systeminit.info:4222}

wget https://artifacts.systeminit.com/veritech/stable/omnibus/linux/$(arch)/veritech-stable-omnibus-linux-$(arch).tar.gz -O - | tar -xzvf - -C /

cat << EOF > /etc/systemd/system/veritech.service

[Unit]
Description=Veritech Server
After=network.target

[Service]
ExecStart=/usr/local/bin/veritech --cyclone-local-firecracker --cyclone-pool-size $POOL_SIZE --nats-url $NATS
Type=exec
Restart=always

[Install]
WantedBy=default.target
RequiredBy=network.target
EOF

systemctl enable –now veritech
