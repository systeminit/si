#!/bin/bash

POOL_SIZE=${1:-1000}
NATS=${2:-tls://connect.ngs.global}

wget https://artifacts.systeminit.com/veritech/stable/omnibus/linux/$(arch)/veritech-stable-omnibus-linux-$(arch).tar.gz -O - | tar -xzvf - -C /

# Awkward install of the aws cli
sudo apt update
sudo apt install unzip jq -y
curl "https://awscli.amazonaws.com/awscli-exe-linux-x86_64.zip" -o "awscliv2.zip"
unzip awscliv2.zip
sudo ./aws/install

aws secretsmanager get-secret-value --region us-east-1 --secret-id si-nats-creds | jq -r '.SecretString' >> /tmp/nats-creds

# Install + run docker with otel on 4317 on the host interface
curl -fsSL get.docker.com | bash

docker run \
 --restart always \
 --env SI_OTEL_COL__CONFIG_PATH=/etc/otelcol/honeycomb-config.yaml \
 --env SI_OTEL_COL__HONEYCOMB_API_KEY=$(aws secretsmanager get-secret-value --region us-east-1 --secret-id si-honeycomb-api-key | jq -r '.SecretString') \
 -p 4317:4317 \
 systeminit/otelcol:stable

cat << EOF > /etc/systemd/system/veritech.service

[Unit]
Description=Veritech Server
After=network.target

[Service]
ExecStart=/usr/local/bin/veritech --cyclone-local-firecracker --cyclone-pool-size $POOL_SIZE --nats-url $NATS --nats-creds-path /tmp/nats-creds --cyclone-connect-timeout 100
Type=exec
Restart=always

[Install]
WantedBy=default.target
RequiredBy=network.target
EOF

systemctl enable --now veritech