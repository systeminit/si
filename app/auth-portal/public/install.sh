#!/usr/bin/env sh
set -e

print_unsupported_platform()
{
    >&2 say_red "Error: We're sorry, but it looks like we don't support our installation on your platform"
    exit 1
}

say_green()
{
    printf "%b%s%b\\n" "\\033[32;1m" "$1" "\\033[0m"
    return 0
}

say_red()
{
    printf "%b%s%b\\n" "\\033[31;1m" "$1" "\\033[0m"
}

say_yellow()
{
    printf "%b%s%b\\n" "\\033[33;1m" "$1" "\\033[0m"
    return 0
}

say_blue()
{
    printf "%b%s%b\\n" "\\033[34;1m" "$1" "\\033[0m"
    return 0
}

say_white()
{
    printf "%b%s%b\\n" "\\033[37;1m" "$1" "\\033[0m"
    return 0
}

at_exit()
{
    # shellcheck disable=SC2181
    # https://github.com/koalaman/shellcheck/wiki/SC2181
    # Disable because we don't actually know the command we're running
    if [ "$?" -ne 0 ]; then
        >&2 say_red
        >&2 say_red "We're sorry, but it looks like something might have gone wrong during installation."
        >&2 say_red "If you need help, please join us on our discord!"
    fi
}

trap at_exit EXIT

get_latest_version() {
    local _version="$(curl -s 'https://auth-api.systeminit.com/github/releases/latest' | \
                         awk 'BEGIN { FS="\""; RS="," }
                         $2 == "version" { print $4 }')"
    ## This won't be necessary when we have a non mirrored repository!
    ## RETVAL="$_version"
    local _version_x="$(echo "$_version" | awk '{ print substr ($0, 2 ) }')"
    RETVAL="$_version_x"
}

get_os() {
  local _os="$(uname)"
  case "$_os" in
      "Linux") _os="linux";;
      "Darwin") _os="darwin";;
      *)
          print_unsupported_platform
          ;;
  esac

  RETVAL="$_os"
}

get_architechure() {
  local _arch="$(uname -m)"
  case "$_arch" in
      "x86_64") _arch="x86_64";;
      "x86-64") _arch="x86_64";;
      "amd64") _arch="x86_64";;
      "x64") _arch="x86_64";;
      "arm64v8") _arch="aarch64";;
      "arm64") _arch="aarch64";;
      "aarch64") _arch="aarch64";;
      *)
          print_unsupported_platform
          ;;
  esac

  RETVAL="$_arch"
}

build_download_url() {
    get_latest_version
    local _version="$RETVAL"
    get_os
    local _os="$RETVAL"
    get_architechure
    local _arch="$RETVAL"

    #local _download_url="https://github.com/systeminit/si/releases/download/$_version/si-$_arch-$_os.tar.gz"
    local _download_url="https://github.com/stack72/test-download/releases/download/$_version/si-$_arch-$_os.tar.gz"

    RETVAL="$_download_url"
}


download_tarball() {
    build_download_url
    local _download_url="$RETVAL"
    say_white "Downloading release binary from $_download_url"
    curl -L -o "/tmp/si.tar.gz" "$_download_url"
}

check_old_release() {
    say_yellow "Checking for an old release of System Initiative"
    if [ -e "/usr/local/bin/si" ]; then
        sudo rm -f /usr/local/bin/si
    fi
}

download_tarball
check_old_release

mkdir /tmp/si
tar xzvf "/tmp/si.tar.gz" -C "/tmp/si"
sudo chmod a+x /tmp/si/si
sudo cp /tmp/si/si /usr/local/bin

rm -f "/tmp/si.tar.gz"
rm -rf "/tmp/si"

if [ "$(command -v si)" != "/usr/local/bin/si" ]; then
  say_red "System Initiative has been downloaded but isn't on your PATH. Check /usr/local/bin to ensure it's on PATH"
else
  say_green "System Initiative has been successfully installed."
fi