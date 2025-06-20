#!/bin/sh
# Enhanced wrapper script for rustc that sets up the environment
set -x
# Get C compiler and C++ compiler from the arguments
CC="$1"
CXX="$2"
SHIMS="$3"
shift 3  # Remove the first two arguments, leaving the rustc command and its args

echo $PATH
echo $(which clang)
# Ensure bash is in the PATH
export PATH="$SHIMS:$PATH"

# Set C/C++ compiler environment variables
export CC
export CXX
echo $PATH
echo $(which clang)
ls -lah $SHIMS
echo $(which clang++)


# Execute the command with all remaining arguments
exec "$@"
