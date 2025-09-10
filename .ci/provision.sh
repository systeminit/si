#!/bin/bash
set -euxo pipefail

# 0. Remove broken EC2 repo if it exists
if grep -q "\[ec2\]" /etc/pacman.conf; then
  echo "[INFO] Removing broken 'ec2' repo from /etc/pacman.conf"
  sed -i '/^\[ec2\]/,/^$/d' /etc/pacman.conf
fi

# 1. Update base packages & generate keychain
pacman -Sy --noconfirm
pacman-key --init
pacman-key --populate archlinux

# 2. Install tools:
pacman -S --noconfirm base-devel git wget postgresql docker

# 2(a) install yay for aur repo usage for us to grab ssm-agent
#git clone https://aur.archlinux.org/yay.git
#makepkg -si --noconfirm -D ./yay/
#yay -Sy amazon-ssm-agent --noconfirm

# (b) Enable and start SSM Agent
#systemctl enable --now amazon-ssm-agent

# (c) Enable and start Docker
systemctl enable --now docker

# 3. Install Docker Compose manually
echo "🔧 Installing latest Docker Compose..."
BINARY_PATH="/usr/local/bin/docker-compose"
curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o "$BINARY_PATH"
chmod +x "$BINARY_PATH"

if command -v docker-compose &> /dev/null; then
    docker-compose version
    echo "🎉 Docker Compose installed successfully."
else
    echo "❌ Failed to install docker-compose." >&2
    exit 1
fi

# 4. Install Nix (multi-user daemon mode)
export HOME=/root
curl -L https://nixos.org/nix/install | sh -s -- --daemon

# 3. Clone the System Initiative repo
git clone https://github.com/systeminit/si.git
cd si

RBE_TOKEN=$(aws secretsmanager get-secret-value \
  --region us-east-1 \
  --secret-id rbe-token \
  --query SecretString \
  --output text)

# Activate nix in current session
. /etc/profile.d/nix.sh

nix develop \
  --extra-experimental-features nix-command \
  --extra-experimental-features flakes \
  -c bash -c "SI_RBE_TOKEN='$RBE_TOKEN' DEV_HOST=0.0.0.0 TILT_HOST=0.0.0.0 buck2 run dev:up"
