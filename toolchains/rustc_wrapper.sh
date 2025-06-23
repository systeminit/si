#!/bin/sh
# Enhanced wrapper script for rustc that sets up the environment

# Get C compiler and C++ compiler from the arguments
CC="$1"
CXX="$2"
shift 2  # Remove the first two arguments, leaving the rustc command and its args

# Ensure bash is in the PATH
export PATH="/bin:/usr/bin:$PATH"

# Set C/C++ compiler environment variables
export CC
export CXX

# Execute the command with all remaining arguments
exec "$@"
