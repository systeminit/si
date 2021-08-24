#!/usr/bin/env bash

export CONTAINER_NAME=si/skopeo
export CONTAINER_VERSION=latest
export DOCKER_FILE=Dockerfile

echo "-------------------------------------------------"
echo "==> Building Container: ${CONTAINER_NAME}"
echo "-------------------------------------------------"

cd "${0%/*}/.."
docker build -t ${CONTAINER_NAME}:${CONTAINER_VERSION} -f ${DOCKER_FILE} .