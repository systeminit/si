#!/usr/bin/env bash

print_usage() {
  local program="$1"
  local author="$2"

  need_cmd sed

  echo "$program

    Builds a Docker image an optionally publishes it.

    USAGE:
        $program [FLAGS] [-- [BUILD_ARGS...]]

    FLAGS:
        -h, --help      Prints help information
        -C, --ci        Enables CI mode
        -p, --push      Publishes the image and all tags

    ARGS:
        BUILD_ARGS      Extra, optional arguments passed to docker build

    AUTHOR:
        $author
    " | sed 's/^ \{1,4\}//g'
}

main() {
  set -eu
  if [ -n "${DEBUG:-}" ]; then set -v; fi
  if [ -n "${TRACE:-}" ]; then set -xv; fi

  local program author
  program="build-image.sh"
  author="The System Initiative <dev@systeminit.com>"

  invoke_cli "$program" "$author" "$@"
}

invoke_cli() {
  local program author ci_mode push
  program="$1"
  shift
  author="$1"
  shift

  local img
  img="${IMG:-systeminit/si-web}"

  ci_mode=""
  push=""

  OPTIND=1
  while getopts "Chp-:" arg; do
    case "$arg" in
      C)
        ci_mode=true
        ;;
      h)
        print_usage "$program" "$author"
        return 0
        ;;
      p)
        push=true
        ;;
      -)
        case "$OPTARG" in
          ci)
            ci_mode=true
            ;;
          help)
            print_usage "$program" "$author"
            return 0
            ;;
          push)
            push=true
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

  if [ "$ci_mode" = "true" ]; then
    push=true
  fi

  if [ -z "${CI:-}" ]; then
    setup_buildx
  fi
  build "$img" "$push" "$ci_mode" "$@"
}

setup_buildx() {
  local name=si

  if ! docker buildx inspect "$name" >/dev/null 2>&1; then
    docker buildx create --name "$name" --driver docker-container --use
  fi
}

build() {
  local img="$1"
  shift
  local push="$1"
  shift
  local ci_mode="$1"
  shift

  need_cmd date
  need_cmd docker
  need_cmd git

  local http_url ws_url
  http_url="${VUE_APP_SDF_BASE_HTTP_URL:-http://app.systeminit.com/api}"
  ws_url="${VUE_APP_SDF_BASE_WS_URL:-ws://app.systeminit.com/api/updates}"

  local build_version
  build_version="$(date -u +%Y%m%d.%H%M%S).0-sha.$(git show -s --format=%h)"

  cd "${0%/*}/.."

  local args
  args=(
    buildx build
    --build-arg "VUE_APP_SDF_BASE_HTTP_URL=$http_url"
    --build-arg "VUE_APP_SDF_BASE_WS_URL=$ws_url"
    --tag "$img:$build_version"
    --tag "$img:latest"
  )
  if [[ "$ci_mode" == "true" ]]; then
    args+=(--tag "$img:stable")
  fi
  args+=(--cache-from "type=registry,ref=$img:buildcache")
  if [[ "$ci_mode" == "true" ]]; then
    args+=(--cache-to "type=registry,mode=max,ref=$img:buildcache")
  fi
  args+=(--file Dockerfile)
  if [[ "$ci_mode" != "true" && "$push" != "true" ]]; then
    args+=(--load)
  fi
  if [[ "$push" == "true" ]]; then
    args+=(--push)
  fi
  args+=("$@" ../..)

  export BUILDKIT_PROGRESS=plain

  set -x
  exec docker "${args[@]}"
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
