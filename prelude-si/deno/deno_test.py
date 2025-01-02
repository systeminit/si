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

    return parser.parse_args()


def run_tests(
    input_paths: List[str],
    filter_pattern: str | None,
    ignore_paths: List[str],
    parallel: bool,
    watch: bool,
) -> None:
    """Run deno test with the specified arguments."""
    cmd = ["deno", "test"]

    if filter_pattern:
        cmd.extend(["--filter", filter_pattern])

    for ignore in ignore_paths:
        cmd.append(f"--ignore={ignore}")

    if parallel:
        cmd.append("--parallel")

    if watch:
        cmd.append("--watch")

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

        run_tests(input_paths, args.filter, args.ignore, args.parallel,
                  args.watch)

        print("Tests completed successfully.")
        return 0

    except Exception as e:
        print(f"Error: {str(e)}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
