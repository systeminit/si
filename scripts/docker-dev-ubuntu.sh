#!/bin/bash

SCRIPT_PATH=$(dirname $(realpath -s $0))
docker run --name si-ubuntu-dev -it --rm -v $SCRIPT_PATH/../:/src ubuntu:latest bash
