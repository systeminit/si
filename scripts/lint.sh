#!/bin/bash

set -e

for c in "$@"
do
  pushd $c && npm run lint
  popd
done
