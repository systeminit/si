#!/usr/bin/env sh

VERSION=1.10.4.1-alpine

docker run \
  --env "DOCKER_VERNEMQ_ACCEPT_EULA=yes" \
  --env "DOCKER_VERNEMQ_ALLOW_ANONYMOUS=on" \
  --env "DOCKER_VERNEMQ_LISTENER__TCP__ALLOWED_PROTOCOL_VERSIONS=3,4,131,5" \
  --publish 1883:1883 \
  --name mqtt \
  --detach \
  "vernemq/vernemq:$VERSION"
