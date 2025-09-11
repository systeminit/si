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
  --disable-forwarder
  --enable-process-gatherer
  -vvvv
)

# got get the rootfs tar and unpack it
curl "https://dl-cdn.alpinelinux.org/alpine/v$ALPINE_VERSION/releases/$(arch)/alpine-minirootfs-$ALPINE_VERSION.0-$(arch).tar.gz" -o $ROOTFS_TAR
sudo tar xf rootfs.tar.gz -C "$ROOTFSMOUNT"

#ENTER CHROOT
sudo chroot "$ROOTFSMOUNT" sh <<EOL

echo "nameserver 8.8.8.8" >"/etc/resolv.conf"

apk update && apk add --no-cache \
  aws-cli \
  binutils \
  curl \
  git \
  mingetty \
  openssh \
  openssh-client \
  openrc \
  py3-pip \
  python3 \
  runuser \
  skopeo \
  wget \
  xz

# Minimal nix setup for dynamic linking
mkdir -p /nix && cd /tmp
ARCH=\$(uname -m)
if [ "\$ARCH" = "aarch64" ]; then
    NIX_ARCH="aarch64-linux"
    LD_LINUX_NAME="ld-linux-aarch64.so.1"
    LIB_DIR="/lib"
else
    NIX_ARCH="x86_64-linux"
    LD_LINUX_NAME="ld-linux-x86-64.so.2"
    LIB_DIR="/lib64"
fi

curl -L https://releases.nixos.org/nix/nix-2.30.2/nix-2.30.2-\$NIX_ARCH.tar.xz | tar -xJ
mv nix-2.30.2-\$NIX_ARCH/store /nix/ && cp -rL nix-2.30.2-\$NIX_ARCH/.reginfo /nix/store/
/nix/store/*-nix-*/bin/nix-store --load-db < /nix/store/.reginfo

# Linking stuff for cyclone and langjs only
GLIBC_PATH=\$(find /nix/store -name "glibc-*" -type d 2>/dev/null | head -1)
GCC_LIB_PATH=\$(find /nix/store -name "*gcc*-lib" -type d 2>/dev/null | head -1)
LD_LINUX_PATH=\$(find /nix/store -name "\$LD_LINUX_NAME" | head -1)
[ -n "\$LD_LINUX_PATH" ] && mkdir -p \$LIB_DIR && ln -sf "\$LD_LINUX_PATH" \$LIB_DIR/\$LD_LINUX_NAME
rm -rf nix-2.30.2-\$NIX_ARCH

# Install binary tools
mkdir -p /usr/local/bin /opt && cd /tmp

install_github_binary() {
  local repo=\$1 binary=\$2 pattern=\$3
  local url=\$(curl -s "https://api.github.com/repos/\$repo/releases/latest" | grep "\$pattern" | cut -d '"' -f 4)
  curl -sL "\$url" -o temp.tar.gz && tar -xzf temp.tar.gz
  find . -name "\$binary" -type f -exec mv {} "/usr/local/bin/" \; && chmod +x "/usr/local/bin/\$binary"
  rm -rf temp.tar.gz \$(ls -d */ 2>/dev/null)
}

# Set architecture-specific patterns
if [ "\$ARCH" = "aarch64" ]; then
    GH_PATTERN="browser_download_url.*linux_arm64.tar.gz"
    FASTLY_PATTERN="browser_download_url.*linux-arm64.tar.gz"
    BUTANE_ARCH="aarch64-unknown-linux-gnu"
    MC_ARCH="linux-arm64"
    DOCTL_ARCH="linux-arm64"
    GCP_ARCH="linux-arm"
    DENO_ARCH="aarch64-unknown-linux-gnu"
else
    GH_PATTERN="browser_download_url.*linux_amd64.tar.gz"
    FASTLY_PATTERN="browser_download_url.*linux-amd64.tar.gz"
    BUTANE_ARCH="x86_64-unknown-linux-gnu"
    MC_ARCH="linux-amd64"
    DOCTL_ARCH="linux-amd64"
    GCP_ARCH="linux-x86_64"
    DENO_ARCH="x86_64-unknown-linux-gnu"
fi

# GitHub CLI
install_github_binary "cli/cli" "gh" "\$GH_PATTERN"

# Fastly CLI
install_github_binary "fastly/cli" "fastly" "\$FASTLY_PATTERN"

# Butane
curl -sL "https://github.com/coreos/butane/releases/latest/download/butane-\$BUTANE_ARCH" -o /usr/local/bin/butane

# MinIO Client
curl -sL "https://dl.min.io/client/mc/release/\$MC_ARCH/mc" -o /usr/local/bin/mc

# DigitalOcean CLI
curl -sL "https://github.com/digitalocean/doctl/releases/download/v1.118.0/doctl-1.118.0-\$DOCTL_ARCH.tar.gz" | tar -xz && mv doctl /usr/local/bin/

# Google Cloud SDK
curl -sL "https://dl.google.com/dl/cloudsdk/channels/rapid/downloads/google-cloud-cli-\$GCP_ARCH.tar.gz" | tar -xz -C /opt/

for tool in gcloud gsutil bq; do ln -sf "/opt/google-cloud-sdk/bin/\$tool" "/usr/local/bin/\$tool"; done
chmod +x /usr/local/bin/{butane,mc,doctl}

# Deno
curl -sL "https://github.com/denoland/deno/releases/download/v2.2.12/deno-\$DENO_ARCH.zip" -o deno.zip
unzip -q deno.zip && mv deno /usr/local/bin/ && chmod +x /usr/local/bin/deno && rm -f deno.zip

# Linode CLI
pip3 install --break-system-packages linode-cli

# Azure CLI
pip3 install azure-cli

cd / && rm -rf /tmp/* 2>/dev/null && adduser -D app

# Create wrapper scripts for binaries
for binary in cyclone lang-js; do
  cat > "/usr/local/bin/\$binary-wrapper" << EOF
#!/bin/sh
export LD_LIBRARY_PATH="\${GLIBC_PATH}/lib:\${GCC_LIB_PATH}/lib:/usr/lib64:/lib:/usr/lib:\\\$LD_LIBRARY_PATH"
exec "/usr/local/bin/\$binary-original" "\\\$@"
EOF
  chmod +x "/usr/local/bin/\$binary-wrapper"
done

cat > /usr/local/bin/setup-wrappers << 'EOF'
#!/bin/sh
for bin in cyclone lang-js; do
  [ -f "/usr/local/bin/\$bin" ] && [ ! -f "/usr/local/bin/\$bin-original" ] && {
    mv "/usr/local/bin/\$bin" "/usr/local/bin/\$bin-original"
    ln -sf "/usr/local/bin/\$bin-wrapper" "/usr/local/bin/\$bin"
  }
done
EOF
chmod +x /usr/local/bin/setup-wrappers && /usr/local/bin/setup-wrappers

mknod /dev/null c 1 3 && chmod 666 /dev/null && ssh-keygen -A

# Configure boot services
for svc in devfs procfs sysfs networking; do rc-update add \$svc boot; done
rc-update add local default && rc-update add sshd

echo "ttyS0::respawn:/sbin/mingetty --autologin root --noclear ttyS0" >> /etc/inittab
sed -i 's/root:*::0:::::/root:::0:::::/g' /etc/shadow
echo "LABEL=scripts /mnt/scripts ext4 defaults 0 0" >> /etc/fstab

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

# For each input, copy the contents into the rootfs
# Handle both tar archives (from omnibus packages) and raw binaries
for binary_input in "${binary_inputs[@]}"; do
  if file "$binary_input" | grep -q "tar archive"; then
    # It's a tar archive, extract it
    sudo tar -xpf "$binary_input" -C "$ROOTFSMOUNT"
  else
    # It's a raw binary, copy it to /usr/local/bin
    binary_name=$(basename "$binary_input")
    sudo mkdir -p "$ROOTFSMOUNT/usr/local/bin"
    sudo cp "$binary_input" "$ROOTFSMOUNT/usr/local/bin/$binary_name"
    sudo chmod +x "$ROOTFSMOUNT/usr/local/bin/$binary_name"
  fi
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
