#!/usr/bin/env bash
set -e

GATEWAY="unknown"
SCRIPTPATH="unknown"

if [ "$(uname -s)" = "Linux" ]; then
    SCRIPTPATH=$(dirname $(realpath $0))
elif [ "$(uname -s)" = "Darwin" ]; then
    SCRIPTPATH=$( cd "$(dirname "$0")" ; pwd -P )
fi
cd $SCRIPTPATH
cd ..
echo "Changed current working directory for script: $(pwd)"

function set-gateway {
    GATEWAY=$(docker run --rm busybox ip route | awk '/^default via/ { print $3 }')
}

# Smoke test the private repository pull first.
function smoke-test {
    docker pull systeminit/sdf:stable
}

function command-dev {
    set +e
    docker-compose down
    set -e
    smoke-test
    set-gateway
    GATEWAY=$GATEWAY docker-compose up
}

function command-down {
    docker-compose down
}

function command-prod {
    smoke-test
    set-gateway
    GATEWAY=$GATEWAY docker-compose up -d
}

case $1 in
    "dev")
        shift
        command-dev
        ;;
    "down")
        shift
        command-down
        ;;
    "prod")
        shift
        command-prod
        ;;
    *)
        echo "dev  -- run interactively and run down beforehand"
        echo "down -- stop and remove resources"
        echo "prod -- run in a detached state and fail on first error"
        exit 1
esac