#!/usr/bin/env bash

APP_DIR="/app"
COMPONENTS_DIR="$APP_DIR/components"
GRAPHQL_DIR="$COMPONENTS_DIR/si-graphql"
WEBAPP_DIR="$COMPONENTS_DIR/si-web-app"

function printHeader() {
    printf "\n%s\n" "-------------------------------------------------"
    printf "%s\n" "==> $1"
    printf "%s\n" "-------------------------------------------------"
}

# General dependencies
printHeader "installing general dependencies"
apt-get update
apt-get install -y \
        build-essential \
        apt-transport-https \
        lsb-release \
        ca-certificates \
        curl
apt-get install -y --reinstall make

# Installing nodeJs and npm
printHeader "installing nodejs"
curl -sL https://deb.nodesource.com/setup_10.x -o ./node-setup_10.x
sh ./node-setup_10.x
apt-get install -y nodejs

# Installing si-graphql dependencies
printHeader "installing si-graphql dependencies"
cd $GRAPHQL_DIR
make deps-install
make db-init

# Installing si-web-app dependencies
printHeader "installing si-web-app"
cd ${WEBAPP_DIR}
make deps-install
