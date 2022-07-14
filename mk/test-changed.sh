#!/usr/bin/env bash

BEFORE_SHA=${1:-"main"}
AFTER_SHA=${2:-"HEAD"}
SKIP_LINT=$SKIP_LINT

set -eu

CHANGED_COMPONENTS=$(git --no-pager diff --name-only $BEFORE_SHA...$AFTER_SHA | xargs dirname | grep '^app/\|lib/\|bin/' | awk -F"/" '{print $1 "/" $2 }' | sort -u)

echo "::group::Changed Components"
echo $CHANGED_COMPONENTS
echo "::endgroup::"

if [ -z "$SKIP_LINT" ]; then
  echo "::group::Lint"
  while IFS= read -r line; do
    if [[ -f "$line/Makefile" ]]; then
      echo "lint//$line"
    fi
  done <<< "$CHANGED_COMPONENTS" | xargs make CI=true CI_FROM_REF=$BEFORE_SHA CI_TO_REF=$AFTER_SHA
  echo "::endgroup::"
fi

echo "::group::Test"
while IFS= read -r line; do
  if [[ -f "$line/Makefile" ]]; then
    echo "test//$line"
  fi
done <<< "$CHANGED_COMPONENTS" | xargs make CI=true CI_FROM_REF=$BEFORE_SHA CI_TO_REF=$AFTER_SHA
echo "::endgroup::"

