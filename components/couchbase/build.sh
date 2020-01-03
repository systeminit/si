#!/bin/sh -ex

TAG="6.5.0-beta2"
IMAGE="si/couchbase"

docker build -t docker.pkg.github.com/systeminit/${IMAGE}:${TAG} -t docker.pkg.github.com/systeminit/${IMAGE}:latest .
