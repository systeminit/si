#!/usr/bin/env bash
set -e

MAX_RETRIES=60
RETRY_COUNT=0

function readiness-check {
    while true; do
        # NOTE(nick): declare VALUE as local first so that we capture the subshell output.
        # Then, ensure that we allow curl to fail as needed.
        local VALUE
        set +e
        VALUE=$(curl -X GET 'https://localhost/api/demo' -k -s -m 1 | jq -r ".isCool")
        set -e

        if [ "$VALUE" = "true" ]; then
            return
        fi

        RETRY_COUNT=$(( RETRY_COUNT + 1 ))
        if [[ $RETRY_COUNT -ge $MAX_RETRIES ]]; then
            echo "hit max retry count: $MAX_RETRIES"
            exit 1
        fi

        echo "sleeping and retrying..."
        sleep 1
    done
}

function check-binary {
    if ! command -v "$1" >/dev/null 2>&1; then
        echo "command not found in PATH: $1"
        exit 1
    fi
}

echo "::group::Readiness check"
check-binary curl
check-binary jq
readiness-check
echo "SI is ready!"
echo "::endgroup::"
