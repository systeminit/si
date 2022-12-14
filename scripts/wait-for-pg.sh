#!/usr/bin/env sh

echo -n "Trying PG"
while :; do
  echo -n "."
  pg_isready -h localhost --timeout=1 -U si 2>&1 >/dev/null
  if [ "$?" = "0" ]; then
    break
  fi
done
echo "ready"
