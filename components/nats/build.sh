#!/bin/sh -ex

REPO="systeminit"
TAG="latest"
IMAGE="nats"
DOCKER_CONTAINER_NAME="nats"

docker rmi ${REPO}/${IMAGE}:${TAG} && docker build -t ${REPO}/${IMAGE}:${TAG} . || docker build -t ${REPO}/${IMAGE}:${TAG} .
