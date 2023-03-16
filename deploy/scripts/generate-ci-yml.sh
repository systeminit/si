#!/usr/bin/env bash

BEFORE_SHA=$1
AFTER_SHA=$2

function set-repopath {
  REPOPATH="$(dirname "$(dirname "$(dirname "$(realpath -s "$0")")")")"
  if [[ -z "$REPOPATH" ]]; then
    echo "REPOPATH not set, aborting"
    exit 1
  else
    echo "path to repository base: $REPOPATH"
  fi
}

echo "::group::Generating correct docker compose for CI web testing"
set-repopath
CHANGED_COMPONENTS=$(git --no-pager diff --name-only $BEFORE_SHA...$AFTER_SHA | xargs dirname | grep '^lib/\|bin/' | awk -F"/" '{print $1 "/" $2 "\n"}' | sort -u)
echo $CHANGED_COMPONENTS | grep "lib/sdf-server\|lib/dal\|bin/sdf\|lib/veritech-server\|bin/veritech\|bin/pinga\|lib/pinga-server\|bin/council"
found_components=$?
if [ $found_components -ne 0 ]; then
  echo "No need to build sdf, veritech, pinga and council"
  cp $REPOPATH/deploy/scripts/partials/docker-compose.ci-minimal.yml $REPOPATH/deploy/docker-compose.ci.yml
else
  pushd $REPOPATH
  make build//bin/sdf build//bin/veritech build//bin/pinga  build//bin/council
  make_succeded=$?
  if [ $make_succeded -ne 0 ]; then
    echo "Build of sdf, veritech, pinga and council failed; aborting!"
    exit 1;
  fi
  cp $REPOPATH/deploy/scripts/partials/docker-compose.ci.yml $REPOPATH/deploy/docker-compose.ci.yml
fi
echo "::endgroup::"

exit 0
