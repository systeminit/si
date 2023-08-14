#!/usr/bin/env python3
"""
Runs shellcheck on a sources tree.
"""
import argparse
import glob
import os
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
        "shellcheck",
    ]
    cmd.extend(
        glob.glob(
            os.path.join(
                "**",
                "*.sh",
            ),
            root_dir=args.srcs_root,
            recursive=True,
        ))

    result = subprocess.run(cmd, cwd=args.srcs_root)

    return result.returncode


if __name__ == "__main__":
    sys.exit(main())
