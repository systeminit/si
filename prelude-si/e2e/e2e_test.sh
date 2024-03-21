#!/bin/bash

# TODO(johnrwatson): In theory we should be able to run this task for any e2e test we wish to execute

set -eo pipefail

# TODO(johnrwatson): We need to port this to python or similar, and check for OS-dependencies that are required. i.e.
# cypress and anything else we need for the cypress tests to run successfully, think vars, etc.

output_file=$1  # i.e. output the file ./e2e_test_result.html

echo "-------------------------------------"
echo "Info: Initiating e2e test"
echo "Artifact Version: $(./${git_helper} | jq -r '.artifact_ver')"
echo "Output File: $output_file"
echo "-------------------------------------"