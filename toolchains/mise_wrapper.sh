#!/bin/bash
# Enhanced wrapper script for rustc that sets up the environment
SHIMS="$1"
shift 1

# Ensure shims is in the PATH
export PATH="$SHIMS:$PATH"

export RUSTC="$SHIMS/rustc"
export C="$SHIMS/clang"
export CXX="$SHIMS/clang++"

# Execute the command with all remaining arguments
exec "$@"
