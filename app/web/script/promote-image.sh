#!/usr/bin/env bash

print_usage() {
  local program="$1"
  local author="$2"

  need_cmd sed

  echo "$program

    Promotes a Docker image Git SHA to a target tag.

    USAGE:
        $program [FLAGS] [--] IMG SRC_SHA TAG

    FLAGS:
        -h, --help      Prints help information
        -C, --ci        Enables CI mode
        -l, --latest    When '--push' is used, also push to the latest tag
        -p, --push      Publishes the image and all tags

    ARGS:
        IMG       Docker image [ex: systeminit/sdf]
        SRC_SHA   Long Git SHA reference
        TAG       Target image tag to push [ex: stable]

    AUTHOR:
        $author
    " | sed 's/^ \{1,4\}//g'
}

main() {
  set -eu
  if [ -n "${DEBUG:-}" ]; then set -v; fi
  if [ -n "${TRACE:-}" ]; then set -xv; fi

  local program author
  program="promote-image.sh"
  author="The System Initiative <dev@systeminit.com>"

  invoke_cli "$program" "$author" "$@"
}

invoke_cli() {
  local program author
  program="$1"
  shift
  author="$1"
  shift

  OPTIND=1
  while getopts "h-:" arg; do
    case "$arg" in
      h)
        print_usage "$program" "$author"
        return 0
        ;;
      -)
        case "$OPTARG" in
          help)
            print_usage "$program" "$author"
            return 0
            ;;
          '')
            # "--" terminates argument processing
            break
            ;;
          *)
            print_usage "$program" "$author" >&2
            die "invalid argument --$OPTARG"
            ;;
        esac
        ;;
      \?)
        print_usage "$program" "$author" >&2
        die "invalid argument; arg=-$OPTARG"
        ;;
    esac
  done
  shift "$((OPTIND - 1))"

  if [[ -z "${1:-}" ]]; then
    print_usage "$program" "$author" >&2
    die "required argument: IMG"
  fi
  local img="$1"
  shift

  if [[ -z "${1:-}" ]]; then
    print_usage "$program" "$author" >&2
    die "required argument: SRC_SHA"
  fi
  local src_sha="$1"
  shift

  if [[ -z "${1:-}" ]]; then
    print_usage "$program" "$author" >&2
    die "required argument: TAG"
  fi
  local tag="$1"

  promote "$img" "$src_sha" "$tag"
}

promote() {
  local img="$1"
  local src_sha="$2"
  local tag="$3"

  need_cmd docker

  echo "  - Pulling image tagged with ${img}:sha-${src_sha}"
  docker pull "$img:sha-$src_sha"
  echo "  - Tagging image ${img}: sha-${src_sha} -> ${tag}"
  docker tag "$img:sha-$src_sha" "$img:$tag"
  echo "  - Pushing image tag ${img}:${tag}"
  docker push "$img:$tag"
}

die() {
  printf -- "\nxxx %s\n\n" "$1" >&2
  exit 1
}

need_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    die "Required command '$1' not found on PATH"
  fi
}

main "$@" || exit 1
