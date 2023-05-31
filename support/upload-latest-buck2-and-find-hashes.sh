#!/usr/bin/env bash
set -euxo pipefail

# First, cache the universal date.
DATE=$(date -u +%F)

function subflake-hashes {
  PREFETCH_PREFIX="https://buck2-binaries.s3.us-east-2.amazonaws.com/$DATE"
  AARCH64_DARWIN=$(nix hash to-sri --type sha256 $(nix-prefetch-url $PREFETCH_PREFIX/buck2-aarch64-apple-darwin.zst))
  AARCH64_LINUX=$(nix hash to-sri --type sha256 $(nix-prefetch-url $PREFETCH_PREFIX/buck2-aarch64-unknown-linux-gnu.zst))
  X86_64_DARWIN=$(nix hash to-sri --type sha256 $(nix-prefetch-url $PREFETCH_PREFIX/buck2-x86_64-apple-darwin.zst))
  X86_64_LINUX=$(nix hash to-sri --type sha256 $(nix-prefetch-url $PREFETCH_PREFIX/buck2-x86_64-unknown-linux-gnu.zst))
  echo "\
hashes for the subflake:

  x86_64-linux    $X86_64_LINUX
  aarch64-linux   $AARCH64_LINUX
  x86_64-darwin   $X86_64_DARWIN
  aarch64-darwin  $AARCH64_DARWIN
"
}

# Check if we need to perform an upload. Find the hashes for the subflake
# if the objects already exist.
DATE=$(date -u +%F)
if aws --region us-east-2 s3 ls "buck2-binaries/$DATE"; then
  echo "buck2-binaries is up to date: $DATE"
  subflake-hashes
  exit 0
fi

# Download what we need into a temporary directory.
pushd $(mktemp -d)
SOURCE_PREFIX="https://github.com/facebook/buck2/releases/download/latest"
mkdir ${DATE}
pushd ${DATE}
wget $SOURCE_PREFIX/buck2-aarch64-apple-darwin.zst
wget $SOURCE_PREFIX/buck2-aarch64-unknown-linux-gnu.zst
wget $SOURCE_PREFIX/buck2-x86_64-apple-darwin.zst
wget $SOURCE_PREFIX/buck2-x86_64-unknown-linux-gnu.zst
popd

# Upload the files to the bucket.
aws \
  --region us-east-2 \
  s3 mv ${DATE} "s3://buck2-binaries/$DATE/" \
  --recursive \
  --acl public-read
popd

# Find the hashes for the subflake.
subflake-hashes
echo "success! now you can update the buck2 subflake to use the new objects with the hashes above"
