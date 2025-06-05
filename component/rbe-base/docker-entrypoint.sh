#!/usr/bin/env bash
set -eu

main() {
  . "$HOME/.nix-profile/etc/profile.d/nix.sh"

  cp /workdir/* ./

  # If a first argument is present, then invoke with `--command`
  if [[ "$#" -eq 0 ]]; then
    exec nix develop "$@"
  else
    exec nix develop --command "$@"
  fi
}

main "$@"
