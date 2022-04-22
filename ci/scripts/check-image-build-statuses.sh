#!/usr/bin/env bash

echo "::group::ENV"
echo "CHECK_CHANGES_STATUS='${CHECK_CHANGES_STATUS}'"
echo "NATS_RUN='${NATS_RUN}'"
echo "OTELCOL_RUN='${OTELCOL_RUN}'"
echo "POSTGRES_RUN='${POSTGRES_RUN}'"
echo "SDF_RUN='${SDF_RUN}'"
echo "VERITECH_RUN='${VERITECH_RUN}'"
echo "WEB_RUN='${WEB_RUN}'"
echo "NATS_STATUS='${NATS_STATUS}'"
echo "OTELCOL_STATUS='${OTELCOL_STATUS}'"
echo "POSTGRES_STATUS='${POSTGRES_STATUS}'"
echo "SDF_STATUS='${SDF_STATUS}'"
echo "VERITECH_STATUS='${VERITECH_STATUS}'"
echo "WEB_STATUS='${WEB_STATUS}'"
echo "::endgroup::"

echo "::group::Check Changes Status"
set -x
echo "CHECK_CHANGES_STATUS: ${CHECK_CHANGES_STATUS}"
if [ "${CHECK_CHANGES_STATUS}" != "success" ]; then
  exit 2
fi
set +x
echo "::endgroup::"

EXIT_STATUS=0

if [ "${NATS_RUN}" == "true" ]; then
  echo "::group::NATS build check"
  set -x
  if [ "${NATS_STATUS}" != "success" ]; then
    EXIT_STATUS=1
  fi
  set +x
  echo "::endgroup::"
fi

if [ "${OTELCOL_RUN}" == "true" ]; then
  echo "::group::OTELCOL build check"
  set -x
  if [ "${OTELCOL_STATUS}" != "success" ]; then
    EXIT_STATUS=1
  fi
  set +x
  echo "::endgroup::"
fi

if [ "${POSTGRES_RUN}" == "true" ]; then
  echo "::group::Postgres build check"
  set -x
  if [ "${POSTGRES_STATUS}" != "success" ]; then
    EXIT_STATUS=1
  fi
  set +x
  echo "::endgroup::"
fi

if [ "${SDF_RUN}" == "true" ]; then
  echo "::group::SDF build check"
  set -x
  if [ "${SDF_STATUS}" != "success" ]; then
    EXIT_STATUS=1
  fi
  set +x
  echo "::endgroup::"
fi

if [ "${VERITECH_RUN}" == "true" ]; then
  echo "::group::Veritech build check"
  set -x
  if [ "${VERITECH_STATUS}" != "success" ]; then
    EXIT_STATUS=1
  fi
  set +x
  echo "::endgroup::"
fi

if [ "${WEB_RUN}" == "true" ]; then
  echo "::group::Web build check"
  set -x
  if [ "${WEB_STATUS}" != "success" ]; then
    EXIT_STATUS=1
  fi
  set +x
  echo "::endgroup::"
fi

exit ${EXIT_STATUS}
