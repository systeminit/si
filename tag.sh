#!/usr/bin/env bash
set -euxo pipefail

tag="20260123.001416.0-sha.f58bd6ad-amd64"

for image in "edda" "forklift" "luminork" "pinga" "rebaser" "sdf" "veritech"; do
  docker tag "systeminit/${image}:${tag}" "systeminit/${image}:stable"
done
