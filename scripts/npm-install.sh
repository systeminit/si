#!/bin/bash

set -e

for c in "$@"
do
  pushd components/$c && npm install
  popd
done
