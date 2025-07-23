#!/usr/bin/env python3
"""
Builds an isolated tree containing all sources.
"""
import argparse
import os
from pathlib import Path
import shutil
import sys


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--src",
        action="append",
        help="Add a source into the source tree",
    )
    parser.add_argument(
        "--root-src",
        action="append",
        help="Add a source into the root of the source tree",
    )
    parser.add_argument(
        "out_path",
        help="Path to output directory",
    )

    return parser.parse_args()


def main() -> int:
    args = parse_args()

    for root_src in args.root_src or []:
        root_path = Path(root_src)
        os.makedirs(args.out_path, exist_ok=True)
        shutil.copy(
            os.path.abspath(root_path),
            os.path.join(args.out_path, root_path.name),
        )

    for src in args.src or []:
        parent_dir = os.path.dirname(src)
        if parent_dir:
            dst_dir = os.path.join(args.out_path, parent_dir)
            if not os.path.isdir(dst_dir):
                os.makedirs(dst_dir, exist_ok=True)
        abspath_src = os.path.abspath(src)
        if os.path.isdir(abspath_src):
            shutil.copytree(
                abspath_src,
                os.path.join(args.out_path, src),
                symlinks=True,
                dirs_exist_ok=True,
            )
        else:
            shutil.copy(
                abspath_src,
                os.path.join(args.out_path, src),
            )

    return 0


if __name__ == "__main__":
    sys.exit(main())
