#!/usr/bin/env bash
set -eux

main() {
  # shellcheck shell=sh disable=SC1091
  . "$HOME/.nix-profile/etc/profile.d/nix.sh"

  find /workdir -user root -print0 | xargs -0 sudo chown "$(id -u)" || echo "None found"
  find /workdir -group root -print0 | xargs -0 sudo chgrp "$(id -g)" || echo "None found"

  # If a first argument is present, then invoke with `--command`
  if [[ "$#" -eq 0 ]]; then
    exec nix develop "$@"
  else
    exec nix develop --command "$@"
  fi
}

main "$@"
