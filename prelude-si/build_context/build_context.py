#!/usr/bin/env python3
"""
Builds an isolated context tree containing all sources needed to build a target
with Buck2.
"""
import argparse
import os
import shutil
import subprocess
import sys
import tempfile


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--bxl-file",
        required=True,
        help="Path to helper BXL file",
    )
    parser.add_argument(
        "--bxl-script",
        required=True,
        help="BXL script to invoke",
    )
    parser.add_argument(
        "--src",
        action="append",
        metavar="SRC=DST",
        help="Add a source into the source tree",
    )
    parser.add_argument(
        "--dep",
        action="append",
        help="Add all dependent input sources into the source tree",
    )
    parser.add_argument(
        "out_path",
        help="Path to output directory",
    )

    return parser.parse_args()


def main() -> int:
    args = parse_args()

    srcs = args.src or []

    if args.dep:
        cmd = [
            "buck2",
            "bxl",
            "{}:{}".format(args.bxl_file, args.bxl_script),
            "--",
        ]
        for dep in args.dep:
            cmd.append("--dep")
            cmd.append(dep)
        # Okay, okay, this might be evil--removing the `$BUCK2_DAEMON_UUID`
        # prevents a buck2-in-buck2 recursion check and allows us to call `buck2
        # bxl` withouth issue. You can blame @fnichol if this goes sideways one
        # day.
        env = os.environ
        if "BUCK2_DAEMON_UUID" in env:
            del (env["BUCK2_DAEMON_UUID"])
        result = subprocess.run(cmd, capture_output=True, env=env)
        # Print out stderr from process if it failed
        if result.returncode != 0:
            sys.stderr.write(result.stderr.decode("ascii"))
        result.check_returncode()
        srcs_from_deps_raw = result.stdout.decode("ascii").splitlines()
        srcs_from_deps = map(
            lambda src: "{}={}".format(
                src,
                os.path.dirname(src) or ".",
            ),
            srcs_from_deps_raw,
        )
        srcs.extend(srcs_from_deps)

    with tempfile.TemporaryDirectory() as tempdir:
        root_dir = os.path.join(tempdir, "root")

        for arg in srcs or []:
            src, dst = arg.split("=")
            if not dst:
                dst = os.path.dirname(src) or "."

            os.makedirs(os.path.join(root_dir, dst), exist_ok=True)

            if os.path.isdir(src):
                shutil.copytree(
                    src,
                    os.path.join(root_dir, dst, os.path.basename(src)),
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
