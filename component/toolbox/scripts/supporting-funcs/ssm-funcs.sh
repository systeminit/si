#!/bin/bash

# shellcheck disable=SC2034

# Define the SSM documents to execute the functions
upgrade_check_script="si-check-node-upgrade"
service_maintenance_script="si-service-maintenance"
service_state_script="si-service-state"
sdf_migrate_script="si-migrate-sdf"

# Function to start SSM session
start_and_track_ssm_session() {

  instance_id=$1
  script=$2
  results_directory=$3
  params=$4

  output=$(aws ssm send-command --instance-ids "$instance_id" --document-name "$script" --parameters "$params" 2>&1)

  status=$?

  if [ $status -ne 0 ]; then
    output="{\"instance_id\": \"$instance_id\", \"status\": \"error\", \"message\": \"$output\"}"
    echo "$output" >"$results_directory/$instance_id.json"
    return
  fi

  command_id=$(echo "$output" | jq -r '.Command.CommandId')

  # Poll for command status with a timeout of 60 seconds
  timeout=60
  elapsed=0
  interval=1

  while [ $elapsed -lt $timeout ]; do
    status=$(check_ssm_command_status)

    if [ "$status" == "Success" ] || [ "$status" == "Failed" ] || [ "$status" == "TimedOut" ] || [ "$status" == "Cancelled" ]; then
      break
    fi

    sleep $interval
    elapsed=$((elapsed + interval))
  done

  # Check if command was successful
  if [ "$status" == "Success" ]; then
    # Get the output
    output=$(aws ssm get-command-invocation \
      --command-id "$command_id" \
      --instance-id "$instance_id" \
      | jq -r '.StandardOutputContent')
    echo "$output" >"$results_directory/$instance_id.json"
  else
    echo "Command failed with status: $status"
    exit_code=$(aws ssm get-command-invocation \
      --command-id "$command_id" \
      --instance-id "$instance_id" \
      | jq -r '.ResponseCode')

    echo "Exit code: $exit_code"
    echo "Failure message:"
    aws ssm get-command-invocation \
      --command-id "$command_id" \
      --instance-id "$instance_id" \
      | jq -r '.StandardErrorContent'
  fi

}

# Function to start an interactive SSM session with any given instance
start_interactive_ssm_session() {
  instance_id=$1
  name=$2
  aws ssm start-session --target "$instance_id" --document-name AWS-StartInteractiveCommand --parameters \
    "{\"command\": [\"PS1=\\\"\\\\u@\\\\h \\\\e[32m$name\\\\e[0m > \\\" bash -l\"]}"
}

# Function to check command status
check_ssm_command_status() {
  status=$(aws ssm list-command-invocations \
    --command-id "$command_id" \
    --details \
    | jq -r '.CommandInvocations[0].Status')
  echo "$status"
}
