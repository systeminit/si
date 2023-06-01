#!/usr/bin/env python3
"""
Runs a program, optionally in a directory.
"""
import argparse
import os
import shutil
import subprocess
import sys
from typing import Optional


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--cwd",
        help="Directory under which to run the program",
    )
    parser.add_argument(
        "--copy-tree",
        action="append",
        metavar="SRC=DST",
        help="Copy a resulting SRC directory tree to DST",
    )
    parser.add_argument(
        "args",
        help="Program and arguments",
        nargs=argparse.REMAINDER,
    )

    return parser.parse_args()


def main() -> int:
    args = parse_args()
    args.args.pop(0)

    cwd = args.cwd

    cmd = []
    for arg in args.args:
        cmd.append(compute_path(arg, cwd))

    print("--- Running: cmd='{}', cwd={}".format(" ".join(cmd), cwd))
    exit_code = subprocess.call(cmd, cwd=cwd)

    if exit_code == 0:
        # Determine final path entries based on rel or abs paths
        copy_trees = []
        for arg in args.copy_tree or []:
            src, dst = arg.split("=")
            src = compute_path(src, cwd)
            dst = compute_path(dst, cwd)
            copy_trees.append((src, dst))

        # Change to `cwd` to ensure that relative path tree copies are relative
        os.chdir(cwd)

        for copy_tree in copy_trees:
            src, dst = copy_tree

            print(f"  - Copying tree: src='{src}', dst={dst}, cwd={cwd}")
            shutil.copytree(
                src,
                dst,
                symlinks=True,
                dirs_exist_ok=True,
            )

    return exit_code


def compute_path(arg: str, cwd: Optional[str]) -> str:
    if arg.endswith("::abspath"):
        return os.path.abspath(arg.removesuffix("::abspath"))
    elif arg.endswith("::relpath"):
        if not cwd:
            print("Cannot compute relative path, --cwd not set")
            sys.exit(1)

        return os.path.relpath(
            os.path.abspath(arg.removesuffix("::relpath")),
            cwd,
        )
    else:
        return arg


if __name__ == "__main__":
    sys.exit(main())
