#!/bin/bash

export SI_SERVICE=veritech
export SI_HOSTENV={}
export SI_VERSION=$(aws ssm get-parameter --query "Parameter.Value" --output text --name "$SI_HOSTENV-si-version-$SI_SERVICE")
export INIT_VERSION=$(aws ssm get-parameter --query "Parameter.Value" --output text --name "$SI_HOSTENV-si-version-init")

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

# create raid array from local volumes for veritech
VOLUME="/dev/md0"
create_md_array
mkfs -t xfs -f $VOLUME
mkdir -p /srv/jailer
mount $VOLUME /srv/jailer

# create a volume for our friend the scripts dir
KEY_VOLUME=/firecracker-data/scripts
KEY_MOUNT=/firecracker-data/mnt
mkdir -p $KEY_MOUNT
dd if=/dev/zero of="$KEY_VOLUME" bs=1M count=1
mkfs.ext4 -v $KEY_VOLUME
e2label $KEY_VOLUME scripts
mount $KEY_VOLUME $KEY_MOUNT
aws secretsmanager get-secret-value --region us-east-1 --secret-id ${SI_HOSTENV}-veritech-scripts | jq -r '.SecretString' > $KEY_MOUNT/scripts
umount $KEY_VOLUME

# get build metadata
METADATA=$(curl -Ls https://artifacts.systeminit.com/${SI_SERVICE}/${SI_VERSION}/omnibus/linux/x86_64/${SI_SERVICE}-${SI_VERSION}-omnibus-linux-x86_64.tar.gz.metadata.json)

BRANCH=$(echo $METADATA | jq -r '.branch // empty')
COMMIT=$(echo $METADATA | jq -r '.commit')
VERSION=$(echo $METADATA | jq -r '.version')

# install build
wget https://artifacts.systeminit.com/${SI_SERVICE}/${SI_VERSION}/omnibus/linux/$(arch)/$SI_SERVICE-${SI_VERSION}-omnibus-linux-$(arch).tar.gz -O - | tar -xzvf - -C /

# prep system
mkdir -p /run/app
wget https://raw.githubusercontent.com/systeminit/si/${BRANCH:-main}/component/deploy/docker-compose.yaml -O /run/app/docker-compose.yaml

docker-compose -f /run/app/docker-compose.yaml --profile $SI_SERVICE up --wait

cat << EOF > /etc/systemd/system/$SI_SERVICE.service

[Unit]
Description=$SI_SERVICE
After=network.target

[Service]
ExecStart=/usr/local/bin/$SI_SERVICE

Type=exec
Restart=always

[Install]
WantedBy=default.target
RequiredBy=network.target
EOF

systemctl enable --now $SI_SERVICE

# marker in honeycomb
HONEYCOMB_API_KEY=$(aws secretsmanager get-secret-value --region us-east-1 --secret-id ${SI_HOSTENV}-honeycomb-api-key | jq -r '.SecretString')

curl https://api.honeycomb.io/1/markers/$SI_SERVICE -X POST \
    -H "X-Honeycomb-Team: $HONEYCOMB_API_KEY" \
    -d '{"message":" '"$SI_SERVICE replica deployed! Commit: $COMMIT Version: $VERSION"' ", "type":"deploy"}'
