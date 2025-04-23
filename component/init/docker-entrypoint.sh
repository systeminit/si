#!/bin/sh

set -o pipefail

if [ -z "$SI_HOSTENV" ] || [ -z "$SI_SERVICE" ]; then
    echo "Error: Required environment variables (SI_HOSTENV and SI_SERVICE) are not set."
    exit 1
fi

DIR="/config"

retry_command() {
    local max_attempts=3
    local attempt=1
    local output=""

    while [ $attempt -le $max_attempts ]; do
        output=$($@ 2>&1)
        if [ $? -eq 0 ]; then
            echo "$output"
            return 0
        fi
        echo "Attempt $attempt failed, retrying..."
        attempt=$((attempt + 1))
        sleep 1
    done
    echo "$output"
    return 1
}

# Get a list of parameters that match the prefix
PARAMETERS=$(retry_command aws ssm describe-parameters --query "Parameters[?starts_with(Name, '$SI_HOSTENV')]" --output json)
if [ -z "$PARAMETERS" ]; then
    echo "Warning: No parameters found with the specified prefix."
fi

# Loop through each parameter
for parameter_info in $(echo "$PARAMETERS" | jq -c '.[]'); do
    parameter_name=$(echo "$parameter_info" | jq -r '.Name')
    env_var_name=$(echo "$parameter_info" | jq -r '.Description')

    if [ -n "$env_var_name" ] && [ "$env_var_name" != "null" ]; then
        # Get the parameter value
        resource_value=$(retry_command aws ssm get-parameter --name "$parameter_name" --query "Parameter.Value" --output json | tr -d '"')

        # Export the environment variable
        export "${env_var_name}=${resource_value}"
        echo "Exported Parameter: $env_var_name"
    fi
done


# Get a list of secrets that match the prefix
SECRETS=$(retry_command aws secretsmanager list-secrets --query "SecretList[?starts_with(Name, '$SI_HOSTENV')].Name" --output json)
if [ -z "$SECRETS" ]; then
    echo "Warning: No secrets found with the specified prefix."
fi

# Loop through each secret
for secret_name in $(echo "$SECRETS" | jq -r '.[]'); do
    env_var_name=$(retry_command aws secretsmanager describe-secret --secret-id "$secret_name" --query "Description" --output json | tr -d '"')

    if [ -n "$env_var_name" ] && [ "$env_var_name" != "null" ]; then
        resource_value=$(retry_command aws secretsmanager get-secret-value --secret-id "$secret_name" --query "SecretString" --output json | tr -d '"')
        export "${env_var_name}=${resource_value}"
        echo "Exported Secret: $env_var_name"
    fi
done

# Set some basic machine-specific env vars
TOKEN=`curl -X PUT "http://169.254.169.254/latest/api/token" -H "X-aws-ec2-metadata-token-ttl-seconds: 21600"`
export SI_HOSTNAME=$(curl -H "X-aws-ec2-metadata-token: $TOKEN" http://169.254.169.254/latest/meta-data/hostname)
export SI_INSTANCE_ID=$(curl -H "X-aws-ec2-metadata-token: $TOKEN" http://169.254.169.254/latest/meta-data/instance-id)

for file_path in "$DIR"/*; do
  file_name=$(basename "$file_path")
  echo "Updating $file_name"
  for env_var in $(env | grep '^SI_' | awk -F= '{print $1}'); do
      sed -i "s/\$${env_var}/$(eval echo \"\$$env_var\" | sed 's/[\/&]/\\&/g')/g" "$file_path"
  done
done

mkdir -p /service
cp $DIR/service.toml /service/$SI_SERVICE.toml
