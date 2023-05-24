#!/usr/bin/env python3
"""
Builds a binary executable using `pkg`.
"""
import argparse
import json
import os
from posix import symlink
import shutil
import subprocess
import sys
import tempfile

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--pkg-bin",
        help="Path to `pkg` binary",
    )
    parser.add_argument(
        "--package-dir",
        help="Path to the workspace member package",
    )
    parser.add_argument(
        "--package-node-modules-path",
        help="Path to package `node_modules`",
    )
    parser.add_argument(
        "--dist-path",
        help="Path to `dist` scripts",
    )
    parser.add_argument("--extra-src",
                        action="append",
                        help="Additional file(s) needed to produce the binary")
    parser.add_argument(
        "bin_out_path",
        help="Path to output binary executable",
    )

    args = parser.parse_args()

    with tempfile.TemporaryDirectory() as tempdir:
        root_dir = os.path.join(tempdir, "root")
        package_dir = os.path.join(root_dir, args.package_dir)

        # Copy node_modules prunned tree into tempdir
        shutil.copytree(
            args.package_node_modules_path,
            root_dir,
            symlinks=True,
        )
        # Copy dist into the sub-package's dir
        shutil.copytree(
            args.dist_path,
            os.path.join(
                package_dir,
                os.path.basename(args.dist_path),
            ),
            symlinks=True,
        )

        # Copy in extra files into the sub-package's dir
        for src in args.extra_src or []:
            if os.path.dirname(src):
                os.makedirs(
                    os.path.join(root_dir, os.path.dirname(src)),
                    exist_ok=True,
                )
            shutil.copy(os.path.abspath(src), os.path.join(root_dir, src))

        cmd = [
            os.path.abspath(args.pkg_bin),
            ".",
            "--output",
            os.path.join("out", os.path.basename(args.bin_out_path)),
        ]

        exit_code = subprocess.call(cmd, cwd=package_dir)

        if exit_code == 0:
            shutil.copy(
                os.path.join(
                    package_dir,
                    "out",
                    os.path.basename(args.bin_out_path),
                ),
                args.bin_out_path,
            )

    sys.exit(exit_code)
