#!/usr/bin/env python3
"""
Redirects stdout of a command to an output file.
"""
import argparse
import subprocess

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--output",
        help="File to write stdout",
    )
    parser.add_argument(
        "cmd",
        help="Command to run",
    )
    parser.add_argument(
        "args",
        help="Extra arguments passed to the command",
        nargs=argparse.REMAINDER,
    )
    args = parser.parse_args()

    with open(args.output, "wb") as file:
        subprocess.run([args.cmd] + args.args, stdout=file).check_returncode()
