#!/bin/bash

set -e

pushd ./components/si-graphql && npm run build
popd
pushd ./components/si-web-app && npm run build
