#!/usr/bin/env zsh
set -e

function help-and-die {
    echo "options:
    update.sh buck2    <commit-hash>
    update.sh reindeer <commit-hash>"
    exit 1
}

REPO=""
if [ ! $1 ] || [ "$1" = "" ] || [ ! $2 ] || [ "$2" = "" ]; then
    help-and-die
elif [ "$1" = "buck2" ]; then
    REPO="https://github.com/facebook/buck2.git"
elif [ "$1" = "reindeer" ]; then
    REPO="https://github.com/facebookincubator/reindeer.git"
else
    help-and-die
fi

# Ensure that we can move the files to this script's directory on both macOS and Linux.
SCRIPT_DIR=$(cd $(dirname "${BASH_SOURCE[0]}"); pwd -P)
FLAKE_DIR=${SCRIPT_DIR}/${1}

# Create a temporary directory, clone the repository, and checkout the provided commit.
TEMP_DIR=$(mktemp -d)
cd $TEMP_DIR
git clone $REPO $1
cd $1
git checkout $2

# Generate the new lockfile and patch file. The "git diff" command will have a non-zero exit code,
# so we need to ignore it.
cargo generate-lockfile
git --no-pager diff /dev/null Cargo.lock > ${FLAKE_DIR}/Cargo.lock.patch || true
cp Cargo.lock ${FLAKE_DIR}/Cargo.lock
