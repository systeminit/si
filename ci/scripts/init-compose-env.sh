#!/usr/bin/env bash
set -eu

function set-repopath {
  REPOPATH="$(dirname "$(dirname "$(dirname "$(realpath -s "$0")")")")"
  if [[ -z "$REPOPATH" ]]; then
    echo "REPOPATH not set, aborting"
    exit 1
  else
    echo "path to repository base: $REPOPATH"
  fi
}

# For local development, this function should likely exit 0 rather than completing.
# It is intended for ephemeral CI use.
function check-init {
  if [[ -f "$REPOPATH/deploy/docker-compose.env.yml" ]]; then
    echo "skipping compose env file creation (file already exists): " \
      "$REPOPATH/deploy/docker-compose.env.yml"
    exit 0
  fi
}

function set-cyclone-encryption-key {
  CYCLONE_ENCRYPTION_KEY="$(
    realpath "$REPOPATH/lib/cyclone-server/src/dev.encryption.key"
  )"
  if [[ ! -f "$CYCLONE_ENCRYPTION_KEY" ]]; then
    echo "file does not exist or could not be found: $CYCLONE_ENCRYPTION_KEY"
    exit 1
  fi
}

function set-cyclone-decryption-key {
  CYCLONE_DECRYPTION_KEY="$(
    realpath "$REPOPATH/lib/cyclone-server/src/dev.decryption.key"
  )"
  if [[ ! -f "$CYCLONE_DECRYPTION_KEY" ]]; then
    echo "file does not exist or could not be found: $CYCLONE_DECRYPTION_KEY"
    exit 1
  fi
}

# We will not check for the honeycomb token since it will be passed in by our CI platform.
function perform-init {
  local compose_yaml="$REPOPATH/deploy/docker-compose.env.yml"

  cp "$REPOPATH/ci/docker-compose.env.yml" "$compose_yaml"
  sed -i "s|<cyclone-encryption-key>|$CYCLONE_ENCRYPTION_KEY|g" "$compose_yaml"
  sed -i "s|<cyclone-decryption-key>|$CYCLONE_DECRYPTION_KEY|g" "$compose_yaml"
  sed -i "s|<honeycomb-token>|$HONEYCOMB_TOKEN|g" "$compose_yaml"
}

set-repopath
check-init
set-cyclone-encryption-key
set-cyclone-decryption-key
perform-init
