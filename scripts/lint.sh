#!/bin/bash

set -e

pushd ./components/si-graphql && npm run lint
popd
pushd ./components/si-web-app && npm run lint
