#!/usr/bin/env bash

SCRIPT_PATH="$( cd "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"
CACHE_PATH="$SCRIPT_PATH/../node_modules/.vite"

if [ -d $CACHE_PATH ]
then 
    rm -r $CACHE_PATH
    echo "Vite cache removed."
else
    echo "Vite cache not found - nothing to clean."
fi