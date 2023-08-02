#!/bin/sh
set -e

print_unsupported_platform()
{
    printf "\n error: We're sorry, but it looks like we don't support our installation on your platform"
}

get_latest_version() {
    local _version="$(curl -s 'https://auth-api.systeminit.com/github/releases/latest' | \
                         awk 'BEGIN { FS="\""; RS="," }
                         $2 == "version" { print $4 }')"
    RETVAL="$_version"
}

get_os() {
  local _os="$(uname)"
  case "$_os" in
      "Linux") OS="linux";;
      "Darwin") OS="darwin";;
      *)
          print_unsupported_platform
          exit 1
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
          exit 1
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
    local _download_url="https://github.com/systeminit/si/releases/download/$_version/si-$_arch-$_os.tar.gz"

    RETVAL="$_download_url"
}


download_tarball() {
    build_download_url
    local _download_url="$RETVAL"
    echo "Downloading $_download_url..."
    curl -L -o "/tmp/si.tar.gz" "$_download_url";
}

if download_tarball; then
    echo "Downloaded the release"

    if [ -e "/usr/local/bin/si" ]; then
        sudo rm -f /usr/local/bin/si
    fi

    mkdir /tmp/si
    tar xzvf "/tmp/si.tar.gz" -C "/tmp/si"
    sudo chmod a+x /tmp/si/si
    sudo cp /tmp/si/si /usr/local/bin

    rm -f "/tmp/si.tar.gz"
    rm -rf "/tmp/si"
else
    echo "error: failed to download the release"
    echo"       check your internet and try again; if the problem persists, let us know in discord"
    exit 1
fi

if command -v si >/dev/null; then
  printf "\n SI has been successfully installed."
else
  printf "\n SI has been downloaded but isn't on your PATH. Check /usr/local/bin to ensure it's on PATH"
fi