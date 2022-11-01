#!/bin/bash
# Production variable set
AWS=/usr/bin/aws
JQ=/bin/jq
BASE64=/usr/bin/base64
honeycomb_env=/etc/honeycomb_env
decryption_file=/etc/dev.decryption.key
encryption_file=/etc/dev.encryption.key
jwt_secret_file=/etc/jwt_secret_key.bin


# Development variable set
#AWS=aws
#JQ=jq
#BASE64=base64
# The following declarations point to the stdout file descriptor,  making the system print out the values
#honeycomb_env=/dev/fd/1
#decryption_file=/dev/fd/1
#encryption_file=/dev/fd/1
#jwt_secret_file=/dev/fd/1


honeycomb=$($AWS secretsmanager get-secret-value --secret-id staging/honeycomb --region us-east-2 \
  | $JQ --raw-output '.SecretString')

token=$(echo "$honeycomb" | $JQ --raw-output '.token')
dataset=$(echo "$honeycomb" | $JQ --raw-output '.dataset')

echo "HONEYCOMB_TOKEN=$token" > $honeycomb_env
echo "HONEYCOMB_DATASET=$dataset" >> $honeycomb_env

keys=$($AWS secretsmanager get-secret-value --secret-id staging/keys --region us-east-2 \
  | $JQ --raw-output '.SecretString')

echo "$keys" | $JQ --raw-output '.decryption' | $BASE64 -d > $decryption_file
echo "$keys" | $JQ --raw-output '.encryption' | $BASE64 -d  > $encryption_file
echo "$keys" | $JQ --raw-output '.jwt_secret_key' | $BASE64 -d > $jwt_secret_file
