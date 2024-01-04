#!/bin/bash

set -euo pipefail

SB_ID="${1:-null}"

# Kill the firecracker process
ps aux | grep "firecracke[r] --id $SB_ID" | awk '{ print $2 }' | xargs kill -9 || true
