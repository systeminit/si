#!/usr/bin/env python3
"""
Formats TypeScript/JavaScript files using deno fmt.
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
        "--deno-binary",
        required=True,
        type=pathlib.Path,
        help="The path to the deno binary",
    )
    parser.add_argument(
        "--input",
        action="append",
        required=True,
        type=pathlib.Path,
        help="The files or directories to format",
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="Check if files are formatted without making changes",
    )
    parser.add_argument("--ignore",
                        nargs="*",
                        default=[],
                        help="List of files or directories to ignore")

    return parser.parse_args()


def run_format(deno_binary: str, input_paths: List[str], check_only: bool,
               ignore_paths: List[str]) -> None:
    """Run deno fmt with the specified arguments."""
    cmd = [str(deno_binary), "fmt"]

    if check_only:
        cmd.append("--check")

    for ignore in ignore_paths:
        cmd.append(f"--ignore={ignore}")

    # Add all input paths to the command
    cmd.extend(input_paths)

    try:
        result = subprocess.run(cmd, check=True, text=True)
        if result.stdout:
            print(result.stdout)
    except subprocess.CalledProcessError as e:
        print("Error running deno fmt:", file=sys.stderr)
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

        run_format(args.deno_binary, input_paths, args.check, args.ignore)

        if args.check:
            print("Format check completed successfully.")
        else:
            print("Formatting completed successfully.")

        return 0

    except Exception as e:
        print(f"Error: {str(e)}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
