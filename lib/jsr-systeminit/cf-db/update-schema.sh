#!/usr/bin/env bash

set -o errexit
set -o nounset
set -o pipefail
if [[ "${TRACE-0}" == "1" ]]; then set -o xtrace; fi
pushd "$(dirname ${BASH_SOURCE[0]})"
mkdir -p ./cloudformation-schema
pushd ./cloudformation-schema
rm -f *.json
wget https://schema.cloudformation.us-east-1.amazonaws.com/CloudformationSchema.zip
unzip CloudformationSchema.zip
rm CloudformationSchema.zip*
perl -pi -e 's/\"resource-schema.json\#/\"#/g' *.json
jq 'del(.definitions.SseSpecification)' aws-ec2-verifiedaccesstrustprovider.json > tmp.json && mv tmp.json aws-ec2-verifiedaccesstrustprovider.json
deno fmt *.json
popd
rm -f provider.definition.schema.v1.json
wget https://raw.githubusercontent.com/aws-cloudformation/cloudformation-cli/refs/heads/master/src/rpdk/core/data/schema/provider.definition.schema.v1.json
jq -c -n '[inputs]' cloudformation-schema/aws*.json > cf-schema.json
popd
