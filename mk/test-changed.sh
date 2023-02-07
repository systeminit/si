#!/usr/bin/env bash
set -eu

BEFORE_SHA="${1:-"main"}"
AFTER_SHA="${2:-"HEAD"}"

CHANGED_COMPONENTS="$(
  git --no-pager diff --name-only "$BEFORE_SHA...$AFTER_SHA" \
    | xargs dirname \
    | grep '^app/\|lib/\|bin/\|.' \
    | awk -F"/" '{print $1 "/" $2 }' \
    | sort -u
)"

echo "::group::Changed Components"
echo "$CHANGED_COMPONENTS" | tr '[:space:]' '\n'
echo "::endgroup::"

if [ -z "${SKIP_CHECK:-}" ]; then
  check_targets="$(while IFS= read -r line; do
    if [[ $line = "./" ]]; then
      echo "check"
    elif [[ $line != "./" && -f "$line/Makefile" ]]; then
      echo "check//$line"
    fi
  done <<<"$CHANGED_COMPONENTS")"
  echo "::group::make $check_targets"
  set -x
  make CI=true "CI_FROM_REF=$BEFORE_SHA" "CI_TO_REF=$AFTER_SHA" $check_targets
  set +x
  echo "::endgroup::"
fi

test_targets="$(while IFS= read -r line; do
  if [[ $line = "./" ]]; then
    echo "check"
  elif [[ $line != "./" && -f "$line/Makefile" ]]; then
    echo "check//$line"
  fi
done <<<"$CHANGED_COMPONENTS")"
echo "::group::make $test_targets"
set -x
make CI=true "CI_FROM_REF=$BEFORE_SHA" "CI_TO_REF=$AFTER_SHA" $test_targets
set +x
echo "::endgroup::"
