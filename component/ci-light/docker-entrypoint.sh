#!/usr/bin/env bash
set -eu

main() {
  # shellcheck shell=sh disable=SC1091
  . "$HOME/.nix-profile/etc/profile.d/nix.sh"

  find /workdir -user root -print0 | xargs -0 sudo chown "$(id -u)"
  find /workdir -group root -print0 | xargs -0 sudo chgrp "$(id -g)"

  # If a first argument is present, then invoke with `--command`
  if [[ "$#" -eq 0 ]]; then
    exec nix develop .#ci "$@"
  else
    exec nix develop .#ci --command "$@"
  fi
}

main "$@"
