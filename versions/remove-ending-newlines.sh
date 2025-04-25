#!/usr/bin/env bash
set -euo pipefail
VERSIONS_DIRECTORY=$(dirname "$(realpath "$0")")

echo "Target directory: $VERSIONS_DIRECTORY"
while true; do
  read -r -n1 -p "Confirm to begin [Y/n]: " yn
  case $yn in
  [yY])
    echo ""
    break
    ;;
  "")
    echo "Exiting..."
    exit 0
    ;;
  *)
    echo ""
    echo "Exiting..."
    exit 0
    ;;
  esac
done

find "$VERSIONS_DIRECTORY" -type f -name "*.bzl" -exec sed -i -z 's/\n\{1,\}$//' {} \;
echo "Done!"
