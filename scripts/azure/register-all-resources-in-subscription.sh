  #!/usr/bin/env bash
  # azure-register-providers.sh
  # Script to register Azure resource providers
  # Before running this script ensure that you are logged into Azure and have run az login.

  # Suppress Python warnings
  export PYTHONWARNINGS="ignore::FutureWarning"

  # Colors for output
  RED='\033[0;31m'
  GREEN='\033[0;32m'
  YELLOW='\033[1;33m'
  BLUE='\033[0;34m'
  NC='\033[0m'

  echo "${BLUE}================================${NC}"
  echo "${BLUE}Azure Resource Provider Registration${NC}"
  echo "${BLUE}================================${NC}"
  echo ""

  # Check if Azure CLI is installed
  if ! command -v az &> /dev/null; then
      echo "${RED}Error: Azure CLI is not installed${NC}"
      echo "Please install it from: https://docs.microsoft.com/en-us/cli/azure/install-azure-cli"
      exit 1
  fi

  # Check if logged in to Azure
  echo "${YELLOW}Checking Azure login status...${NC}"
  if ! az account show &> /dev/null; then
      echo "${RED}Error: Not logged in to Azure${NC}"
      echo "Please run: az login"
      exit 1
  fi

  # Show current subscription
  echo ""
  echo "${GREEN}Current subscription:${NC}"
  az account show --query "{Name:name, ID:id, State:state}" -o table

  # Ask if user wants to switch subscriptions
  echo ""
  echo "${YELLOW}Available subscriptions:${NC}"
  az account list --query "[].{Name:name, ID:id, State:state}" -o table

  echo ""
  echo "${BLUE}Do you want to switch subscriptions? (y/n)${NC}"
  read -r switch_sub

  if [[ "$switch_sub" =~ ^[Yy]$ ]]; then
      echo "${BLUE}Enter subscription name or ID:${NC}"
      read -r sub_input
      az account set --subscription "$sub_input"
      echo "${GREEN}Switched to subscription: $sub_input${NC}"
      echo ""
  fi

  # Fetch and register all providers
  echo ""
  echo "${YELLOW}Fetching all providers...${NC}"
  all_providers=$(az provider list --query "[].namespace" -o tsv)

  if [ -z "$all_providers" ]; then
      echo "${RED}Error: Failed to fetch providers${NC}"
      exit 1
  fi

  total=$(echo "$all_providers" | wc -l | tr -d ' ')
  echo "${YELLOW}Found $total providers. Starting registration...${NC}"
  echo ""

  # Register each provider in parallel
  while IFS= read -r provider; do
      az provider register --namespace "$provider" &
  done <<< "$all_providers"

  echo ""
  echo "${GREEN}================================${NC}"
  echo "${GREEN}All $total provider registrations have been initiated in the background.${NC}"
  echo "${GREEN}This process will continue even after this script exits.${NC}"
  echo "${GREEN}================================${NC}"
  echo ""
  echo "${BLUE}To check registration status later, run:${NC}"
  echo "az provider list --query \"[?registrationState=='Registered'].namespace\" -o table"
