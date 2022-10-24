#!/usr/bin/env bash

# make sure SI_ROOT is set (run from make deploy//partial-local-pg)

set -e

REPO_PORT=31337
REPO_TAG=localhost:${REPO_PORT}/systeminit/postgres

if [[ "$(docker image inspect ${REPO_TAG} --format='image exists' 2> /dev/null)" == "" ]]; then
    pushd $SI_ROOT
    make image//component/postgres
    echo "Tagging"
    docker tag systeminit/postgres:latest ${REPO_TAG}
    popd
fi
echo "Starting registry"
if [[ "$(docker ps | grep registry)" != "" ]]; then
    docker stop registry
fi
docker run --rm -d -p ${REPO_PORT}:5000 --name registry registry:2
sleep 1
echo "Pushing"
docker push ${REPO_TAG}
