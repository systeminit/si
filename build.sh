#!/usr/bin/env bash
set -euxo pipefail
for image in "edda" "forklift" "luminork" "pinga" "rebaser" "sdf" "veritech"; do
  buck2 build "//bin/${image}:${image}-container-image"
done
