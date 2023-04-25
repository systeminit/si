#!/usr/bin/env bash
if [ ! $1 ] || [ "$1" == "" ]; then
    echo "must provide buck2 commit on \"latest\" branch as first argument"
    exit 1
fi

# Ensure that we can execute this script from any directory on both macOS and Linux.
cd $(dirname "${BASH_SOURCE[0]}")

# Now, clone buck2 and checkout at the provided commit.
git clone https://github.com/facebook/buck2.git buck2
cd buck2
git checkout $1

# Generate the new lockfile and patch file.
cargo generate-lockfile
git diff /dev/null Cargo.lock > Cargo.lock.patch
mv Cargo.lock ..
mv Cargo.lock.patch ..

# Cleanup and exit.
cd ..
rm -rf buck2
