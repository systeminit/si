#!/usr/bin/env bash
set -eu

if [[ -n "${DEBUG:-}" ]]; then set -v; fi
if [[ -n "${TRACE:-}" ]]; then set -xv; fi

# If running in Github, we don't have an interactive
# terminal so the commands can't request user input
if [[ "${GITHUB_ACTIONS:-}" = "true" ]]; then
  terminal="-t"
else
  terminal="-it"
fi

args=(
  --rm
  "${terminal}"
  -v ~/.aws:/root/.aws
  -v "$(pwd)":/aws
  -e AWS_ACCESS_KEY_ID="${AWS_ACCESS_KEY_ID:-}"
  -e AWS_SECRET_ACCESS_KEY="${AWS_SECRET_ACCESS_KEY:-}"
  -e AWS_SESSION_TOKEN="${AWS_SESSION_TOKEN:-}"
)

if [[ -n "${AWS_PROFILE:-}" ]]; then
  args+=(-e AWS_PROFILE="${AWS_PROFILE}")
fi

if [[ -n "${AWS_REGION:-}" ]]; then
  args+=(-e AWS_REGION="${AWS_REGION}")
fi

# Validate that the first argument is a valid script name
if [[ $# -gt 0 ]]; then
  script_name="$1"
  script_dir="$(dirname "$0")/scripts"

  # Build array of valid script names
  valid_scripts=()
  if [[ -d "$script_dir" ]]; then
    while IFS= read -r -d '' script_file; do
      script_basename=$(basename "$script_file")
      valid_scripts+=("$script_basename")
    done < <(find "$script_dir" -maxdepth 1 -type f -executable -print0 | sort -z)
  fi

  if [[ ${#valid_scripts[@]} -eq 0 ]] || [[ ! " ${valid_scripts[*]} " == *" ${script_name} "* ]]; then
    echo "Error: Unknown script '${script_name}'"
    echo "Available scripts:"
    printf "  %s\n" "${valid_scripts[@]}"
    echo ""
    echo "Run './awsi.sh info' to see usage for all scripts"
    exit 1
  fi
fi

docker run "${args[@]}" systeminit/toolbox:stable "$*"
