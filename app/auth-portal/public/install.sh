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
    RETVAL="$_version"
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

get_si_file_name() {
    get_os
    local _os="$RETVAL"
    get_architechure
    local _arch="$RETVAL"

    local _file="si-$_arch-$_os.tar.gz"
    RETVAL="$_file"
}

build_download_url() {
    get_latest_version
    local _version="$RETVAL"

    get_si_file_name
    local _file="$RETVAL"

    #local _download_url="https://github.com/systeminit/si/releases/download/$_version/$_file"
    local _download_url="https://github.com/stack72/test-download/releases/download/$_version/$_file"

    RETVAL="$_download_url"
}

get_download_location() {
    local _temp_dir="$(mktemp -d)"
    RETVAL="$_temp_dir"
}

download_tarball() {
    build_download_url
    local _download_url="$RETVAL"

    get_download_location
    local _temp_dir="$RETVAL"

    get_si_file_name
    local _file="$_temp_dir/$RETVAL"

    say_white "Downloading release binary from $_download_url to $_file"
    curl -L -o "$_file" "$_download_url"

    RETVAL="$_temp_dir"
}

check_old_release() {
    say_yellow "Checking for an old release of System Initiative"
    if [ -e "/usr/local/bin/si" ]; then
        sudo rm -f /usr/local/bin/si
    fi
}

download_tarball
_temp_dir="$RETVAL"

get_si_file_name
_file="$RETVAL"

check_old_release

say_white "Extracting file: $_temp_dir/$_file"
tar xzvf "$_temp_dir/$_file" -C "$_temp_dir"

say_white "Changing binary permissions to make it executable"
chmod +x "$_temp_dir/si"

say_white "Moving the binary to $HOME/.si/bin"
mkdir -p $HOME/.si/bin
mv "$_temp_dir/si" "$HOME/.si/bin"

say_white "Linking $HOME/.si/bin/si to /usr/local/bin"
sudo ln -s $HOME/.si/bin/si /usr/local/bin/

if [ "$(command -v si)" != "/usr/local/bin/si" ]; then
  say_red "System Initiative has been downloaded but isn't on your PATH. Check /usr/local/bin to ensure it's on PATH"
else
  say_green "System Initiative has been successfully installed."
fi