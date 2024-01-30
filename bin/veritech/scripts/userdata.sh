#!/bin/bash

# Detects all ssd devices present on the machine,
function detect_devices {
  local block_devices=$(ls /dev | grep nvme[1-9]*n[0-9]*$)
  local result=""
  for device_name in $block_devices
  do
    device="/dev/$device_name"
    # if it is a block device and not a root volume
    if [[ -b $device ]] && [[ ! "$(file -b -s $device)" == *"boot sector"* ]]
    then
      result+=" $device"
    fi
  done
  echo $result
}

# create RAID array
function create_md_array {
  local devices="$(detect_devices)"
  local dev_count=$(echo $devices | wc -w)

  echo "Total devices: $dev_count"
  echo "Devices found: $devices"

  if $(mdadm --assemble /dev/md0 $devices)
  then
    echo "Found previously created Raid0 array, assembling as /dev/md0"
  else
    echo "Raid0 array not found, creating..."
    mdadm --create --verbose /dev/md0 --level raid0 --force --raid-devices=$dev_count --chunk=64K $devices
  fi

  mdadm --examine --scan
  cat /proc/mdstat
  # see https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/disk-performance.html
  echo $((30*1024)) > /proc/sys/dev/raid/speed_limit_min
}

# create raid array from local volumes
VOLUME="/dev/md0"
create_md_array
mkfs -t xfs -f $VOLUME
mkdir -p /srv/jailer
mount $VOLUME /srv/jailer

###################################################################
################       Application Start           ################
###################################################################

POOL_SIZE=${1:-500}
NATS=${2:-tls://connect.ngs.global}
CYCLONE_ENCRYPTION_KEY_SECRET=${3:-tools-encryption-key}
NATS_CREDS_SECRET=${3:-tools-prod-nats-creds}
HONEYCOMB_API_SECRET=${4:-tools-honeycomb-api-key}

# create a volume for our friend the decryption key
KEY_VOLUME=/firecracker-data/decrypt_key.ext4
KEY_MOUNT=/firecracker-data/key
mkdir -p $KEY_MOUNT
dd if=/dev/zero of="$KEY_VOLUME" bs=1M count=1
mkfs.ext4 -v $KEY_VOLUME
e2label $KEY_VOLUME dkey
mount $KEY_VOLUME $KEY_MOUNT
aws secretsmanager get-secret-value --region us-east-1 --secret-id $CYCLONE_ENCRYPTION_KEY_SECRET | jq -r '.SecretString' > $KEY_MOUNT/decryption.key
chmod 777 $KEY_MOUNT/decryption.key
umount $KEY_VOLUME

wget https://artifacts.systeminit.com/veritech/stable/omnibus/linux/$(arch)/veritech-stable-omnibus-linux-$(arch).tar.gz -O - | tar -xzvf - -C /

# Awkward install of the aws cli
sudo apt update
sudo apt install unzip jq -y
curl "https://awscli.amazonaws.com/awscli-exe-linux-x86_64.zip" -o "awscliv2.zip"
unzip awscliv2.zip
sudo ./aws/install

aws secretsmanager get-secret-value --region us-east-1 --secret-id $NATS_CREDS_SECRET | jq -r '.SecretString' >> /tmp/nats-creds

# Install + run docker with otel on 4317 on the host interface
curl -fsSL get.docker.com | bash

docker run \
 --restart always \
 --env SI_OTEL_COL__CONFIG_PATH=/etc/otelcol/honeycomb-config.yaml \
 --env SI_OTEL_COL__HONEYCOMB_API_KEY=$(aws secretsmanager get-secret-value --region us-east-1 --secret-id $HONEYCOMB_API_SECRET | jq -r '.SecretString') \
 -p 4317:4317 \
 -d systeminit/otelcol:stable

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
