#!/usr/bin/env bash

main() {
  set -eux

  # Redirect output to log file and console
  exec > >(
    tee /var/log/user-data.log | logger -t user-data -s 2>/dev/console
  ) 2>&1

  local user=si
  local group="$user"
  local hostname=si-dev
  local gh_users=(fnichol adamhjk alex-init)

  update_system
  install_sudo
  setup_user "$user" "$group"
  setup_workstation "$user" "$hostname"
  add_authorized_keys "$user" "${gh_users[@]}"
  save_cookie
  echo "--- All Done"
}

update_system() {
  pacman -Syu --noconfirm
}

install_sudo() {
  pacman -S --noconfirm bash bash-completion sudo
  echo "%wheel ALL=(ALL) NOPASSWD:ALL" >/etc/sudoers.d/01_wheel
}

setup_user() {
  local user="$1"
  local group="$2"

  groupadd --gid 1001 "$group"
  useradd \
    --create-home \
    --shell /bin/bash \
    --groups wheel \
    --gid "$group" --uid 1001 "$user"

}

setup_workstation() {
  local user="$1"
  local hostname="$2"

  pacman -S --noconfirm git

  local user_home
  user_home="$(getent passwd "$user" | cut -d: -f 6)"

  sudo -u "$user" git clone \
    https://github.com/fnichol/workstation.git \
    "$user_home/workstation"

  rm -f "$user_home/.bashrc"

  cd "$user_home/workstation"
  sudo -u "$user" ./bin/log ./bin/prep \
    --profile=headless \
    --skip=ruby,go \
    "$hostname"
}

add_authorized_keys() {
  local user="$1"
  shift

  sudo -u "$user" paru -S --noconfirm python-distro ssh-import-id

  local gh_user
  for gh_user in "$@"; do
    sudo -u "$user" ssh-import-id "gh:$gh_user"
  done
}

save_cookie() {
  touch /.provision_complete
}

main "$@"
