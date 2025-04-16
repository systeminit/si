#!/bin/bash

# TODO(johnrwatson): In theory we should be able to run this task for any of the
# components we need rootfs' for but there are some cyclone-specifics bits that
# will cause us problems

set -euxo pipefail

# TODO(johnrwatson): We need to port this to python or similar, and check for
# OS-dependencies that are required. i.e. docker and basic priviledged
# escalation for the mounts

# i.e. ./${git_metadata} | jq -r '.abbreviated_commit_hash' (it returns a json
# blob output via python)
git_metadata=$1
# i.e. ./metadata-out.json (a build metadata file containing contents of type
# etc)
build_metadata_out=$2
# i.e. output the file ./johns_rootfs.tar
tar_file_out=$3

# Shift the parsed arguments off after assignment
shift 3

# The rest of the inputs are a list of input files or directories, to also
# include in the build i.e. consume a binary ./johns_binary.bin for use within
# this script
binary_inputs=("$@")

echo "-------------------------------------"
echo "Info: Initiating rootfs build"
echo "Artifact Version: $(jq -r '.canonical_version' <"$git_metadata")"
echo "Output File: $tar_file_out"
echo "Input Binaries (list):"
for binary_input in "${binary_inputs[@]}"; do
  echo "$(
    echo "$binary_input" \
      | awk -F "/" '{print $NF}'
  ) Full Path: $binary_input"
done
echo "-------------------------------------"

BUCKROOT="$BUCK_SCRATCH_PATH" # This is provided by buck2

BIN=cyclone
PACKAGEDIR=$(realpath "$BUCKROOT/cyclone-pkg")
ROOTFS="$PACKAGEDIR/cyclone-rootfs.ext4"
ROOTFSMOUNT="$PACKAGEDIR/rootfs"
ROOTFS_TAR="rootfs.tar.gz"
INITSCRIPT="$PACKAGEDIR/init.sh"
ALPINE_VERSION=3.18

# Vendored from https://github.com/fnichol/libsh/blob/main/lib/setup_traps.sh
setup_traps() {
  local _sig
  for _sig in HUP INT QUIT ALRM TERM; do
    trap "
      $1
      trap - $_sig EXIT
      kill -s $_sig "'"$$"' "$_sig"
  done
  # shellcheck disable=SC2064
  trap "$1" EXIT

  unset _sig
}

cleanup() {
  set +e

  # cleanup the PACKAGEDIR
  sudo umount -fv "$ROOTFSMOUNT"

  rm -rfv "$ROOTFSMOUNT" "$INITSCRIPT" "$ROOTFS_TAR"
}

setup_traps cleanup

# create disk and mount to a known location
mkdir -pv "$ROOTFSMOUNT"
# As of now, the overlay that maps this image is 10,737,418,240 bytes,
# which 10gb. If you make the image below larger than that, you must increase
# the size of the overlays. They can be found in
# lib/si-firecracker/src/scripts/{firecracker-setup|prepare_jailer}.sh
dd if=/dev/zero of="$ROOTFS" bs=1M count=7189
mkfs.ext4 -v "$ROOTFS"
sudo mount -v "$ROOTFS" "$ROOTFSMOUNT"

cyclone_args=(
  --bind-vsock 3:52
  --lang-server /usr/local/bin/lang-js
  --limit-requests 1
  --watch-timeout 30
  --enable-ping
  --enable-watch
  --enable-forwarder
  --enable-process-gatherer
  -vvvv
)

# got get the rootfs tar and unpack it
curl "https://dl-cdn.alpinelinux.org/alpine/v$ALPINE_VERSION/releases/$(arch)/alpine-minirootfs-$ALPINE_VERSION.0-$(arch).tar.gz" -o $ROOTFS_TAR
sudo tar xf rootfs.tar.gz -C "$ROOTFSMOUNT"

#ENTER CHROOT
sudo chroot "$ROOTFSMOUNT" sh <<EOL

# Set up DNS resolution
echo "nameserver 8.8.8.8" >"/etc/resolv.conf"

apk update
apk add openrc openssh mingetty runuser

adduser -D app
for dir in / run etc usr/local/etc home/app/.config; do
    mkdir -pv "/\$dir/$BIN"
done

# create /dev/null
mknod /dev/null c 1 3
chmod 666 /dev/null

ssh-keygen -A

# Make sure special file systems are mounted on boot:
rc-update add devfs boot
rc-update add procfs boot
rc-update add sysfs boot
rc-update add networking boot
rc-update add local default
rc-update add sshd

# autologin
echo "ttyS0::respawn:/sbin/mingetty --autologin root --noclear ttyS0" >> /etc/inittab
sed -i 's/root:*::0:::::/root:::0:::::/g' /etc/shadow

# mount scripts volume
cat <<EOV >>"/etc/fstab"
LABEL=scripts     /mnt/scripts    ext4   defaults 0 0
EOV

# autostart cyclone
cat <<EOF >"/etc/init.d/cyclone"
#!/sbin/openrc-run

name="cyclone"
description="Cyclone"
supervisor="supervise-daemon"
pidfile="/cyclone/agent.pid"

start(){
  if [ -f /mnt/scripts/scripts ]; then
      source /mnt/scripts/scripts
  fi
  export HOME=/root
  cyclone ${cyclone_args[*]} >> /var/log/cyclone.log 2>&1 &
}

depend(){
  need net
}
EOF

chmod +x "/etc/init.d/cyclone"
rc-update add cyclone

# Set up TAP device route/escape
cat <<EOZ >"/etc/network/interfaces"
auto lo
iface lo inet loopback

auto eth0
iface eth0 inet static
        address 10.0.0.1/30
        gateway 10.0.0.2
EOZ

EOL
# LEAVE CHROOT

# For each tar.gz, copy the contents into the rootfs into the rootfs partition
# we created above. This will cumulatively stack the content of each.
for binary_input in "${binary_inputs[@]}"; do
  sudo tar -xpf "$binary_input" -C "$ROOTFSMOUNT"
done
# Must be unmounted then moved with sudo or permission issues will prevent all directories
# from copying over for some mysterious reason.
sudo umount -fv "$ROOTFSMOUNT"
sudo mv -v "$ROOTFS" "$tar_file_out"

# Then generate the build metadata
#
# TODO(johnrwatson): family here needs adjusted to the service/component name as
# this doesn't currently support services outside of cyclone.
cat <<EOF >"$build_metadata_out"
{
  "family":"cyclone",
  "variant":"rootfs",
  "version":"$(jq -r '.canonical_version' <"$git_metadata")",
  "arch":"$(uname -m | tr '[:upper:]' '[:lower:]')",
  "os":"$(uname -s | tr '[:upper:]' '[:lower:]')",
  "commit": "$(jq -r '.commit_hash' <"$git_metadata")",
  "b3sum": "$(b3sum --no-names "$tar_file_out")"
}
EOF

echo "--- rootfs build complete."
