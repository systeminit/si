#!/bin/bash
set -ex

SHIMS="$1"
shift 1

SHIMS=$(readlink -f "$SHIMS")

export PATH="$SHIMS:$PATH"
export C="$SHIMS/clang"
export CXX="$SHIMS/clang++"

exec "$@"
