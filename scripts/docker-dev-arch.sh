#!/bin/bash

SCRIPT_PATH=$(dirname $(realpath -s $0))
docker run --name si-arch-dev -it --rm -v $SCRIPT_PATH/../:/src archlinux:latest bash
