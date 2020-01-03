#!/bin/bash

set -e

echo "*** Starting $START_SERVICE ***"
cd /svc/$START_SERVICE
if [[ -f $START_SERVICE-server ]]; then 
  exec ./$START_SERVICE-server "$@"
else
  exec env NODE_ENV=production npm run prod
fi
