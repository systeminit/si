#!/usr/bin/env bash
set -eu

main() {
  if [ -n "${SI__SDF__HOST:-}" ]; then
    echo "Info: SI__SDF__HOST supplied [ $SI__SDF__HOST ]! Registering in nginx.conf"
    sed -i "s/server sdf:5156/$SI__SDF__HOST/g" "$(find /nix/store/ -path '/nix/store/*web/conf/nginx.conf')"
  fi
  exec @@nginx@@ -c @@conf@@ -p @@prefix@@ -g "daemon off;" "$@"
}

main "$@"
