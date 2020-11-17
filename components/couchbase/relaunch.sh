#!/usr/bin/env sh
# shellcheck shell=sh disable=SC2039

print_usage() {
  local program="$1"

  echo "$program

    Re-launches a couchbase database container

    This program effectively launches a new, empty database instance which can
    be convenient when developing new features that affect the data model.

    USAGE:
        $program [FLAGS] [--]

    FLAGS:
        -h, --help      Prints help information
    " | sed 's/^ \{1,4\}//g'
}

main() {
  set -eu
  if [ -n "${DEBUG:-}" ]; then set -v; fi
  if [ -n "${TRACE:-}" ]; then set -xv; fi

  local program
  program="$(basename "$0")"

  OPTIND=1
  while getopts "h-:" arg; do
    case "$arg" in
      h)
        print_usage "$program"
        return 0
        ;;
      -)
        case "$OPTARG" in
          help)
            print_usage "$program"
            return 0
            ;;
          '')
            # "--" terminates argument processing
            break
            ;;
          *)
            print_usage "$program" >&2
            die "invalid argument --$OPTARG"
            ;;
        esac
        ;;
      \?)
        print_usage "$program" >&2
        die "invalid argument; arg=-$OPTARG"
        ;;
    esac
  done
  shift "$((OPTIND - 1))"

  local name="db"

  if is_running "$name"; then
    echo "--- Stopping running container '$name'"
    docker container stop "$name"
    docker container wait "$name"
  fi

  if created "$name"; then
    echo "--- Removing container '$name'"
    docker container rm "$name"
  fi

  exec "${0%/*}/run.sh"
}

created() {
  [ -n "$(docker container ls --filter "name=$1" --all --quiet)" ]
}

is_running() {
  [ -n "$(
    docker container ls --filter "name=$1" --filter "status=running" --quiet
  )" ]
}

main "$@" || exit 1
