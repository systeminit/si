#!/usr/bin/env python3
"""
Builds a portable, standalone deno binary.
"""
import argparse
import os
import pathlib
import subprocess
import sys
from typing import List, Optional


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--deno-exe",
        required=True,
        type=pathlib.Path,
        help="The path to the deno executable",
    )
    parser.add_argument(
        "--input",
        required=True,
        type=pathlib.Path,
        help="The path to compile, relative to the project root.",
    )
    parser.add_argument(
        "--output",
        required=True,
        type=pathlib.Path,
        help="The target directory for outputting the artifact",
    )
    parser.add_argument(
        "--deno-dir",
        type=pathlib.Path,
        default=None,
        help="Path to the pre-populated Deno cache directory (DENO_DIR)",
    )
    parser.add_argument(
        "--workspace-dir",
        type=pathlib.Path,
        default=None,
        help="The workspace directory to use as the CWD for compilation.",
    )
    parser.add_argument(
        "--permissions",
        nargs="*",
        default=[],
        help="List of Deno permissions to grant (e.g., all, read, net).",
    )
    parser.add_argument(
        "--unstable-flags",
        nargs="*",
        default=[],
        help="List of unstable flags to enable (e.g., ffi, node-globals).",
    )
    return parser.parse_args()


def run(
    deno_binary: pathlib.Path,
    input_path: pathlib.Path,
    output_path: pathlib.Path,
    permissions: List[str],
    flags: List[str],
    deno_dir: Optional[pathlib.Path],
    workspace_dir: Optional[pathlib.Path],
) -> None:
    """Run deno run with the specified arguments."""
    deno_binary_abs = deno_binary.resolve()
    deno_dir_abs = deno_dir.resolve() if deno_dir else None
    cwd = workspace_dir.resolve() if workspace_dir else pathlib.Path.cwd()

    cmd = [str(deno_binary_abs), "run"]

    cmd.extend([f"--{p}" for p in permissions])
    cmd.extend([f"--unstable-{f}" for f in flags])

    cmd.append(str(input_path))

    if output_path:
        cmd.append(str(output_path.resolve()))

    env = os.environ.copy()
    if deno_dir_abs:
        env["DENO_DIR"] = str(deno_dir_abs)

    try:
        subprocess.run(cmd,
                       check=True,
                       capture_output=True,
                       text=True,
                       env=env,
                       cwd=cwd)
    except subprocess.CalledProcessError as e:
        print("Error during Deno run:", file=sys.stderr)
        print(f"Command: {' '.join(cmd)}", file=sys.stderr)
        print(f"CWD: {cwd}", file=sys.stderr)
        print(f"Exit code: {e.returncode}", file=sys.stderr)
        if e.stdout:
            print(f"stdout: {e.stdout}", file=sys.stderr)
        if e.stderr:
            print(f"stderr: {e.stderr}", file=sys.stderr)
        raise


def main() -> int:
    try:
        args = parse_args()

        run(
            args.deno_exe,
            args.input,
            args.output,
            args.permissions,
            args.unstable_flags,
            args.deno_dir,
            args.workspace_dir,
        )

        if args.output:
            args.output.resolve().chmod(0o775)
        return 0
    except Exception as e:
        print(f"Error: {str(e)}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
