#!/bin/bash

role=$1

TOKEN=`curl -s -X PUT "http://169.254.169.254/latest/api/token" -H "X-aws-ec2-metadata-token-ttl-seconds: 21600"` \
VALUES=$(curl -s -H "X-aws-ec2-metadata-token: $TOKEN" -v http://169.254.169.254/latest/meta-data/iam/security-credentials/$role)
echo "export AWS_ACCESS_KEY_ID=$(echo $VALUES | jq -r '.AccessKeyId')"
echo "export AWS_SECRET_ACCESS_KEY=$(echo $VALUES | jq -r '.SecretAccessKey')"
echo "export AWS_SESSION_TOKEN=$(echo $VALUES | jq -r '.Token')"
