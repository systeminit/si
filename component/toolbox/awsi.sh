#!/usr/bin/env bash

# If running in Github, we don't have an interactive
# terminal so the commands can't request user input
if [ "$GITHUB_ACTIONS" = "true" ]; then
  terminal="-t"
else
  terminal="-it"
fi

docker run --rm "${terminal}" \
  -v ~/.aws:/root/.aws \
  -v "$(pwd)":/aws \
  -e AWS_ACCESS_KEY_ID="${AWS_ACCESS_KEY_ID}" \
  -e AWS_SECRET_ACCESS_KEY="${AWS_SECRET_ACCESS_KEY}" \
  -e AWS_SESSION_TOKEN="${AWS_SESSION_TOKEN}" \
  systeminit/toolbox:stable "$*"
