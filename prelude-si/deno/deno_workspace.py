#!/usr/bin/env python3

import argparse
import os
import shutil
import sys
from typing import List, Optional


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=
        "Create a Deno workspace from a root config and package files")
    parser.add_argument("--workspace_dir",
                        required=True,
                        help="Directory where the workspace will be created")
    parser.add_argument("--root_config",
                        required=True,
                        help="Path to the root deno.json configuration file")
    parser.add_argument("--package",
                        action='append',
                        required=True,
                        help="Package files to include in the workspace")

    return parser.parse_args()


def get_target_path(pkg_file: str) -> str:
    """Extract the target path from a package file path.

    Args:
        pkg_file: Full path to package file like 'buck-out/v2/gen/root/.../bin/lang-js/deno.json'

    Returns:
        Relative path like 'bin/lang-js/deno.json'
    """
    parts = pkg_file.split(os.sep)
    try:
        # Find 'bin' or 'lib' in the path
        for i, part in enumerate(parts):
            if part in ['bin', 'lib']:
                return os.path.join(*parts[i:])
    except:
        return os.path.basename(pkg_file)
    return os.path.basename(pkg_file)


def create_workspace(workspace_dir: str, root_config: str,
                     packages: List[str]) -> None:
    """Create a Deno workspace with all files in the correct structure at the root level.

    Args:
        workspace_dir: Directory that will become the root
        root_config: Path to the root deno.json configuration file
        packages: List of package files to include in the workspace
    """
    os.makedirs(workspace_dir, exist_ok=True)

    shutil.copy2(root_config, os.path.join(workspace_dir, "deno.json"))

    for pkg_file in packages:
        target_path = get_target_path(pkg_file)
        target_path = target_path.replace('__deno.json__/', '')
        dest = os.path.join(workspace_dir, target_path)

        os.makedirs(os.path.dirname(dest), exist_ok=True)

        try:
            print(f"Copying {pkg_file} to {dest}")
            shutil.copy2(pkg_file, dest)
        except Exception as e:
            print(f"Error copying {pkg_file} to {dest}: {e}", file=sys.stderr)
            sys.exit(1)


def main() -> Optional[int]:
    try:
        args = parse_args()
        create_workspace(args.workspace_dir, args.root_config, args.package)
        return 0
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
