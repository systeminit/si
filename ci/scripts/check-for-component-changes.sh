#!/usr/bin/env bash
set -u

CHANGED_PATHS="$(git diff --name-only origin/main)"

COMMON_PATHS=".github/workflows/promote-image.yml
  .github/workflows/build-docker-image.yml
  .github/workflows/build-docker-images.yml
  ci/scripts/check-for-component-changes.sh"
NATS_PATHS=".github/workflows/promote-nats.yml
  component/nats/**"
OTELCOL_PATHS=".github/workflows/promote-otelcol.yml
  component/otelcol/**"
POSTGRES_PATHS=".github/workflows/promote-postgres.yml
  component/postgres/**"
SDF_PATHS=".github/workflows/promote-sdf.yml
  .cargo/**
  Cargo.*
  bin/sdf/**
  lib/buck2-resources/**
  lib/config-file/**
  lib/council-server/**
  lib/dal/**
  lib/dal-test/**
  lib/module-index-client/**
  lib/sdf-server/**
  lib/si-data-nats/**
  lib/si-data-pg/**
  lib/si-posthog-rs/**
  lib/si-settings/**
  lib/si-std/**
  lib/si-test-macros/**
  lib/telemetry-application-rs/**
  lib/telemetry-rs/**
  lib/veritech-client/**
  rust-toolchain
  rustfmt.toml"
VERITECH_PATHS=".github/workflows/promote-veritech.yml
  .cargo/**
  Cargo.*
  bin/veritech/**
  lib/buck2-resources/**
  lib/config-file/**
  lib/cyclone-core/**
  lib/cyclone-server/**
  lib/si-data-nats/**
  lib/si-settings/**
  lib/si-test-macros/**
  lib/telemetry-application-rs/**
  lib/telemetry-rs/**
  lib/veritech-core/**
  lib/veritech-server/**
  rust-toolchain
  rustfmt.toml"
WEB_PATHS=".github/workflows/promote-web.yml
  .prettierrc.js
  babel.config.js
  app/web/**"
PINGA_PATHS="${SDF_PATHS}
  .github/workflows/promote-pinga.yml
  bin/pinga/**
  lib/buck2-resources/**
  lib/pinga-server/**"
COUNCIL_PATHS="${SDF_PATHS}
  .github/workflows/promote-council.yml
  bin/council/**
  lib/buck2-resources/**
  lib/council-server/**"
MODULE_INDEX_PATHS="
  .github/workflows/promote-module-index.yml
  bin/module-index/**
  lib/buck2-resources/**
  lib/module-index-server/**
  lib/module-index-client/**
"

NATS_CHANGES=false
OTELCOL_CHANGES=false
POSTGRES_CHANGES=false
SDF_CHANGES=false
VERITECH_CHANGES=false
WEB_CHANGES=false
PINGA_CHANGES=false
COUNCIL_CHANGES=false
MODULE_INDEX_CHANGES=false

echo "::group::Changed files"
echo "${CHANGED_PATHS}"
echo "::endgroup::"

echo "::group::Check file paths"
set -x
for changed_path in ${CHANGED_PATHS}; do
  for check_path in ${COMMON_PATHS} ${NATS_PATHS}; do
    if [[ "${changed_path}" == "${check_path}"* ]]; then
      echo "NATS MATCH!"
      NATS_CHANGES=true
    fi
  done
  for check_path in ${COMMON_PATHS} ${OTELCOL_PATHS}; do
    if [[ "${changed_path}" == "${check_path}"* ]]; then
      echo "OTELCOL MATCH!"
      OTELCOL_CHANGES=true
    fi
  done
  for check_path in ${COMMON_PATHS} ${POSTGRES_PATHS}; do
    if [[ "${changed_path}" == "${check_path}"* ]]; then
      echo "Postgres MATCH!"
      POSTGRES_CHANGES=true
    fi
  done
  for check_path in ${COMMON_PATHS} ${SDF_PATHS}; do
    if [[ "${changed_path}" == "${check_path}"* ]]; then
      echo "SDF MATCH!"
      SDF_CHANGES=true
    fi
  done
  for check_path in ${COMMON_PATHS} ${VERITECH_PATHS}; do
    if [[ "${changed_path}" == "${check_path}"* ]]; then
      echo "Veritech MATCH!"
      VERITECH_CHANGES=true
    fi
  done
  for check_path in ${COMMON_PATHS} ${WEB_PATHS}; do
    if [[ "${changed_path}" == "${check_path}"* ]]; then
      echo "Web MATCH!"
      WEB_CHANGES=true
    fi
  done
  for check_path in ${COMMON_PATHS} ${PINGA_PATHS}; do
    if [[ "${changed_path}" == "${check_path}"* ]]; then
      echo "Pinga MATCH!"
      PINGA_CHANGES=true
    fi
  done
  for check_path in ${COMMON_PATHS} ${COUNCIL_PATHS}; do
    if [[ "${changed_path}" == "${check_path}"* ]]; then
      echo "Council MATCH!"
      COUNCIL_CHANGES=true
    fi
  done
  for check_path in ${COMMON_PATHS} ${MODULE_INDEX_PATHS}; do
    if [[ "${changed_path}" == "${check_path}"* ]]; then
      echo "Module-Index MATCH!"
      MODULE_INDEX_CHANGES=true
    fi
  done
done
set +x
echo "::endgroup::"

echo "::group::Changed components"
echo "NATS:     ${NATS_CHANGES}"
echo "OTELCOL:  ${OTELCOL_CHANGES}"
echo "Postgres: ${POSTGRES_CHANGES}"
echo "SDF:      ${SDF_CHANGES}"
echo "Veritech: ${VERITECH_CHANGES}"
echo "Web:      ${WEB_CHANGES}"
echo "Pinga:    ${PINGA_CHANGES}"
echo "Council:    ${COUNCIL_CHANGES}"
echo "Module-Index: ${MODULE_INDEX_CHANGES}"
echo "::endgroup::"

{
  echo "component--nats=${NATS_CHANGES}"
  echo "component--otelcol=${OTELCOL_CHANGES}"
  echo "component--postgres=${POSTGRES_CHANGES}"
  echo "bin--sdf=${SDF_CHANGES}"
  echo "bin--veritech=${VERITECH_CHANGES}"
  echo "app--web=${WEB_CHANGES}"
  echo "bin--pinga=${PINGA_CHANGES}"
  echo "bin--council=${COUNCIL_CHANGES}"
  echo "bin--module-index=${MODULE_INDEX_CHANGES}"
} >>"$GITHUB_OUTPUT"
