#!/bin/bash

# Function to get input or use environment variable
get_param_or_env() {
  local param=$1
  local env_var=$2
  local prompt=$3

  if [ -z "$param" ]; then
    if [ -z "${!env_var}" ]; then
      read -p "$prompt: " value
      echo "$value"
    else
      echo "${!env_var}"
    fi
  else
    echo "$param"
  fi
}

await_file_results() {

  results_directory=$1
  required_file_count=$2

  timeout=60             # Timeout in seconds
  start_time=$(date +%s) # Record the start time

  while true; do
    current_time=$(date +%s)
    elapsed_time=$((current_time - start_time))

    if ((elapsed_time > timeout)); then
      echo "Error: Timeout reached waiting for SSM document responses to arrive. Not all files are present."
      exit 1
    fi

    file_count=$(ls "$results_directory" | wc -l)

    if ((file_count >= required_file_count)); then
      break
    fi

    # Wait for a short period before checking again
    sleep 1
  done

}

sassy_selection_check() {
  selection=${1^^}
  if [ "$selection" != "Y" ]; then
    echo "Don't Trust Scott and John? We're friends I promise, exiting"
    exit 1
  fi
}

concat_and_output_json() {

  results_directory=$1
  output_file=$2

  # Check if the directory exists
  if [ -d "$results_directory/" ]; then
    # Aggregate all the individual json documents into one
    cat $results_directory/* | jq -s '.' >>$results_directory/$output_file
    cat $results_directory/$output_file | jq
    echo "----------------------------------------"
    echo "Results can be found within $results_directory"
  else
    echo "Results Directory $results_directory does not exist."
    exit 1
  fi
  echo "----------------------------------------"

}