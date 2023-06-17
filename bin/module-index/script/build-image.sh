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
        -l, --latest    When '--push' is used, also push to the latest tag
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
  local program author ci_mode push latest
  program="$1"
  shift
  author="$1"
  shift

  local img source license
  img="${IMG:-systeminit/module-index}"
  source="http://github.com/systeminit/si.git"
  license="PROPRIETARY"

  ci_mode=""
  push=""
  latest=""

  OPTIND=1
  while getopts "Chlp-:" arg; do
    case "$arg" in
      C)
        ci_mode=true
        ;;
      h)
        print_usage "$program" "$author"
        return 0
        ;;
      l)
        latest=true
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
          latest)
            latest=true
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
  build "$img" "$push" "$latest" "$ci_mode" "$author" "$source" "$license" "$@"
}

setup_buildx() {
  local name=si

  if ! docker buildx inspect "$name" >/dev/null 2>&1; then
    docker buildx create --name "$name" --driver docker-container --use
  else
    docker buildx use "$name"
  fi
}

build() {
  local img="$1"
  shift
  local push="$1"
  shift
  local latest="$1"
  shift
  local ci_mode="$1"
  shift
  local author="$1"
  shift
  local source="$1"
  shift
  local license="$1"
  shift

  need_cmd docker
  need_cmd git

  # Prefer `gdate` if available if we are running on macOS to try and favor the
  # GNU version of date over the BSD version for epoch timestamp parsing.
  local date_cmd
  if command -v gdate >/dev/null 2>&1; then
    date_cmd="gdate"
  else
    need_cmd date
    date_cmd="date"
  fi

  # Get a build time in UTC, allowing for override by SOURCE_DATE_EPOCH
  # See: https://reproducible-builds.org/specs/source-date-epoch/
  local build_time
  build_time="${SOURCE_DATE_EPOCH:-$("$date_cmd" -u +%s)}"

  local created
  created="$("$date_cmd" -u -d "@$build_time" +%FT%TZ)"

  local revision
  revision="$(git show -s --format=%H)"

  local build_version
  build_version="$(
    "$date_cmd" -u -d "@$build_time" +%Y%m%d.%H%M%S
  ).0-sha.$(git show -s --format=%h)"

  local image_url
  image_url="https://hub.docker.com/r/$img"
  image_url="$image_url/tags?page=1&ordering=last_updated&name=$build_version"

  local commit_url
  commit_url="${source%.git}/commit/$revision"

  cd "${0%/*}/.."

  local args
  args=(
    buildx build
    --label "name=$img"
    --label "maintainer=$author"
    --label "org.opencontainers.image.version=$build_version"
    --label "org.opencontainers.image.authors=$author"
    --label "org.opencontainers.image.licenses=$license"
    --label "org.opencontainers.image.source=$source"
    --label "org.opencontainers.image.revision=$revision"
    --label "org.opencontainers.image.created=$created"
    --label "com.systeminit.image.image_url=$image_url"
    --label "com.systeminit.image.commit_url=$commit_url"
    --tag "$img:$build_version"
  )
  if [[ "$push" != "true" || "$latest" == "true" ]]; then
    args+=(--tag "$img:latest")
  fi
  if [[ "$ci_mode" == "true" ]]; then
    args+=(--tag "$img:sha-$revision")
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

  echo "--- Building image '$img'"
  set -x
  docker "${args[@]}" || die "Docker build failed"
  set +x

  build_manifest "$img" "$build_version" "$revision" "$created" \
    "$author" "$source" "$license" "$image_url" "$commit_url"
}

build_manifest() {
  local img="$1"
  local build_version="$2"
  local revision="$3"
  local created="$4"
  local author="$5"
  local source="$6"
  local license="$7"
  local image_url="$8"
  local commit_url="$9"

  need_cmd basename
  need_cmd dirname
  need_cmd jq
  need_cmd mkdir
  need_cmd pwd

  local manifest
  manifest="../../tmp/release-$(basename "$img").manifest.json"
  mkdir -p "$(dirname "$manifest")"
  manifest="$(cd "$(dirname "$manifest")" && pwd)/$(basename "$manifest")"

  echo "--- Writing manifest $manifest"
  jq -n -c \
    --arg img "$img" \
    --arg build_version "$build_version" \
    --arg revision "$revision" \
    --arg created "$created" \
    --arg author "$author" \
    --arg source "$source" \
    --arg license "$license" \
    --arg image_url "$image_url" \
    --arg commit_url "$commit_url" \
    '. + {
      name: $img,
      author: $author,
      license: $license,
      version: $build_version,
      created: $created,
      revision: $revision,
      repo_url: $source,
      image_url: $image_url,
      commit_url: $commit_url
    }' >"$manifest"
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
