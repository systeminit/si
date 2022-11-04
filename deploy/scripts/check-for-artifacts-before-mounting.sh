#!/usr/bin/env bash

set -euxo pipefail

echo "::group::Checking for required artifacts"
[[ -f "$REPOPATH/target/debug/sdf" ]]
[[ -f "$REPOPATH/bin/sdf/src/dev.jwt_secret_key.bin" ]]
[[ -f "$REPOPATH/lib/cyclone-server/src/dev.encryption.key" ]]
[[ -f "$REPOPATH/target/debug/veritech" ]]
[[ -f "$REPOPATH/target/debug/cyclone" ]]
[[ -f "$REPOPATH/bin/lang-js/target/lang-js" ]]
[[ -d "$REPOPATH/app/web/dist" ]]
[[ -f "$REPOPATH/app/web/nginx.conf" ]]
[[ -f "$REPOPATH/app/web/nginx/dhparam.pem" ]]
[[ -f "$REPOPATH/app/web/nginx/nginx-selfsigned.crt" ]]
[[ -f "$REPOPATH/app/web/nginx/nginx-selfsigned.key" ]]
[[ -f "$REPOPATH/target/debug/pinga" ]]
echo "::endgroup::"
exit 0
