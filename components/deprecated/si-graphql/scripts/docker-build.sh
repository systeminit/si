#!/usr/bin/env bash

# Get the parent directory of where this script is.
SOURCE="${BASH_SOURCE[0]}"
while [ -h "$SOURCE" ] ; do SOURCE="$(readlink "$SOURCE")"; done
DIR="$( cd -P "$( dirname "$SOURCE" )/.." && pwd )"

# Change into that directory
cd "$DIR"

export CONTAINER_NAME=si/graphql-service
export CONTAINER_VERSION=latest
export DOCKER_FILE=Dockerfile

echo "-------------------------------------------------"
echo "==> Building Container: ${CONTAINER_NAME}"
echo "-------------------------------------------------"
docker build -t ${CONTAINER_NAME}:${CONTAINER_VERSION} -f ${DOCKER_FILE} .