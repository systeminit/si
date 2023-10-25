#!/bin/sh
# shellcheck disable=SC3043
set -eu

main() {
  exec /usr/local/bin/.cyclone "$@"
}

main "$@"
