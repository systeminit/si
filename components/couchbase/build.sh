#!/bin/sh -ex

TAG="6.5.0-beta2"
IMAGE="si/couchbase"

docker build -t ${IMAGE}:${TAG} -t ${IMAGE}:latest .
