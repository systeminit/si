#!/usr/bin/env python3
"""
Runs a Pnpm script
"""
import argparse
import subprocess
import sys

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--package-dir",
        help="Path to the workspace member package",
    )
    parser.add_argument(
        "script",
        help="Pnpm script to invoke",
    )
    parser.add_argument(
        "args",
        help="Extra arguments passed to the Pnpm run script",
        nargs=argparse.REMAINDER,
    )

    args = parser.parse_args()

    cmd = ["pnpm", "run", args.script] + args.args

    exit_code = subprocess.call(cmd, cwd=args.package_dir)

    sys.exit(exit_code)
