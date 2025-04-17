#!/usr/bin/env bash
if [ -z "$1" ] || [ "$1" = "-h" ] || [ "$1" = "--help" ] || [ "$1" = "help" ]; then
  echo "============================================================================"
  echo "Provide the buck2 command to run for all Rust targets in the repository."
  echo "You can also provide the mode, flags, and more (e.g. 'build @//mode/debug')."
  echo "============================================================================"
  exit 1
fi

set -euxo pipefail
buck2 uquery 'kind("rust_(binary|library|test)", set("//bin/..." "//lib/..."))' | xargs buck2 "$@"
