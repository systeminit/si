#!/bin/bash

# TODO(johnrwatson): In theory we should be able to run this task for any of the components we need rootfs' for 
# but there are some cyclone-specifics bits that will cause us problems

set -eo pipefail

# TODO(johnrwatson): We need to port this to python or similar, and check for OS-dependencies that are required. i.e.
# docker and basic priviledged escalation for the mounts

git_helper=$1   # i.e. ./${git_helper} | jq -r '.abbreviated_commit_hash' (it returns a json blob output via python)
output_file=$2  # i.e. output the file ./johns_rootfs.tar

# Shift the parsed arguments off after assignment
shift 2

# The rest of the inputs are a list of input files or directories, to also include in the build
# i.e. consume a binary ./johns_binary.bin for use within this script
binary_inputs=("$@")

echo "-------------------------------------"
echo "Info: Initiating rootfs build"
echo "Artifact Version: $(./${git_helper} | jq -r '.artifact_ver')"
echo "Output File: $output_file"
echo "Input Binaries (list):"
for binary_input in "${binary_inputs[@]}"; do
  echo "$(echo $binary_input | awk -F "/" '{print $NF}') Full Path: $binary_input"
done
echo "-------------------------------------"

GITROOT="$(pwd)"
BUCKROOT="$BUCK_SCRATCH_PATH" # This is provided by buck2

PACKAGEDIR="$BUCKROOT/cyclone-pkg"
ROOTFS="$PACKAGEDIR/cyclone-rootfs.ext4"
ROOTFSMOUNT="$PACKAGEDIR/rootfs"
GUESTDISK="/rootfs"
INITSCRIPT="$PACKAGEDIR/init.sh"

# create disk and mount to a known location
mkdir -p $ROOTFSMOUNT
dd if=/dev/zero of=$ROOTFS bs=1M count=1024
mkfs.ext4 $ROOTFS
sudo mount $ROOTFS $ROOTFSMOUNT

# create our script to add an init system to our container image
cat << EOL > $INITSCRIPT
apk update
apk add openrc openssh

apk add --no-cache runuser; \
adduser -D app; \
for dir in /run /etc /usr/local/etc /home/app/.config; do \
    mkdir -pv "$dir/$BIN"; \
done;

ssh-keygen -A

# Make sure special file systems are mounted on boot:
rc-update add devfs boot
rc-update add procfs boot
rc-update add sysfs boot
rc-update add local default
rc-update add networking boot
rc-update add sshd

# Then, copy the newly configured system to the rootfs image:
for d in bin dev etc lib root sbin usr nix; do tar c "/\${d}" | tar x -C ${GUESTDISK}; done
for dir in proc run sys var; do mkdir ${GUESTDISK}/\${dir}; done

# autostart cyclone
cat << EOF > ${GUESTDISK}/etc/init.d/cyclone
#!/sbin/openrc-run

name="cyclone"
description="Cyclone"
supervisor="supervise-daemon"
command="cyclone"
command_args="--bind-vsock 3:52 --decryption-key /dev.decryption.key --lang-server /usr/local/bin/lang-js --enable-watch --limit-requests 1 --watch-timeout 10 --enable-ping --enable-resolver --enable-action-run"
pidfile="/run/agent.pid"
EOF

chmod +x ${GUESTDISK}/usr/local/bin/cyclone
chmod +x ${GUESTDISK}/usr/local/bin/lang-js
chmod +x ${GUESTDISK}/etc/init.d/cyclone

chroot ${GUESTDISK} rc-update add cyclone boot

# networking bits
echo "nameserver 8.8.8.8" > ${GUESTDISK}/etc/resolv.conf
cat << EOZ >${GUESTDISK}/etc/network/interfaces
auto lo
iface lo inet loopback

auto eth0
iface eth0 inet static
        address 10.0.0.1/30
        gateway 10.0.0.2
EOZ
EOL

# run the script, mounting the disk so we can create a rootfs
docker run \
  -v $(pwd)/$ROOTFSMOUNT:$GUESTDISK \
  -v $(pwd)/$INITSCRIPT:/init.sh \
  --rm \
  --entrypoint sh \
  alpine:3.1  \
  /init.sh

# TODO(johnrwatson): This can never make it into Production
# We need to figure out how to pass these decryption keys at all for the services
# That need them, maybe we need another sub-service specifically for fetching these from
# a secret provider or similar. Only for cyclone pull the dev decryption key
[[$(echo $binary_input | awk -F "/" '{print $NF}') == "cyclone" ]] && cp $GITROOT/lib/cyclone-server/src/dev.decryption.key $ROOTFSMOUNT

# cleanup the PACKAGEDIR
sudo umount $ROOTFSMOUNT
rm -rf $ROOTFSMOUNT $INITSCRIPT

mv $PACKAGEDIR/cyclone-rootfs.ext4 $output_file