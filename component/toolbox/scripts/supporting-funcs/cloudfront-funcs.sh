#!/bin/bash

# Function to list CloudFront Distributions by ID
list_distributions() {
  filter=$1
  if [[ "${filter,,}" == "all" || -z "${filter}" ]]; then
    # List all CloudFront distributions
    aws cloudfront list-distributions --query 'DistributionList.Items[*].[Id,DomainName,Status,Comment]' --output text
  else
    # Filter CloudFront distributions based on the provided filter
    aws cloudfront list-distributions --query 'DistributionList.Items[*].[Id,DomainName,Status,Comment]' --output text | grep -E "${filter}"
  fi
}

# Function to start an interactive SSM session with any given instance
track_invalidations() {
  distribution_id=$1
  time=$2

  # Get the end time by adding the current time and the specified time in seconds
  end_time=$((SECONDS + time))

  echo "Checking CloudFront invalidations for distribution ID: $distribution_id"

  # Loop until the time has expired
  while [ $SECONDS -lt $end_time ]; do
    # Get the list of invalidations that are still in progress
    active_invalidations=$(aws cloudfront list-invalidations --distribution-id "$distribution_id" --query "InvalidationList.Items[?Status=='InProgress'].Id" --output text)

    if [[ -z "$active_invalidations" || "$active_invalidations" == "None" ]]; then
      return
    else
      echo "$distribution_id: Active invalidation $active_invalidations | [current time: $SECONDS / timeout: $end_time]"
    fi

    # Sleep for 1 second before polling again
    sleep 1
  done

  echo "Time expired. There may still be active invalidations."
  exit 1
}
