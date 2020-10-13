#!/bin/bash

CONTAINER_NAME="db"

docker start ${CONTAINER_NAME} || docker run -d --name ${CONTAINER_NAME} -p 8091-8096:8091-8096 -p 11210-11211:11210-11211 systeminit/couchbase:latest 
