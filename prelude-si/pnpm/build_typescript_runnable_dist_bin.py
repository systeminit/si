#!/usr/bin/env python3
"""
Builds an isolated dist tree containing a pruned sub-package and all
production node_modules.
"""
import argparse
import os
import shutil
import stat
import tempfile

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--cd-path",
        required=True,
        help="Path to runnable dist tree root",
    )
    parser.add_argument(
        "--rel-path",
        required=True,
        help="Path to program under tree root",
    )
    parser.add_argument(
        "out_path",
        help="Path to output",
    )

    args = parser.parse_args()

    abs_cd_path = os.path.abspath(args.cd_path)

    binary_content = [
        "#!/usr/bin/env sh",
        "set -eu",
        f"cd {abs_cd_path}",
        f"exec ./{args.rel_path} \"$@\"",
    ]

    with open(args.out_path, "w") as f:
        f.write("\n".join(binary_content) + "\n")

    os.chmod(
        args.out_path,
        stat.S_IRUSR
        | stat.S_IXUSR
        | stat.S_IRGRP
        | stat.S_IXGRP
        | stat.S_IROTH
        | stat.S_IXOTH,
    )
