export AWS_DEFAULT_REGION=us-east-1
eval $(aws lambda get-function-configuration --function-name $1 | jq -r '.Environment.Variables | to_entries[] | "export " + .key + "=" + .value')
