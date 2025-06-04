#!/usr/bin/env bash
set -eu

main() {
  # shellcheck shell=sh disable=SC1091
  . "$HOME/.nix-profile/etc/profile.d/nix.sh"

  cd /workdir

  # If a first argument is present, then invoke with `--command`
  if [[ "$#" -eq 0 ]]; then
    exec nix develop "$@"
  else
    exec nix develop --command "$@"
  fi
}

main "$@"
