#!/bin/bash

docker run -d --name db -p 8091-8096:8091-8096 -p 11210-11211:11210-11211 docker.pkg.github.com/systeminit/si/couchbase:latest 
