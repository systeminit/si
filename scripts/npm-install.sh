#!/bin/bash

set -e

for c in "$@"
do
  pushd $c && npm install
  popd
done
