#!/usr/bin/env bash
set -e

GATEWAY="unknown"
SCRIPTPATH="unknown"

if [ "$(uname -s)" = "Linux" ]; then
    SCRIPTPATH=$(dirname $(realpath $0))
elif [ "$(uname -s)" = "Darwin" ]; then
    SCRIPTPATH=$( cd "$(dirname "$0")" ; pwd -P )
fi

# Only change directory to where the compose file is if we are in the repository.
if [ "$(basename $(dirname $SCRIPTPATH))" = "deploy" ]; then
    cd $(dirname $SCRIPTPATH)
fi

function set-gateway {
    GATEWAY=$(docker run --rm busybox ip route | awk '/^default via/ { print $3 }')
}

# Smoke test the private repository pull first.
function smoke-test {
    env GATEWAY=$GATEWAY docker-compose pull
}

function check-for-env-yml {
  if [[ ! -f "docker-compose.env.yml" ]]; then
    echo "Missing docker-compose.env.yml - must be included for prod!";
    echo "A sample file appears below, for your convenience. ;)";
    echo "---"
    echo "services:"
    echo "  web:"
    echo "    environment:"
    echo "      - HONEYCOMB_TOKEN=the_token"
    echo "      - HONEYCOMB_DATASET=the_dataset"
  fi
}

function command-dev {
    set +e
    env GATEWAY=$GATEWAY docker-compose down
    set -e
    smoke-test
    check-for-env-yml
    set-gateway
    env GATEWAY=$GATEWAY docker-compose -f docker-compose.yml -f docker-compose.env.yml up
}

function command-down {
    env GATEWAY=$GATEWAY docker-compose down
}

function command-prod {
    smoke-test
    check-for-env-yml
    set-gateway
    env GATEWAY=$GATEWAY docker-compose -f docker-compose.yml -f docker-compose.env.yml up -d
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
