#!/bin/sh

set -o pipefail

# Check if SI_FUNCTION is set
if [ -z "$SI_FUNCTION" ]; then
    echo "Error: Required environment variable SI_FUNCTION is not set."
    exit 1
fi

# Directory containing Python functions
DIR="/functions"

# Check if the specified function file exists
if [ ! -f "$DIR/$SI_FUNCTION.py" ]; then
    echo "Error: Python function $SI_FUNCTION not found in $DIR."
    exit 1
fi

# Execute the specified Python function
python3 "$DIR/$SI_FUNCTION.py"