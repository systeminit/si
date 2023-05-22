#!/usr/bin/env bash
set -euxo pipefail

# Ensure that we can move the files to this script's directory on both macOS and Linux.
SCRIPT_DIR=$(
  cd $(dirname "${BASH_SOURCE[0]}")
  pwd -P
)
FLAKE_DIR=$(dirname $SCRIPT_DIR)

# Must provide commit to generate lockfile and patch file for.
echo "commit: $1"

# Create a temporary directory, clone the repository, and checkout the provided commit.
TEMP_DIR=$(mktemp -d)
cd $TEMP_DIR
git clone https://github.com/facebook/buck2.git buck2
cd buck2
git checkout $1

# Generate the new lockfile and patch file. The "git diff" command will have a non-zero exit code,
# so we need to ignore it.
cargo generate-lockfile
git --no-pager diff /dev/null Cargo.lock >${FLAKE_DIR}/Cargo.lock.patch || true
cp Cargo.lock ${FLAKE_DIR}/Cargo.lock
