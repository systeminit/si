#!/bin/sh -ex

REPO="docker.pkg.github.com/systeminit"
TAG="latest"
IMAGE="si/couchbase"
DOCKER_CONTAINER_NAME="db"

echo "# Couchbase Memory Configuration" > ./.env
echo "export DATA_RAM_QUOTA=${DATA_RAM_QUOTA:-2048}" >> ./.env
echo "export INDEX_RAM_QUOTA=${INDEX_RAM_QUOTA:-512}" >> ./.env
echo "export FULL_TEXT_RAM_QUOTA=${FULL_TEXT_RAM_QUOTA:-512}" >> ./.env
echo "export BUCKET_SI_RAM_QUOTA=${BUCKET_SI_RAM_QUOTA:-512}" >> ./.env
echo "export BUCKET_SI_INTEGRATION_RAM_QUOTA=${BUCKET_SI_INTEGRATION_RAM_QUOTA:-512}" >> ./.env

docker rmi ${REPO}/${IMAGE}:${TAG} && docker build -t ${REPO}/${IMAGE}:${TAG} . || docker build -t ${REPO}/${IMAGE}:${TAG} .


