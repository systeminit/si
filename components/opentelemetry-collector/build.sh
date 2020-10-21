#!/bin/sh -ex

REPO="systeminit"
TAG="latest"
IMAGE="otelcol"
RELEASE=$1
DIR="$(dirname "${BASH_SOURCE[0]}")"  # get the directory name
DIR="$(realpath "${DIR}")"    # resolve its full path if need be

if [[ -z "$RELEASE" ]]; then 
  TAG=${USER}
  cat ${DIR}/otel-local-config.yaml | sed "s/USERNAME/${USER}/g" > ${DIR}/otel-local-config-user.yaml
  docker rmi ${REPO}/${IMAGE}:${TAG} && docker build -t ${REPO}/${IMAGE}:${TAG} -f Dockerfile-user . || docker build -t ${REPO}/${IMAGE}:${TAG} -f Dockerfile-user .
else
  docker rmi ${REPO}/${IMAGE}:${TAG} && docker build -t ${REPO}/${IMAGE}:${TAG} -f Dockerfile . || docker build -t ${REPO}/${IMAGE}:${TAG} -f Dockerfile .
fi
