#!/usr/bin/env bash
set -e

REPOPATH="unknown"
JWT_SECRET_KEY="unknown"
CYCLONE_PUBLIC_KEY="unknown"
CYCLONE_SECRET_KEY="unknown"

function set-repopath {
    REPOPATH=$(dirname $(dirname $(dirname $(realpath -s $0))))
    echo "path to repository base: $REPOPATH"
}

# For local development, this function should likely exit 0 rather than completing.
# It is intended for ephemeral CI use.
function check-init {
    if [ -f $REPOPATH/deploy/docker-compose.env.yml ]; then
        echo "skipping compose env file creation (file already exists): $REPOPATH/deploy/docker-compose.env.yml"
        exit 0
    fi
}

function set-jwt-secret-key {
    JWT_SECRET_KEY=$(realpath $REPOPATH/bin/sdf/src/dev.jwt_secret_key.bin)
    if [ ! -f $JWT_SECRET_KEY ]; then
        echo "file does not exist or could not be found: $JWT_SECRET_KEY"
        exit 1
    fi
}

function set-cyclone-public-key {
    CYCLONE_PUBLIC_KEY=$(realpath $REPOPATH/lib/cyclone/src/dev.public_key.pub)
    if [ ! -f $CYCLONE_PUBLIC_KEY ]; then
        echo "file does not exist or could not be found: $CYCLONE_PUBLIC_KEY"
        exit 1
    fi
}

function set-cyclone-secret-key {
    CYCLONE_SECRET_KEY=$(realpath $REPOPATH/lib/cyclone/src/dev.secret_key.bin)
    if [ ! -f $CYCLONE_SECRET_KEY ]; then
        echo "file does not exist or could not be found: $CYCLONE_SECRET_KEY"
        exit 1
    fi
}

# We will not check for the honeycomb token since it will be passed in by our CI platform.
function perform-init {
    cp $REPOPATH/ci/docker-compose.env.yml $REPOPATH/deploy/docker-compose.env.yml
    sed -i "s|<jwt-secret-key>|$JWT_SECRET_KEY|g" $REPOPATH/deploy/docker-compose.env.yml
    sed -i "s|<cyclone-public-key>|$CYCLONE_PUBLIC_KEY|g" $REPOPATH/deploy/docker-compose.env.yml
    sed -i "s|<cyclone-secret-key>|$CYCLONE_SECRET_KEY|g" $REPOPATH/deploy/docker-compose.env.yml
    sed -i "s|<honeycomb-token>|$HONEYCOMB_TOKEN|g" $REPOPATH/deploy/docker-compose.env.yml
}


set-repopath
check-init
set-jwt-secret-key
set-cyclone-public-key
set-cyclone-secret-key
perform-init
