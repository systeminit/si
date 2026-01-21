#!/usr/bin/env bash
set -euxo pipefail

tag="20260121.190028.0-sha.d890379e_dirty-amd64"

for image in "edda" "forklift" "luminork" "pinga" "rebaser" "sdf" "veritech"; do
  docker tag "systeminit/${image}:${tag}" "systeminit/${image}:stable"
done
