#!/bin/bash

set -e

echo "*** Starting $START_SERVICE ***"
cd /svc/$START_SERVICE
if [[ -f $START_SERVICE-server ]]; then 
  exec ./$START_SERVICE-server "$@"
elif [[ -f server.bundle.js ]]; then 
  exec env NODE_ENV=production node ./server.bundle.js
else 
  exec npm start
fi
