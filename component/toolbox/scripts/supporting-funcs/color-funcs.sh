#!/bin/bash
# ---------------------------------------------------------------------------------------------------
# Color helper functions for consistent formatting across scripts
# ---------------------------------------------------------------------------------------------------

# Basic colors
red() { echo -e "\033[0;31m$*\033[0m"; }
green() { echo -e "\033[0;32m$*\033[0m"; }
yellow() { echo -e "\033[1;33m$*\033[0m"; }
blue() { echo -e "\033[0;34m$*\033[0m"; }
cyan() { echo -e "\033[1;36m$*\033[0m"; }
magenta() { echo -e "\033[0;35m$*\033[0m"; }
white() { echo -e "\033[1;37m$*\033[0m"; }

# Styles
bold() { echo -e "\033[1m$*\033[0m"; }
dim() { echo -e "\033[2m$*\033[0m"; }
underline() { echo -e "\033[4m$*\033[0m"; }

# Semantic colors for usage output
title() { cyan "$*"; }
section() { blue "$*"; }
usage_text() { yellow "$*"; }
option() { green "$*"; }
example() { dim "$*"; }
