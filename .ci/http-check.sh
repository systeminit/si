#!/bin/bash

# Usage: http-check.sh <host> <port> <path> <timeout>
# Example: http-check.sh localhost 8080 / 180

# Input arguments with defaults
HOST="${1:-localhost}"
PORT="${2:-8080}"
URL_PATH="${3:-/}"
TIMEOUT="${4:-180}"

FULL_URL="http://${HOST}:${PORT}${URL_PATH}"

echo "⏳ Waiting for ${FULL_URL} to return HTTP 200 (timeout: ${TIMEOUT}s)..."

start_time=$(date +%s)

while true; do
  # Run curl and capture both the HTTP status and exit code
  http_code=$(curl -s -o /dev/null -w "%{http_code}" "$FULL_URL")
  curl_exit_code=$?

  current_time=$(date +%s)
  elapsed=$((current_time - start_time))

  if [[ "$curl_exit_code" -eq 0 && "$http_code" -eq 200 ]]; then
    echo "✅ $FULL_URL responded with 200 OK"
    break
  else
    echo "🔁 Attempt failed (HTTP ${http_code}, curl exit ${curl_exit_code}) after ${elapsed}s — retrying..."
  fi

  if [ "$elapsed" -ge "$TIMEOUT" ]; then
    echo "❌ Timeout: $FULL_URL did not return 200 within ${TIMEOUT}s"
    exit 1
  fi

  sleep 5
done