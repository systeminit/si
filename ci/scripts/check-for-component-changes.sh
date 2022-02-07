#!/usr/bin/env bash

CHANGED_PATHS="$(git diff --name-only origin/main)"

COMMON_PATHS=".github/workflows/promote-image.yml
  .github/workflows/build-docker-image.yml
  .github/workflows/build-docker-images.yml"
NATS_PATHS=".github/workflows/promote-nats.yml
  component/nats/**"
OTELCOL_PATHS=".github/workflows/promote-otelcol.yml
  component/otelcol/**"
POSTGRES_PATHS=".github/workflows/promote-postgres.yml
  component/postgres/**"
SDF_CHANGES=".github/workflows/promote-sdf.yml
  .cargo/**
  Cargo.*
  bin/sdf/**
  lib/config-file/**
  lib/dal/**
  lib/sdf/**
  lib/si-data/**
  lib/si-settings/**
  lib/telemetry-rs/**
  rust-toolchain
  rustfmt.toml"
VERITECH_PATHS=".github/workflows/promote-veritech.yml
  .cargo/**
  Cargo.*
  bin/veritech/**
  lib/config-file/**
  lib/cyclone/**
  lib/si-data/**
  lib/si-settings/**
  lib/telemetry-rs/**
  lib/veritech/**
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
echo "::endgroup::"

echo "::set-output name=nats::${NATS_CHANGES}"
echo "::set-output name=otelcol::${OTELCOL_CHANGES}"
echo "::set-output name=postgres::${POSTGRES_CHANGES}"
echo "::set-output name=sdf::${SDF_CHANGES}"
echo "::set-output name=veritech::${VERITECH_CHANGES}"
echo "::set-output name=web::${WEB_CHANGES}"
