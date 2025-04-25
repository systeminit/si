#!/usr/bin/env bash
set -euo pipefail
VERSIONS_DIRECTORY=$(dirname "$(realpath "$0")")

for VERSION_FILE in $(find "$VERSIONS_DIRECTORY" -type f -name "*.bzl" | sort); do
  FILE_CONTENT=$(<"$VERSION_FILE")
  FILE_NAME=$(basename "$VERSION_FILE")
  echo "${FILE_CONTENT} / ${FILE_NAME}"
done
