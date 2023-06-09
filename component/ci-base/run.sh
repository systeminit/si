#!/usr/bin/env bash
set -euxo pipefail
echo "tag provided: $1"

SCRIPT_DIR=$(cd $(dirname "${BASH_SOURCE[0]}"); pwd -P)
REPO_DIR=$(dirname $(dirname $SCRIPT_DIR))

docker run --rm -it -v $REPO_DIR:/src docker.io/systeminit/ci-base:$1
