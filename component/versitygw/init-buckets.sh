#!/usr/bin/env bash
set -euo pipefail

main() {
  local data_dir="${VGW_DATA_DIR:-/data}"

  echo "--- Creating S3 buckets (directories) in ${data_dir}"

  # Create all layer cache buckets
  local buckets=(
    "si-layer-cache-cas"
    "si-layer-cache-change-batches"
    "si-layer-cache-workspace-snapshots"
    "si-layer-cache-rebase-batches"
    "si-layer-cache-encrypted-secrets"
    "si-layer-cache-func-runs"
    "si-layer-cache-func-run-logs"
    "si-layer-cache-split-snapshot-subgraphs"
    "si-layer-cache-split-snapshot-supergraphs"
    "si-layer-cache-split-snapshot-rebase-batches"
  )

  for bucket in "${buckets[@]}"; do
    mkdir -p "${data_dir}/${bucket}"
    echo "  - Created bucket: ${bucket}"
  done

  # Create ready file for healthcheck
  echo "ready" > /tmp/ready
  echo "--- Bucket initialization complete"
}

main "$@"
