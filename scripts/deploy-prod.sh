#!/usr/bin/env bash

set -e

SCRIPT_PATH=$(dirname $(realpath -s $0))

echo "*** Deploying production ***"

echo "*** Rsync ***"
rsync -vaP $SCRIPT_PATH/../deploy --exclude 'docker-compose.env.yml' fedora@prod-1:~

echo "*** Deployment ***"
ssh fedora@prod-1 '(cd ~/deploy && make down; make prod)'
