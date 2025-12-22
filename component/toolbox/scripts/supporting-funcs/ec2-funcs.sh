#!/bin/bash

# Function to list EC2 instances with their Name tag, either all or filtered
list_instances() {
  filter=$1
  if [[ "${filter,,}" == "all" || -z "${filter}" ]]; then
    # shellcheck disable=SC2016
    aws ec2 describe-instances --query 'Reservations[*].Instances[?State.Name==`running`].[Tags[?Key==`Name`].Value | [0],InstanceId,InstanceType,PrivateIpAddress]' --output text | grep -E 'sdf|veritech|pinga|rebaser|forklift|edda|luminork'
  elif [[ "${filter,,}" != "all" ]]; then
    # shellcheck disable=SC2016
    aws ec2 describe-instances --query 'Reservations[*].Instances[?State.Name==`running`].[Tags[?Key==`Name`].Value | [0],InstanceId,InstanceType,PrivateIpAddress]' --output text | grep -E "${filter}"
  fi
}
