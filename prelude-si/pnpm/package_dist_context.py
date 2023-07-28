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
    parser.add_argument(
        "out_path",
        help="Path to output directory",
    )

    args = parser.parse_args()

    with tempfile.TemporaryDirectory() as tempdir:
        root_dir = os.path.join(tempdir, "root")
        lib_dir = os.path.join(root_dir, "lib")
        bin_dir = os.path.join(root_dir, "bin")
        package_dir = os.path.join(lib_dir, args.package_dir)

        # Copy node_modules prunned tree into tempdir
        shutil.copytree(
            args.package_node_modules_path,
            lib_dir,
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

        js_path = '${0%/*}/../lib/' + args.package_dir + '/dist/index.js'
        binary_content = [
            "#!/usr/bin/env sh",
            f"exec node \"{js_path}\" \"$@\"",
        ]

        binary = os.path.join(bin_dir, os.path.basename(args.package_dir))
        os.makedirs(os.path.dirname(binary), exist_ok=True)
        with open(binary, "w") as f:
            f.write("\n".join(binary_content) + "\n")
        os.chmod(
            binary,
            stat.S_IRUSR
            | stat.S_IXUSR
            | stat.S_IRGRP
            | stat.S_IXGRP
            | stat.S_IROTH
            | stat.S_IXOTH,
        )

        shutil.move(root_dir, args.out_path)
