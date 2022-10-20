#!/usr/bin/env sh

FILES_TO_REMOVE="$(ls -t /var/log/postgresql/*.log | sed -e '1,3d')"

if [ -n "$FILES_TO_REMOVE" ]; then
  /usr/bin/logger "Deleting old PostgreSQL log files: $FILES_TO_REMOVE"
  rm -f $FILES_TO_REMOVE
fi
