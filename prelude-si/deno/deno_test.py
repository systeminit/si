#!/usr/bin/env python3
"""
Runs Deno tests.
"""
import argparse
import os
import pathlib
import subprocess
import sys
from typing import List


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--input",
        action="append",
        required=True,
        type=pathlib.Path,
        help="The test file to run (can be specified multiple times)",
    )
    parser.add_argument(
        "--filter",
        type=str,
        help="Run tests with this string or pattern in the test name",
    )
    parser.add_argument("--ignore",
                        nargs="*",
                        default=[],
                        help="List of files or directories to ignore")
    parser.add_argument("--parallel",
                        action="store_true",
                        help="Run tests in parallel")
    parser.add_argument("--watch",
                        action="store_true",
                        help="Watch for file changes and restart tests")
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


def run_tests(
    input_paths: List[str],
    filter_pattern: str | None,
    flags: List[str],
    ignore_paths: List[str],
    parallel: bool,
    permissions: List[str],
    watch: bool,
) -> None:
    """Run deno test with the specified arguments."""
    deno_bin = os.environ.get("DENO_BIN", "deno")
    cmd = [deno_bin, "test"]

    if filter_pattern:
        cmd.extend(["--filter", filter_pattern])

    for ignore in ignore_paths:
        cmd.append(f"--ignore={ignore}")

    if parallel:
        cmd.append("--parallel")

    if watch:
        cmd.append("--watch")

    if permissions:
        cmd.extend(permissions)

    if flags:
        cmd.extend(flags)

    cmd.extend(input_paths)

    try:
        result = subprocess.run(cmd, check=True, text=True)
        if result.stdout:
            print(result.stdout)
    except subprocess.CalledProcessError as e:
        print("Error running deno test:", file=sys.stderr)
        print(f"Command: {' '.join(cmd)}", file=sys.stderr)
        print(f"Exit code: {e.returncode}", file=sys.stderr)
        if e.stdout:
            print(f"stdout: {e.stdout}", file=sys.stderr)
        if e.stderr:
            print(f"stderr: {e.stderr}", file=sys.stderr)
        raise


def main() -> int:
    try:
        args = parse_args()

        input_paths = []
        for input_path in args.input:
            abs_path = os.path.abspath(input_path)
            if not os.path.exists(abs_path):
                print(f"Error: Input path not found: {abs_path}",
                      file=sys.stderr)
                return 1
            input_paths.append(abs_path)

        permissions_list = parse_permissions(args.permissions)
        flags_list = parse_unstable_flags(args.unstable_flags)

        run_tests(input_paths, args.filter, flags_list, args.ignore,
                  args.parallel, permissions_list, args.watch)

        print("Tests completed successfully.")
        return 0

    except Exception as e:
        print(f"Error: {str(e)}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
