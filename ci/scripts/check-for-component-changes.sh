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
PINGA_PATHS=".github/workflows/promote-pinga.yml
  .cargo/**
  Cargo.*
  bin/pinga/**
  lib/config-file/**
  lib/dal/**
  lib/dal-test/**
  lib/pinga-server/**
  lib/si-data-faktory/**
  lib/si-data-nats/**
  lib/si-data-pg/**
  lib/si-settings/**
  lib/si-std/**
  lib/si-test-macros/**
  lib/telemetry-application-rs/**
  lib/telemetry-rs/**
  lib/veritech-client/**
  rust-toolchain
  rustfmt.toml"
SDF_PATHS=".github/workflows/promote-sdf.yml
  .cargo/**
  Cargo.*
  bin/sdf/**
  lib/config-file/**
  lib/dal/**
  lib/dal-test/**
  lib/sdf/**
  lib/si-data-faktory/**
  lib/si-data-nats/**
  lib/si-data-pg/**
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

NATS_CHANGES=false
OTELCOL_CHANGES=false
POSTGRES_CHANGES=false
SDF_CHANGES=false
VERITECH_CHANGES=false
WEB_CHANGES=false
PINGA_CHANGES=false

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
echo "::endgroup::"

echo "::set-output name=component--nats::${NATS_CHANGES}"
echo "::set-output name=component--otelcol::${OTELCOL_CHANGES}"
echo "::set-output name=component--postgres::${POSTGRES_CHANGES}"
echo "::set-output name=bin--sdf::${SDF_CHANGES}"
echo "::set-output name=bin--veritech::${VERITECH_CHANGES}"
echo "::set-output name=app--web::${WEB_CHANGES}"
echo "::set-output name=bin--pinga::${PINGA_CHANGES}"
