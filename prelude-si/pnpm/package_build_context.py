#!/usr/bin/env python3
"""
Builds an isolated source tree containing a pruned sub-package and all
node_modules.
"""
import argparse
import os
import shutil
import tempfile

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--package-dir",
        help="Path to the workspace member package",
    )
    parser.add_argument(
        "--package-node-modules-path",
        help="Path to package `node_modules`",
    )
    parser.add_argument(
        "--src",
        action="append",
        metavar="DST=SRC",
        help="Add a source into the source tree"
    )
    parser.add_argument(
        "out_path",
        help="Path to output directory",
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

        for arg in args.src or []:
            dst, src = arg.split("=")

            parent_dir = os.path.dirname(dst)
            if parent_dir:
                dst_dir = os.path.join(root_dir, parent_dir)
                if not os.path.isdir(dst_dir):
                    os.makedirs(dst_dir, exist_ok=True)
            abspath_src = os.path.abspath(src)
            if os.path.isdir(abspath_src):
                shutil.copytree(
                    abspath_src,
                    os.path.join(root_dir, dst),
                    symlinks=True,
                    dirs_exist_ok=True,
                )
            else:
                shutil.copy(
                    abspath_src,
                    os.path.join(root_dir, dst),
                )


        shutil.move(root_dir, args.out_path)
