#!/bin/bash
credential=$(/usr/bin/aws secretsmanager get-secret-value --secret-id dockerhub_readonly --region us-east-2 \
  | /bin/jq --raw-output '.SecretString')

user=$(echo "$credential" | /bin/jq --raw-output '.User')
pass=$(echo "$credential" | /bin/jq --raw-output '.Password')

echo "$pass" | /usr/bin/docker login --username "$user" --password-stdin
