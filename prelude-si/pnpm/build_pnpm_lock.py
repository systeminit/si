#!/usr/bin/env python3
"""
Generates/updates the Pnpm lock file
"""
import argparse
import os
import shutil
import subprocess
import sys

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--package-dir",
        help="Path to the workspace package",
    )
    parser.add_argument(
        "--pnpm-lock-out-path",
        help="Path to output `pnpm-lock.yaml`",
    )

    args = parser.parse_intermixed_args()

    if args.package_dir:
        cwd = args.package_dir
    else:
        cwd = None

    cmd = ["pnpm", "install", "--lockfile-only"]

    exit_code = subprocess.call(cmd, cwd=cwd)

    if exit_code == 0:
        src = "pnpm-lock.yaml"
        if cwd:
            src = os.path.join(cwd, src)

        shutil.copy(
            src,
            args.pnpm_lock_out_path,
        )

    sys.exit(exit_code)
