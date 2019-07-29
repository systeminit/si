#!/usr/bin/env bash

# Get the parent directory of where this script is.
SOURCE="${BASH_SOURCE[0]}"
while [ -h "$SOURCE" ] ; do SOURCE="$(readlink "$SOURCE")"; done
DIR="$( cd -P "$( dirname "$SOURCE" )/.." && pwd )"

# Change into that directory
cd "$DIR"

# Install dependencies
echo "-------------------------------------------------"
echo "==> Installing dependencies"
echo "-------------------------------------------------"
npm install --no-shrinkwrap --no-package-lock
npm install knex -g