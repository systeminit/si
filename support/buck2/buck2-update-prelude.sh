#!/usr/bin/env bash

print_usage() {
  local program="$1"
  local version="$2"
  local author="$3"

  cat <<-EOF
	$program $version

	Updates the upstream common prelude.

	USAGE:
	    $program [FLAGS] [OPTIONS]

	FLAGS:
	    -h, --help        Prints help information
	    -V, --version     Prints version information

	OPTIONS:
	    -g, --git-repo=<URL>    Upstream Git repository with prelude
	                            (default: $GIT_REPO)
	    -r, --git-ref=<REF>     Git ref to update from
	                            (default: $GIT_REMOTE_REF)

	AUTHOR:
	    $author
	EOF
}

main() {
  set -euo pipefail
  if [ -n "${DEBUG:-}" ]; then set -v; fi
  if [ -n "${TRACE:-}" ]; then set -xv; fi

  local program version author
  program="$(basename "$0")"
  version="0.1.0"
  author="System Initiative Inc. <hello@systeminit.com>"

  # Parse CLI arguments and set local variables
  parse_cli_args "$program" "$version" "$author" "$@"
  local git_repo="$GIT_REPO"
  local git_remote_ref="$GIT_REMOTE_REF"
  unset GIT_REPO GIT_REMOTE_REF

  need_cmd git

  update "$git_repo" "$git_remote_ref"
}

parse_cli_args() {
  local program version author
  program="$1"
  shift
  version="$1"
  shift
  author="$1"
  shift

  local long_optarg

  # Upstream Git repository with prelude
  GIT_REPO="git@github.com:facebookincubator/buck2-prelude.git"
  # Git ref to update from
  GIT_REMOTE_REF="main"

  OPTIND=1
  # Parse command line flags and options
  while getopts ":g:hr:V-:" opt; do
    case $opt in
      g)
        GIT_REPO="$OPTARG"
        ;;
      h)
        print_usage "$program" "$version" "$author"
        exit 0
        ;;
      r)
        GIT_REMOTE_REF="$OPTARG"
        ;;
      V)
        print_version "$program" "$version"
        exit 0
        ;;
      -)
        long_optarg="${OPTARG#*=}"
        case "$OPTARG" in
          help)
            print_usage "$program" "$version" "$author"
            exit 0
            ;;
          git-ref=?*)
            GIT_REMOTE_REF="$long_optarg"
            ;;
          git-ref*)
            print_usage "$program" "$version" "$author" >&2
            die "missing required argument for --$OPTARG option"
            ;;
          git-repo=?*)
            GIT_REPO="$long_optarg"
            ;;
          git-repo*)
            print_usage "$program" "$version" "$author" >&2
            die "missing required argument for --$OPTARG option"
            ;;
          version)
            print_version "$program" "$version" "true"
            exit 0
            ;;
          '')
            # "--" terminates argument processing
            break
            ;;
          *)
            print_usage "$program" "$version" "$author" >&2
            die "invalid argument --$OPTARG"
            ;;
        esac
        ;;
      \?)
        print_usage "$program" "$version" "$author" >&2
        die "invalid option: -$OPTARG"
        ;;
    esac
  done
  shift "$((OPTIND - 1))"
}

need_cmd() {
  if ! command -v "$1" >/dev/null; then
    die "Required command '$1' not found on PATH"
  fi
}

die() {
  printf -- "\nxxx %s\n\n" "$1" >&2
  exit 1
}

update() {
  local git_repo="$1"
  local git_remote_ref="$2"

  local git_root
  git_root="$(git rev-parse --show-toplevel)"

  cd "$git_root"
  echo "--- Pulling prelude updates from $git_repo#$git_remote_ref"
  git subtree pull --prefix prelude "$git_repo" "$git_remote_ref" --squash
}

if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
  main "$@" || exit 1
fi
