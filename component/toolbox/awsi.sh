#!/usr/bin/env bash

docker run --rm -ti -v ~/.aws:/root/.aws -v "$(pwd)":/aws systeminit/toolbox:stable "$*"
