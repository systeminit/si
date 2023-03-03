#!/usr/bin/env bash
set -eu

BEFORE_SHA="${1:-"main"}"
AFTER_SHA="${2:-"HEAD"}"

changed_files() {
  git --no-pager diff --name-only "$BEFORE_SHA...$AFTER_SHA"
}

CHANGED_COMPONENTS="$(
  changed_files \
    | xargs dirname \
    | grep '^app/\|lib/\|bin/' \
    | awk -F"/" '{print $1 "/" $2 }' \
    | sort -u
)"

changed_files_contains_global_rust_config_for_test() {
  changed_files | grep -q -E '^(Cargo\.(toml|lock)|rust-toolchain)$'
}

changed_files_contains_global_rust_config_for_check() {
  changed_files | grep -q -E '^(clippy.toml|rustfmt.toml)$'
}

cargo_workspace_member_paths() {
  local root
  root="$(cargo metadata --offline --locked --no-deps --quiet \
    | jq -r .workspace_root)"

  cargo metadata --offline --locked --no-deps --quiet \
    | jq -r .workspace_members[] \
    | sed -e "s|^.*(path+file://$root/\(.*\))$|\1|g" \
    | sort
}

all_rust_test_targets() {
  cargo_workspace_member_paths | sed 's|^|test//|'
}

all_rust_check_targets() {
  cargo_workspace_member_paths | sed 's|^|check//|'
}

echo "::group::Changed Components"
echo "$CHANGED_COMPONENTS" | tr '[:space:]' '\n'
echo "::endgroup::"

if [ -z "${SKIP_CHECK:-}" ]; then
  check_targets="$(while IFS= read -r line; do
    if [[ -f "$line/Makefile" ]]; then
      echo "check//$line"
    fi
  done <<<"$CHANGED_COMPONENTS")"
  if changed_files_contains_global_rust_config_for_check; then
    check_targets="$check_targets $(all_rust_check_targets)"
  fi

  if [[ -n "$check_targets" ]]; then
    echo "::group::make $check_targets"
    set -x
    make CI=true "CI_FROM_REF=$BEFORE_SHA" "CI_TO_REF=$AFTER_SHA" $check_targets
    set +x
    echo "::endgroup::"
  fi
fi

test_targets="$(while IFS= read -r line; do
  if [[ -f "$line/Makefile" ]]; then
    echo "test//$line"
  fi
done <<<"$CHANGED_COMPONENTS")"
# If a global Rust config file has changed then we want to test all Rust
# components in the Cargo workspace
if changed_files_contains_global_rust_config_for_test; then
  test_targets="$test_targets $(all_rust_test_targets)"
fi

if [[ -n "$test_targets" ]]; then
  echo "::group::make $test_targets"
  set -x
  make CI=true "CI_FROM_REF=$BEFORE_SHA" "CI_TO_REF=$AFTER_SHA" $test_targets
  set +x
  echo "::endgroup::"
fi
