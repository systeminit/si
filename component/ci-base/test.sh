#!/usr/bin/env bash
set -euxo pipefail
SCRIPT_DIR=$(cd $(dirname "${BASH_SOURCE[0]}"); pwd -P)
REPO_DIR=$(dirname $(dirname $SCRIPT_DIR))

pushd $REPO_DIR
docker build -f $SCRIPT_DIR/Dockerfile -t ci-base .
popd

docker run --rm -it -v $REPO_DIR:/src ci-base