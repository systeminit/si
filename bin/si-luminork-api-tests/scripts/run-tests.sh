#!/bin/bash
# Run Luminork API tests with environment setup

# Stop on first error
set -e

# Help message
function show_help {
  echo "Luminork API Testing Framework"
  echo ""
  echo "Usage: ./scripts/run-tests.sh [options]"
  echo ""
  echo "Options:"
  echo "  -h, --help           Show this help message"
  echo "  -u, --url URL        Set the Luminork API URL (default: http://localhost:5380)"
  echo "  -t, --token TOKEN    Set the auth token (REQUIRED)"
  echo "  -w, --workspace ID   Set the workspace ID (REQUIRED)"
  echo "  --filter PATTERN     Only run tests matching the pattern"
  echo "  --debug              Run the debug script instead of tests"
  echo "  --watch              Run tests in watch mode"
  echo "  --no-check           Skip type checking"
  echo ""
  echo "Example:"
  echo "  ./scripts/run-tests.sh --url http://localhost:5380 --token my_token --workspace my_workspace_id"
  echo ""
  echo "Note: You must provide an auth token and workspace ID either via:"
  echo "  1. Environment variables (LUMINORK_AUTH_TOKEN, LUMINORK_WORKSPACE_ID)"

  echo "  2. Command line arguments (--token, --workspace)"
  echo "  3. Values in a .env file"
}

# Default values
if [ -z "$LUMINORK_API_URL" ]; then
  API_URL="http://localhost:5380"
else
  API_URL="$LUMINORK_API_URL"
fi
AUTH_TOKEN=""
WORKSPACE_ID=""
FILTER=""
DEBUG_MODE=false
WATCH=false
SKIP_CHECK=false

# Script directory
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_DIR="$( cd "$SCRIPT_DIR/.." && pwd )"

# Create an empty .env file if it doesn't exist
if [ ! -f "$PROJECT_DIR/.env" ]; then
  if [ -f "$PROJECT_DIR/.env.example" ]; then
    echo "Creating .env file from .env.example..."
    cp "$PROJECT_DIR/.env.example" "$PROJECT_DIR/.env"
  else
    echo "Creating empty .env file..."
    touch "$PROJECT_DIR/.env"
  fi
fi

# Parse command line arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    -h|--help)
      show_help
      exit 0
      ;;
    -u|--url)
      API_URL="$2"
      shift 2
      ;;
    -t|--token)
      AUTH_TOKEN="$2"
      shift 2
      ;;
    -w|--workspace)
      WORKSPACE_ID="$2"
      shift 2
      ;;
    --filter)
      FILTER="$2"
      shift 2
      ;;
    --debug)
      DEBUG_MODE=true
      shift
      ;;
    --watch)
      WATCH=true
      shift
      ;;
    --no-check)
      SKIP_CHECK=true
      shift
      ;;
    *)
      echo "Unknown option: $1"
      show_help
      exit 1
      ;;
  esac
done

# Set environment variables for the tests
export LUMINORK_API_URL="$API_URL"
export API_URL="$API_URL"

if [ -n "$AUTH_TOKEN" ]; then
  export LUMINORK_AUTH_TOKEN="$AUTH_TOKEN"
  export AUTH_TOKEN="$AUTH_TOKEN"
fi

if [ -n "$WORKSPACE_ID" ]; then
  export LUMINORK_WORKSPACE_ID="$WORKSPACE_ID"
  export WORKSPACE_ID="$WORKSPACE_ID"
fi

# Check for required parameters - error if missing
if [ -z "$AUTH_TOKEN" ] && [ -z "$(grep -E '^(LUMINORK_AUTH_TOKEN|AUTH_TOKEN)=' "$PROJECT_DIR/.env" 2>/dev/null)" ]; then
  echo -e "\033[31mERROR: Auth token is required but not provided\033[0m"
  echo "Please provide an auth token using --token or set LUMINORK_AUTH_TOKEN in environment variables or .env file"
  exit 1
fi

if [ -z "$WORKSPACE_ID" ] && [ -z "$(grep -E '^(LUMINORK_WORKSPACE_ID|WORKSPACE_ID)=' "$PROJECT_DIR/.env" 2>/dev/null)" ]; then
  echo -e "\033[31mERROR: Workspace ID is required but not provided\033[0m"
  echo "Please provide a workspace ID using --workspace or set LUMINORK_WORKSPACE_ID in environment variables or .env file"
  exit 1
fi

# Run the tests or debug script
if [ "$DEBUG_MODE" = true ]; then
  echo -e "\033[1mRunning debug script...\033[0m"
  echo "API URL: $API_URL"
  if [ -n "$WORKSPACE_ID" ]; then
    echo "Workspace ID: $WORKSPACE_ID"
  else
    echo "Workspace ID: <from .env file>"
  fi

  chmod +x "$PROJECT_DIR/debug-api.ts"
  deno run --allow-env --allow-net --allow-read "$PROJECT_DIR/debug-api.ts"
else
  # Build the command for tests
  COMMAND="deno test --allow-env --allow-net --allow-read"

  if [ "$WATCH" = true ]; then
    COMMAND="$COMMAND --watch"
  fi

  if [ "$SKIP_CHECK" = true ]; then
    COMMAND="$COMMAND --no-check"
  fi

  if [ -n "$FILTER" ]; then
    COMMAND="$COMMAND --filter \"$FILTER\""
  fi

  echo -e "\033[1mRunning Luminork API tests...\033[0m"
  echo "API URL: $API_URL"
  if [ -n "$WORKSPACE_ID" ]; then
    echo "Workspace ID: $WORKSPACE_ID"
  else
    echo "Workspace ID: <from .env file>"
  fi

  eval "$COMMAND"
fi
