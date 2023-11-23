#!/bin/bash

set -eo pipefail

# Add an example variables file
cat << HEREDOC > /tmp/variables.txt
CONFIGURATION_MANAGEMENT_BRANCH="main"
CONFIGURATION_MANAGEMENT_TOOL="shell"
AUTOMATED="true"
HEREDOC

curl -s https://raw.githubusercontent.com/systeminit/si/main/component/firecracker-base/orchestrate-install.sh | bash