#!/usr/bin/env python3
"""
Runs deno cache.
"""
import argparse
from datetime import datetime
import os
import pathlib
import subprocess
import sys


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input",
                        required=True,
                        type=pathlib.Path,
                        help="The path to the file to run")
    parser.add_argument('--marker', required=True)
    return parser.parse_args()


def run_cache(input_path: str) -> None:
    """Run deno cache with the specified arguments."""
    deno_bin = os.environ.get("DENO_BIN", "deno")
    cmd = [deno_bin, "cache"]
    cmd.append(input_path)

    try:
        result = subprocess.run(cmd, check=True, text=True)
        if result.stdout:
            print(result.stdout)
    except subprocess.CalledProcessError as e:
        print("Error running deno cache:", file=sys.stderr)
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
        abs_input_path = os.path.abspath(args.input)

        run_cache(abs_input_path)
        with open(args.marker, 'w') as f:
            f.write(f'Cache created at {datetime.now()}')
        return 0

    except Exception as e:
        print(f"Error: {str(e)}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
