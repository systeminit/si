#!/bin/sh

# Configuration Environemnt Variables
# This files is copied from the Dockerfile
. /etc/service/config-couchbase/env

# Log all subsequent commands to logfile. FD 3 is now the console
# for things we want to show up in "docker logs".
LOGFILE=/opt/couchbase/var/lib/couchbase/logs/container-startup.log
exec 3>&1 1>>${LOGFILE} 2>&1

CONFIG_DONE_FILE=/opt/couchbase/var/lib/couchbase/container-configured
config_done() {
  touch ${CONFIG_DONE_FILE}
  echo "Couchbase Admin UI: http://localhost:8091" \
     "\nLogin credentials: si / bugbear" | tee /dev/fd/3
  echo "Stopping config-couchbase service"
  sv stop /etc/service/config-couchbase
}

if [ -e ${CONFIG_DONE_FILE} ]; then
  echo "Container previously configured." | tee /dev/fd/3
  config_done
else
  echo "Configuring Couchbase Server.  Please wait (~60 sec)..." | tee /dev/fd/3
fi

export PATH=/opt/couchbase/bin:${PATH}

wait_for_uri() {
  uri=$1
  expected=$2
  echo "Waiting for $uri to be available..."
  while true; do
    status=$(curl -s -w "%{http_code}" -o /dev/null $uri)
    if [ "x$status" = "x$expected" ]; then
      break
    fi
    echo "$uri not up yet, waiting 2 seconds..."
    sleep 2
  done
  echo "$uri ready, continuing"
}

panic() {
  cat <<EOF 1>&3

@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@
Error during initial configuration - aborting container
Here's the log of the configuration attempt:
@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@
EOF
  cat $LOGFILE 1>&3
  echo 1>&3
  kill -HUP 1
  exit
}

couchbase_cli_check() {
  couchbase-cli $* || {
    echo Previous couchbase-cli command returned error code $?
    panic
  }
}

curl_check() {
  status=$(curl -sS -w "%{http_code}" -o /tmp/curl.txt $*)
  cat /tmp/curl.txt
  rm /tmp/curl.txt
  if [ "$status" -lt 200 -o "$status" -ge 300 ]; then
    echo
    echo Previous curl command returned HTTP status $status
    panic
  fi
}

wait_for_uri http://127.0.0.1:8091/ui/index.html 200

echo "Setting memory quotas with curl:"
curl_check http://127.0.0.1:8091/pools/default -d memoryQuota=${DATA_RAM_QUOTA:-2048} -d indexMemoryQuota=${INDEX_RAM_QUOTA:-512} -d ftsMemoryQuota=${FULL_TEXT_RAM_QUOTA:-512}
echo

echo "Configuring Services with curl:"
curl_check http://127.0.0.1:8091/node/controller/setupServices -d services=kv%2Cn1ql%2Cindex%2Cfts
echo

echo "Setting up credentials with curl:"
curl_check http://127.0.0.1:8091/settings/web -d port=8091 -d username=si -d password=bugbear
echo

echo "Enabling memory-optimized indexes with curl:"
curl_check -u si:bugbear -X POST http://127.0.0.1:8091/settings/indexes -d 'storageMode=memory_optimized'
echo


echo

wait_for_uri http://127.0.0.1:8094/api/index 403

echo "Create buckets with curl:"
curl_check -u si:bugbear -X POST http://127.0.0.1:8091/pools/default/buckets -d name=si -d ramQuotaMB=${BUCKET_SI_RAM_QUOTA:-512} -d bucketType=couchbase -d authType=sasl 
curl_check -u si:bugbear -X POST http://127.0.0.1:8091/pools/default/buckets -d name=si_integration -d ramQuotaMB=${BUCKET_SI_INTEGRATION_RAM_QUOTA:-512} -d bucketType=couchbase -d authType=sasl -d flushEnabled=1

echo "Create primary indexes:"
cbq -u si -p bugbear -script 'create primary index `si_integration_primary_index` on `si_integration`'
cbq -u si -p bugbear -script 'create primary index `si_primary_index` on `si`'


# echo "Creating hotels FTS index with curl:"
# curl_check -u si:bugbear -X PUT http://127.0.0.1:8094/api/index/hotels -H Content-Type:application/json -d @/opt/couchbase/create-index.json
# rm /opt/couchbase/create-index.json
# echo

# echo "Creating RBAC 'admin' user on travel-sample bucket"
# couchbase_cli_check user-manage --set \
#   --rbac-username admin --rbac-password password \
#   --roles 'bucket_full_access[travel-sample]' --auth-domain local \
#   -c 127.0.0.1 -u Administrator -p password
# echo

echo "Configuration completed!" | tee /dev/fd/3

config_done
