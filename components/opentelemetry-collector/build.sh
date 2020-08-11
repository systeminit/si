#!/bin/sh -ex

REPO="docker.pkg.github.com/systeminit"
TAG="latest"
IMAGE="si/otelcol"
DOCKER_CONTAINER_NAME="otelcol"

docker rmi ${REPO}/${IMAGE}:${TAG} && docker build -t ${REPO}/${IMAGE}:${TAG} . || docker build -t ${REPO}/${IMAGE}:${TAG} .


