#!/usr/bin/env bash
set -euxo pipefail

# Ensure that we can move the files to this script's directory on both macOS and Linux.
SCRIPT_DIR=$(
  cd $(dirname "${BASH_SOURCE[0]}")
  pwd -P
)

# Build and upload the flake outputs.
pushd $(dirname $SCRIPT_DIR)
nix build --json | jq -r '.[].outputs | to_entries[].value' | cachix push buck2
popd