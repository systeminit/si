#!/usr/bin/env bash
set -eu

if [[ -n "${DEBUG:-}" ]]; then set -v; fi
if [[ -n "${TRACE:-}" ]]; then set -xv; fi

# If running in Github, we don't have an interactive
# terminal so the commands can't request user input
if [[ "${GITHUB_ACTIONS:-}" = "true" ]]; then
  terminal="-t"
else
  terminal="-it"
fi

args=(
  --rm
  "${terminal}"
  -v ~/.aws:/root/.aws
  -v "$(pwd)":/aws
  -e AWS_ACCESS_KEY_ID="${AWS_ACCESS_KEY_ID:-}"
  -e AWS_SECRET_ACCESS_KEY="${AWS_SECRET_ACCESS_KEY:-}"
  -e AWS_SESSION_TOKEN="${AWS_SESSION_TOKEN:-}"
)

if [[ -n "${AWS_PROFILE:-}" ]]; then
  args+=(-e AWS_PROFILE="${AWS_PROFILE}")
fi

if [[ -n "${AWS_REGION:-}" ]]; then
  args+=(-e AWS_REGION="${AWS_REGION}")
fi

docker run "${args[@]}" systeminit/toolbox:stable "$*"
