#!/usr/bin/env sh

CONTAINER_NAME="nats"

docker start ${CONTAINER_NAME} || docker run \
  --name ${CONTAINER_NAME} \
  --publish 4222:4222 \
  --detach systeminit/nats:latest
