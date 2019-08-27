#!/bin/bash

pushd ./components/si-graphql && npm install
popd
pushd ./components/si-web-app && npm install
