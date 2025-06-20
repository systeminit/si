#!/usr/bin/env python3
"""
Builds a portable, standalone deno binary.
"""
import argparse
import os
import pathlib
import stat
import subprocess
import sys
from typing import List


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--deno-executable",
                        required=False,
                        type=pathlib.Path,
                        help="The location of the deno exe")
    parser.add_argument("--input",
                        required=True,
                        type=pathlib.Path,
                        help="The path to the file to run")
    parser.add_argument("--output",
                        required=False,
                        type=pathlib.Path,
                        help="If there are outputs, specify them")
    parser.add_argument(
        "--permissions",
        nargs='*',
        default=[],
        help="List of Deno permissions to grant (e.g., read write net)",
    )
    parser.add_argument(
        "--unstable-flags",
        nargs='*',
        default=[],
        help="List of unstable flags to enable",
    )

    return parser.parse_args()


def parse_permissions(perms: List[str]) -> List[str]:
    """Convert permission names to Deno CLI flags."""
    return [f'--{perm}' for perm in perms]


def parse_unstable_flags(flags: List[str]) -> List[str]:
    """Convert unstable flag names to Deno CLI flags."""
    return [f'--unstable-{flag}' for flag in flags]


def run(input_path: str, output_path: str, permissions: List[str],
        flags: List[str]) -> None:
    """Run deno compile with the specified arguments."""
    deno_bin = os.environ.get("DENO_BIN", "deno")
    cmd = [deno_bin, "run"]

    if permissions:
        cmd.extend(permissions)

    if flags:
        cmd.extend(flags)

    cmd.append(input_path)

    if output_path:
        cmd.append(output_path)

    try:
        subprocess.run(cmd, check=True, capture_output=True, text=True)
    except subprocess.CalledProcessError as e:
        print("Error running Deno:", file=sys.stderr)
        print(f"Command: {' '.join(cmd)}", file=sys.stderr)
        print(f"Exit code: {e.returncode}", file=sys.stderr)
        print(f"stdout: {e.stdout}", file=sys.stderr)
        print(f"stderr: {e.stderr}", file=sys.stderr)
        raise


def main() -> int:
    try:
        args = parse_args()

        abs_input_path = os.path.abspath(args.input)
        abs_output_path = os.path.abspath(args.output)

        if not os.path.exists(abs_input_path):
            print(f"Error: Input file not found: {abs_input_path}",
                  file=sys.stderr)
            return 1

        permissions_list = parse_permissions(args.permissions)
        flags_list = parse_unstable_flags(args.unstable_flags)

        run(abs_input_path, abs_output_path, permissions_list, flags_list)

        return 0

    except Exception as e:
        print(f"Error: {str(e)}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
