#!/usr/bin/env bash
set -eu

main() {
  . "$HOME/.nix-profile/etc/profile.d/nix.sh"
  find /workdir -user root -print0 | xargs -0 sudo chown "$(id -u)"
  find /workdir -group root -print0 | xargs -0 sudo chgrp "$(id -g)"
  exec nix develop "$@"
}

main "$@"
