#!/usr/bin/env python3
"""
Runs a yapf check.
"""
import argparse
import subprocess
import sys


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "srcs_root",
        help="Path to the top of the sources tree",
    )

    return parser.parse_args()


def main() -> int:
    args = parse_args()

    cmd = [
        "yapf",
        "--diff",
        "--recursive",
    ]
    cmd.append(args.srcs_root)

    result = subprocess.run(cmd, cwd=args.srcs_root)

    return result.returncode


if __name__ == "__main__":
    sys.exit(main())
