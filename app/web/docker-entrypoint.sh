#!/usr/bin/env bash
set -eu

main() {
  if [ -n "${SI__WEB__SDF_HOST:-}" ]; then
    echo "Info: SI__WEB__SDF_HOST supplied [ $SI__WEB__SDF_HOST ]! Registering in nginx.conf"
    sed -i "s^server sdf:5156^server $SI__WEB__SDF_HOST^g" "$(find /nix/store/ -path '/nix/store/*web/conf/nginx.conf')"
  fi
  exec @@nginx@@ -c @@conf@@ -p @@prefix@@ -g "daemon off;" "$@"
}

main "$@"
