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

# Run platform first so we can use Bedrock before spinning up the backend
nix develop \
  --extra-experimental-features nix-command \
  --extra-experimental-features flakes \
  -c bash -c "SI_RBE_TOKEN='$RBE_TOKEN' DEV_HOST=0.0.0.0 TILT_HOST=0.0.0.0 buck2 run dev:platform" &

BUCK_PID=$!  # capture PID of the backgrounded nix/buck process

echo "Started buck2 run dev:platform (PID: $BUCK_PID). Waiting for service on http://localhost:3020/ ..."

# Wait until port 3020 responds with HTTP 200 on /
TIMEOUT=600
INTERVAL=2   # seconds between checks
ELAPSED=0

# Seed the database using Bedrock
while ! curl --silent --fail --max-time 2 --output /dev/null "http://localhost:3020/"; do
  sleep "$INTERVAL"
  ELAPSED=$((ELAPSED + INTERVAL))
  if [ "$ELAPSED" -ge "$TIMEOUT" ]; then
    echo "Error: Service did not respond within $TIMEOUT seconds."
    # Best-effort cleanup
    if ps -p "$BUCK_PID" >/dev/null 2>&1; then
      nix develop \
        --extra-experimental-features nix-command \
        --extra-experimental-features flakes \
        -c bash -c "SI_RBE_TOKEN='$RBE_TOKEN' DEV_HOST=0.0.0.0 TILT_HOST=0.0.0.0 buck2 killall" || true
    fi
    exit 1
  fi
done

# TODO - This 3 minute wait is to ensure that Bedrock finishes seeding the database before we move on
# Ideally we would have some way to watch and see when Bedrock finishes then move on
echo "Service is responding. Waiting..."
sleep 180
echo "Killing buck2 processes..."

# Kill the currently running buck2
nix develop \
  --extra-experimental-features nix-command \
  --extra-experimental-features flakes \
  -c bash -c "SI_RBE_TOKEN='$RBE_TOKEN' DEV_HOST=0.0.0.0 TILT_HOST=0.0.0.0 buck2 killall"

pkill tilt
# Wait for localhost:10350 to stop responding
while true; do
  if ! curl -s --max-time 5 http://localhost:10350 > /dev/null 2>&1; then
    echo "localhost:10350 is no longer responding, continuing..."
    break
  fi
  echo "Waiting for localhost:10350 to stop responding..."
  sleep 2
done

# Run buck2 dev:up
echo "Starting buck2 run dev:up..."
nix develop \
  --extra-experimental-features nix-command \
  --extra-experimental-features flakes \
  -c bash -c "SI_RBE_TOKEN='$RBE_TOKEN' DEV_HOST=0.0.0.0 TILT_HOST=0.0.0.0 buck2 run dev:up"
