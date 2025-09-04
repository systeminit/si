#!/bin/bash
# Wrapper script to run the Deno MCP server for testing with proper environment

# Change to the directory where the script is located
cd "$(dirname "$0")"

# Load .env file from parent directory and run Deno with those variables
if [ -f ../.env ]; then
    echo "Loading environment from .env file..." >&2
    # Read variables for display purposes only
    eval $(grep -v '^#' ../.env)
    echo "SI_API_TOKEN is: ${SI_API_TOKEN:0:10}..." >&2
    echo "SI_BASE_URL is: $SI_BASE_URL" >&2
    
    # Export environment variables from .env file
    echo "Starting Deno MCP server..." >&2
    export $(grep -v '^#' ../.env | xargs)
    exec deno run -A ../main.ts stdio
else
    # Run without .env file
    echo "Starting Deno MCP server..." >&2
    exec deno run -A ../main.ts stdio
fi