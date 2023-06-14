#!/usr/bin/env python3
"""
Builds an isolated Docker context tree containing all sources needed to build
the image.
"""
import argparse
import os
import shutil
import sys
import tempfile


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--dockerfile",
        help="Dockerfile to build",
    )
    parser.add_argument(
        "--src",
        action="append",
        metavar="SRC=DST",
        help="Add a source into the source tree",
    )
    parser.add_argument(
        "out_path",
        help="Path to output directory",
    )

    return parser.parse_args()


def main() -> int:
    args = parse_args()

    with tempfile.TemporaryDirectory() as tempdir:
        root_dir = os.path.join(tempdir, "root")

        dst = os.path.join(root_dir, "Dockerfile")
        parent_dir = os.path.dirname(dst)
        if parent_dir:
            dst_dir = os.path.join(root_dir, parent_dir)
            if not os.path.isdir(dst_dir):
                os.makedirs(dst_dir, exist_ok=True)
        shutil.copy(
            args.dockerfile,
            dst,
        )

        for arg in args.src or []:
            src, dst = arg.split("=")

            os.makedirs(os.path.join(root_dir, dst), exist_ok=True)

            if os.path.isdir(src):
                shutil.copytree(
                    src,
                    os.path.join(root_dir, dst),
                    symlinks=True,
                    dirs_exist_ok=True,
                )
            else:
                shutil.copy(
                    src,
                    os.path.join(root_dir, dst),
                )

        shutil.move(root_dir, args.out_path)
    return 0


if __name__ == "__main__":
    sys.exit(main())
