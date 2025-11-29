#!/usr/bin/env sh
# shellcheck shell=sh disable=SC3043
set -eu

print_usage() {
  local program bin default_dest default_platform
  program="$1"
  bin="$2"
  default_dest="$3"
  default_platform="$4"

  cat <<-EOF
	$program

	Installs a binary release of $bin for supported platforms

	USAGE:
	    $program [OPTIONS] [--]

	OPTIONS:
	    -h, --help                Prints help information
	    -d, --destination=<DEST>  Destination directory for installation
	                              [default: $default_dest]
	    -p, --platform=<PLATFORM> Platform type to install
	                              [examples: linux-x86_64, linux-aarch64,
	                              darwin-x86_64, darwin-aarch64]
	                              [default: $default_platform]
	    -V, --version=<VERSION>   Release version to install
	                              [examples: stable,
	                              20250218.210911.0-sha.bda1ce6ea]
	                              [default: stable]

	EXAMPLES:
	    # Installs the latest release into \`\$HOME/bin\`
	    $program

	    # Installs the latest release for all users under \`/usr/local/bin\`
	    sudo $program

	    # Installs an old release into a temp directory
	    $program -V 20250214.193652.0-sha.0f9972a53 -d /tmp
	EOF
}

main() {
  if [ -n "${DEBUG:-}" ]; then set -v; fi
  if [ -n "${TRACE:-}" ]; then set -xv; fi

  local program bin
  program="install.sh"
  bin="si"

  setup_cleanups
  setup_traps trap_exit

  parse_cli_args "$program" "$bin" "$@"
  local dest os_type cpu_type platform version
  dest="$DEST"
  os_type="$OS_TYPE"
  cpu_type="$CPU_TYPE"
  platform="$PLATFORM"
  version="$VERSION"
  unset DEST OS_TYPE CPU_TYPE PLATFORM VERSION

  section "Downloading and installing '$bin' release '$version' on '$platform'"

  local asset_url
  asset_url="$(
    asset_url "$bin" "$version" "$os_type" "$cpu_type" "$platform"
  )" || die "Unsupported platform '$platform' for '$bin' release '$version'"

  local tmpdir
  tmpdir="$(mktemp_directory)"
  cleanup_directory "$tmpdir"

  local asset
  asset="$(basename "$asset_url")"
  download "$asset_url" "$tmpdir/$asset"

  section "Extracting '$asset'"
  extract_archive "$tmpdir/$asset" "$tmpdir" "$bin"

  section "Installing '$bin'"
  install_bin "$tmpdir/$bin" "$dest/$bin" "$bin"

  section "Installation of '$bin' release '$version' complete"
  indent "$dest/$bin" --version
}

parse_cli_args() {
  local program bin
  program="$1"
  shift
  bin="$1"
  shift

  need_cmd grep
  need_cmd id
  need_cmd uname
  need_cmd tr

  local os_type cpu_type plat dest
  os_type="$(uname -s | tr '[:upper:]' '[:lower:]')"
  cpu_type="$(uname -m | tr '[:upper:]' '[:lower:]')"
  case "$cpu_type" in
    amd64 | x64 | x86_64 | x86-64) cpu_type=x86_64 ;;
    aarch64 | arm64 | arm64v8) cpu_type=aarch64 ;;
  esac
  plat="$os_type-$cpu_type"

  if [ "$(id -u)" -eq 0 ]; then
    dest="/usr/local/bin"
  else
    if echo "$PATH" | tr ':' '\n' | grep -q "^$HOME/.local/bin\$"; then
      dest="$HOME/.local/bin"
    elif echo "$PATH" | tr ':' '\n' | grep -q "^$HOME/bin\$"; then
      dest="$HOME/bin"
    else
      dest="$HOME/.$bin/bin"
    fi
  fi

  DEST="$dest"
  PLATFORM="$plat"
  VERSION="stable"

  OPTIND=1
  while getopts "d:hp:V:-:" arg; do
    case "$arg" in
      d)
        DEST="$OPTARG"
        ;;
      h)
        print_usage "$program" "$bin" "$dest" "$plat"
        exit 0
        ;;
      p)
        PLATFORM="$OPTARG"
        ;;
      V)
        VERSION="$OPTARG"
        ;;
      -)
        long_optarg="${OPTARG#*=}"
        case "$OPTARG" in
          destination=?*)
            DEST="$long_optarg"
            ;;
          destination*)
            print_usage "$program" "$bin" "$dest" "$plat" >&2
            die "missing required argument for --$OPTARG option"
            ;;
          help)
            print_usage "$program" "$bin" "$dest" "$plat"
            exit 0
            ;;
          platform=?*)
            PLATFORM="$long_optarg"
            ;;
          platform*)
            print_usage "$program" "$bin" "$dest" "$plat" >&2
            die "missing required argument for --$OPTARG option"
            ;;
          version=?*)
            VERSION="$long_optarg"
            ;;
          version*)
            print_usage "$program" "$bin" "$dest" "$plat" >&2
            die "missing required argument for --$OPTARG option"
            ;;
          '')
            # "--" terminates argument processing
            break
            ;;
          *)
            print_usage "$program" "$bin" "$dest" "$plat" >&2
            die "invalid argument --$OPTARG"
            ;;
        esac
        ;;
      \?)
        print_usage "$program" "$bin" "$dest" "$plat" >&2
        die "invalid argument; arg=-$OPTARG"
        ;;
    esac
  done
  shift "$((OPTIND - 1))"

  case "$PLATFORM" in
    linux-x86_64 | linux-aarch64 | darwin-x86_64 | darwin-aarch64) ;;
    *) die "Installation failed, unsupported platform: '$PLATFORM'" ;;
  esac

  OS_TYPE="${PLATFORM%%-*}"
  CPU_TYPE="${PLATFORM#*-}"
}

asset_url() {
  local bin version os_type cpu_type platform
  bin="$1"
  version="$2"
  os_type="$3"
  cpu_type="$4"
  platform="$5"

  local type asset_url extension
  type="binary"
  extension="tar.gz"

  asset_url="https://artifacts.systeminit.com/$bin/$version/$type"
  asset_url="$asset_url/$os_type/$cpu_type/$bin-$version-$type-$platform.$extension"

  echo "$asset_url"
}

extract_archive() {
  local archive dest bin
  archive="$1"
  dest="$2"
  bin="$3"

  need_cmd tar

  info_start "Extracting archive"
  tar -xzf "$archive" -C "$dest"
  info_end

  # Verify the binary was extracted
  if [ ! -f "$dest/$bin" ]; then
    die "Failed to extract binary '$bin' from archive"
  fi
}

remove_macos_quarantine() {
  local file
  file="$1"

  if check_cmd xattr; then
    info "Removing macOS quarantine attribute from '$file'"
    # Remove the quarantine attribute - ignore errors if it doesn't exist
    xattr -d com.apple.quarantine "$file" 2>/dev/null || true
  else
    warn "xattr command not found, skipping quarantine attribute removal"
    warn "You may need to run: xattr -d com.apple.quarantine $file"
  fi
}

install_bin() {
  local src dest bin
  src="$1"
  dest="$2"
  bin="$3"

  need_cmd dirname
  need_cmd install
  need_cmd mkdir

  info_start "Installing '$dest'"
  mkdir -p "$(dirname "$dest")"
  install -p -m 755 "$src" "$dest"
  info_end

  # Remove macOS quarantine attribute if on macOS
  if [ "$(uname -s)" = "Darwin" ]; then
    remove_macos_quarantine "$dest"
  fi

  if [ "$(dirname "$dest")" = "$HOME/.$bin/bin" ]; then
    symlink_to_system_path "$dest" "$bin"
  fi
}

symlink_to_system_path() {
  local dest bin
  dest="$1"
  bin="$2"

  need_cmd sudo

  local system_path=/usr/local/bin
  local prompt="[sudo required to link $bin under $system_path]"
  prompt="$prompt Password for %u: "

  info "Symlinking '$dest' to $system_path/$bin"
  sudo -p "$prompt" ln -snf "$dest" "$system_path/$bin"
}

trap_exit() {
  if [ $? -ne 0 ]; then
    local msg
    msg="We're sorry, but it looks like something might have gone wrong "
    msg="$msg during installation."
    {
      echo
      warn "$msg" >&2
      warn "If you need help, please join us on our discord!"
      warn ""
      warn "    https://discord.gg/system-init"
    } >&2
  fi
  trap_cleanups
}

# BEGIN: libsh.sh

#
# Copyright 2019 Fletcher Nichol and/or applicable contributors.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license (see
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option. This
# file may not be copied, modified, or distributed except according to those
# terms.
#
# libsh.sh
# --------
# project: https://github.com/fnichol/libsh
# author: Fletcher Nichol <fnichol@nichol.ca>
# version: 0.10.1
# distribution: libsh.full-minified.sh
# commit-hash: 46134771903ba66967666ca455f73ffc10dd0a03
# commit-date: 2021-05-08
# artifact: https://github.com/fnichol/libsh/releases/download/v0.10.1/libsh.full.sh
# source: https://github.com/fnichol/libsh/tree/v0.10.1
# archive: https://github.com/fnichol/libsh/archive/v0.10.1.tar.gz
#
if [ -n "${KSH_VERSION:-}" ]; then
  eval "local() { return 0; }"
fi
# shellcheck disable=SC2120
mktemp_directory() {
  need_cmd mktemp
  if [ -n "${1:-}" ]; then
    mktemp -d "$1/tmp.XXXXXX"
  else
    mktemp -d 2>/dev/null || mktemp -d -t tmp
  fi
}
# shellcheck disable=SC2120
mktemp_file() {
  need_cmd mktemp
  if [ -n "${1:-}" ]; then
    mktemp "$1/tmp.XXXXXX"
  else
    mktemp 2>/dev/null || mktemp -t tmp
  fi
}
trap_cleanup_files() {
  set +e
  if [ -n "${__CLEANUP_FILES__:-}" ] && [ -f "$__CLEANUP_FILES__" ]; then
    local _file
    while read -r _file; do
      rm -f "$_file"
    done <"$__CLEANUP_FILES__"
    unset _file
    rm -f "$__CLEANUP_FILES__"
  fi
}
need_cmd() {
  if ! check_cmd "$1"; then
    die "Required command '$1' not found on PATH"
  fi
}
trap_cleanups() {
  set +e
  trap_cleanup_directories
  trap_cleanup_files
}
print_version() {
  local _program _version _verbose _sha _long_sha _date
  _program="$1"
  _version="$2"
  _verbose="${3:-false}"
  _sha="${4:-}"
  _long_sha="${5:-}"
  _date="${6:-}"
  if [ -z "$_sha" ] || [ -z "$_long_sha" ] || [ -z "$_date" ]; then
    if check_cmd git \
      && git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
      if [ -z "$_sha" ]; then
        _sha="$(git show -s --format=%h)"
        if ! git diff-index --quiet HEAD --; then
          _sha="${_sha}-dirty"
        fi
      fi
      if [ -z "$_long_sha" ]; then
        _long_sha="$(git show -s --format=%H)"
        case "$_sha" in
          *-dirty) _long_sha="${_long_sha}-dirty" ;;
        esac
      fi
      if [ -z "$_date" ]; then
        _date="$(git show -s --format=%ad --date=short)"
      fi
    fi
  fi
  if [ -n "$_sha" ] && [ -n "$_date" ]; then
    echo "$_program $_version ($_sha $_date)"
  else
    echo "$_program $_version"
  fi
  if [ "$_verbose" = "true" ]; then
    echo "release: $_version"
    if [ -n "$_long_sha" ]; then
      echo "commit-hash: $_long_sha"
    fi
    if [ -n "$_date" ]; then
      echo "commit-date: $_date"
    fi
  fi
  unset _program _version _verbose _sha _long_sha _date
}
warn() {
  case "${TERM:-}" in
    *term | alacritty | rxvt | screen | screen-* | tmux | tmux-* | xterm-*)
      printf -- "\033[1;31;40m!!! \033[1;37;40m%s\033[0m\n" "$1"
      ;;
    *)
      printf -- "!!! %s\n" "$1"
      ;;
  esac
}
section() {
  case "${TERM:-}" in
    *term | alacritty | rxvt | screen | screen-* | tmux | tmux-* | xterm-*)
      printf -- "\033[1;36;40m--- \033[1;37;40m%s\033[0m\n" "$1"
      ;;
    *)
      printf -- "--- %s\n" "$1"
      ;;
  esac
}
setup_cleanup_directories() {
  if [ "${__CLEANUP_DIRECTORIES_SETUP__:-}" != "$$" ]; then
    unset __CLEANUP_DIRECTORIES__
    __CLEANUP_DIRECTORIES_SETUP__="$$"
    export __CLEANUP_DIRECTORIES_SETUP__
  fi
  if [ -z "${__CLEANUP_DIRECTORIES__:-}" ]; then
    __CLEANUP_DIRECTORIES__="$(mktemp_file)"
    if [ -z "$__CLEANUP_DIRECTORIES__" ]; then
      return 1
    fi
    export __CLEANUP_DIRECTORIES__
  fi
}
setup_cleanup_files() {
  if [ "${__CLEANUP_FILES_SETUP__:-}" != "$$" ]; then
    unset __CLEANUP_FILES__
    __CLEANUP_FILES_SETUP__="$$"
    export __CLEANUP_FILES_SETUP__
  fi
  if [ -z "${__CLEANUP_FILES__:-}" ]; then
    __CLEANUP_FILES__="$(mktemp_file)"
    if [ -z "$__CLEANUP_FILES__" ]; then
      return 1
    fi
    export __CLEANUP_FILES__
  fi
}
setup_cleanups() {
  setup_cleanup_directories
  setup_cleanup_files
}
setup_traps() {
  local _sig
  for _sig in HUP INT QUIT ALRM TERM; do
    trap "
      $1
      trap - $_sig EXIT
      kill -s $_sig "'"$$"' "$_sig"
  done
  if [ -n "${ZSH_VERSION:-}" ]; then
    eval "zshexit() { eval '$1'; }"
  else
    # shellcheck disable=SC2064
    trap "$1" EXIT
  fi
  unset _sig
}
trap_cleanup_directories() {
  set +e
  if [ -n "${__CLEANUP_DIRECTORIES__:-}" ] \
    && [ -f "$__CLEANUP_DIRECTORIES__" ]; then
    local _dir
    while read -r _dir; do
      rm -rf "$_dir"
    done <"$__CLEANUP_DIRECTORIES__"
    unset _dir
    rm -f "$__CLEANUP_DIRECTORIES__"
  fi
}
check_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    return 1
  fi
}
cleanup_directory() {
  setup_cleanup_directories
  echo "$1" >>"$__CLEANUP_DIRECTORIES__"
}
cleanup_file() {
  setup_cleanup_files
  echo "$1" >>"$__CLEANUP_FILES__"
}
die() {
  case "${TERM:-}" in
    *term | alacritty | rxvt | screen | screen-* | tmux | tmux-* | xterm-*)
      printf -- "\n\033[1;31;40mxxx \033[1;37;40m%s\033[0m\n\n" "$1" >&2
      ;;
    *)
      printf -- "\nxxx %s\n\n" "$1" >&2
      ;;
  esac
  exit 1
}
download() {
  local _url _dst _code _orig_flags
  _url="$1"
  _dst="$2"
  need_cmd sed
  if check_cmd curl; then
    info "Downloading $_url to $_dst (curl)"
    _orig_flags="$-"
    set +e
    curl -sSfL "$_url" -o "$_dst"
    _code="$?"
    set "-$(echo "$_orig_flags" | sed s/s//g)"
    if [ $_code -eq 0 ]; then
      unset _url _dst _code _orig_flags
      return 0
    else
      local _e
      _e="curl failed to download file, perhaps curl doesn't have"
      _e="$_e SSL support and/or no CA certificates are present?"
      warn "$_e"
      unset _e
    fi
  fi
  if check_cmd wget; then
    info "Downloading $_url to $_dst (wget)"
    _orig_flags="$-"
    set +e
    wget -q -O "$_dst" "$_url"
    _code="$?"
    set "-$(echo "$_orig_flags" | sed s/s//g)"
    if [ $_code -eq 0 ]; then
      unset _url _dst _code _orig_flags
      return 0
    else
      local _e
      _e="wget failed to download file, perhaps wget doesn't have"
      _e="$_e SSL support and/or no CA certificates are present?"
      warn "$_e"
      unset _e
    fi
  fi
  if check_cmd ftp; then
    info "Downloading $_url to $_dst (ftp)"
    _orig_flags="$-"
    set +e
    ftp -o "$_dst" "$_url"
    _code="$?"
    set "-$(echo "$_orig_flags" | sed s/s//g)"
    if [ $_code -eq 0 ]; then
      unset _url _dst _code _orig_flags
      return 0
    else
      local _e
      _e="ftp failed to download file, perhaps ftp doesn't have"
      _e="$_e SSL support and/or no CA certificates are present?"
      warn "$_e"
      unset _e
    fi
  fi
  unset _url _dst _code _orig_flags
  warn "Downloading requires SSL-enabled 'curl', 'wget', or 'ftp' on PATH"
  return 1
}
indent() {
  local _ecfile _ec _orig_flags
  need_cmd cat
  need_cmd rm
  need_cmd sed
  _ecfile="$(mktemp_file)"
  _orig_flags="$-"
  set +e
  {
    "$@" 2>&1
    echo "$?" >"$_ecfile"
  } | sed 's/^/       /'
  set "-$(echo "$_orig_flags" | sed s/s//g)"
  _ec="$(cat "$_ecfile")"
  rm -f "$_ecfile"
  unset _ecfile _orig_flags
  return "${_ec:-5}"
}
info() {
  case "${TERM:-}" in
    *term | alacritty | rxvt | screen | screen-* | tmux | tmux-* | xterm-*)
      printf -- "\033[1;36;40m  - \033[1;37;40m%s\033[0m\n" "$1"
      ;;
    *)
      printf -- "  - %s\n" "$1"
      ;;
  esac
}
info_end() {
  case "${TERM:-}" in
    *term | alacritty | rxvt | screen | screen-* | tmux | tmux-* | xterm-*)
      printf -- "\033[1;37;40m%s\033[0m\n" "done."
      ;;
    *)
      printf -- "%s\n" "done."
      ;;
  esac
}
info_start() {
  case "${TERM:-}" in
    *term | alacritty | rxvt | screen | screen-* | tmux | tmux-* | xterm-*)
      printf -- "\033[1;36;40m  - \033[1;37;40m%s ... \033[0m" "$1"
      ;;
    *)
      printf -- "  - %s ... " "$1"
      ;;
  esac
}

# END: libsh.sh

main "$@"
