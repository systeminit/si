#!/usr/bin/env python3
"""
Runs a "deny check bans" command with "cargo".
"""
import argparse
import subprocess
import sys


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "crate_root",
        help="Path to the top level source file of the crate",
    )

    return parser.parse_args()


def main() -> int:
    args = parse_args()

    cmd = [
        "deny",
        "check",
        "bans",
    ]
    if args.config_path:
        cmd.append("--config-path")
        cmd.append(args.config_path)
    cmd.append(args.crate_root)

    result = subprocess.run(cmd)

    return result.returncode


if __name__ == "__main__":
    sys.exit(main())
