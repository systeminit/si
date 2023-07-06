#!/usr/bin/env python3
"""
Prints output of a Clippy compilation and exits non-zero if appropriate.
"""
import argparse
import os
import sys


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "clippy_txt",
        help="The file from a Clippy compilation with its output")

    return parser.parse_args()


def main() -> int:
    args = parse_args()

    # If output is empty, there are no errors/warnings and we can exit `0`
    if os.path.getsize(args.clippy_txt) == 0:
        return 0

    # Otherwise print output and exit non-zero
    with open(args.clippy_txt, encoding="utf-8") as f:
        print(f.read())

    return 1


if __name__ == "__main__":
    sys.exit(main())
