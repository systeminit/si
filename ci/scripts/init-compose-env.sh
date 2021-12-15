#!/usr/bin/env bash
set -e

REPO_PATH=$(dirname $(dirname $(dirname $(realpath -s $0))))
echo "path to repo directory: $REPO_PATH"

if [ -f $REPO_PATH/deploy/docker-compose.env.yml ]; then
    echo "skipping docker-compose.env.yml creation (file exists): $REPO_PATH/deploy/docker-compose.env.yml"
    exit 0
fi

CI_JWT_SECRET_KEY=$(realpath $REPO_PATH/bin/sdf/src/dev.jwt_secret_key.bin)
echo "path to jwt secret key: $CI_JWT_SECRET_KEY"

sed -i "s|<jwt-secret-key>|$CI_JWT_SECRET_KEY|g" $REPO_PATH/ci/docker-compose.env.yml
sed -i "s|<honeycomb-token>|$HONEYCOMB_TOKEN|g" $REPO_PATH/ci/docker-compose.env.yml

cp $REPO_PATH/ci/docker-compose.env.yml $REPO_PATH/deploy/docker-compose.env.yml