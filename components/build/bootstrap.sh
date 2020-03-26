#!/bin/bash
#
# Bootstrap an environment to build our software
#
# - Supports Arch Linux and Ubuntu

OS="unknown"

if [[ -f /usr/bin/pacman ]]; then
  OS="arch"
elif [[ -f /usr/bin/apt-get ]]; then 
  OS="ubuntu"
fi

if [[ $OS == arch ]]; then
  pacman -Syu --noconfirm make sudo git kustomize
elif [[ $OS == ubuntu ]]; then
  apt-get update -y && apt-get upgrade -y
  apt-get install -y make sudo git
else
  echo "*** Cannot install on unknown OS ***"
  exit 1
fi

echo "Successfully ready to build The System Initiative"

exit 0
