#!/usr/bin/env bash
set -eu

main() {
  exec @@nginx@@ -c @@conf@@ -p @@prefix@@ -g "daemon off;" "$@"
}

main "$@"
