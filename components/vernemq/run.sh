#!/bin/bash

docker run -e "DOCKER_VERNEMQ_ACCEPT_EULA=yes" -e "DOCKER_VERNEMQ_ALLOW_ANONYMOUS=on" -p 1883:1883 -p 8081:8080 --name vernemq -d erlio/docker-vernemq
