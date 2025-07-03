#!/usr/bin/env python3
"""
Runs deno cache within a workspace, populating a specific DENO_DIR.
This script does NOT change the current working directory.
"""
import argparse
import os
import pathlib
import shutil
import subprocess
import sys
import tempfile
from typing import List


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--deno-binary",
                        required=True,
                        type=pathlib.Path,
                        help="The path to the deno binary or wrapper script")
    parser.add_argument("--deno-dir",
                        required=True,
                        type=pathlib.Path,
                        help="The output directory for the Deno cache")
    parser.add_argument("--workspace-dir",
                        required=True,
                        type=pathlib.Path,
                        help="The root directory of the Deno workspace to cache")
    parser.add_argument("--input",
                        action="append",
                        required=True,
                        type=pathlib.Path,
                        help="An input file to use as a cache entrypoint.")
    return parser.parse_args()


def run_cache(shim_dir: pathlib.Path,
              deno_dir: pathlib.Path,
              inputs: List[pathlib.Path],
              cwd: pathlib.Path) -> None:
    """
    Run `deno cache` on a given set of input files in a specific directory.
    """
    cmd = ["deno", "cache"]
    cmd.extend([str(p) for p in inputs])

    env = os.environ.copy()
    env["DENO_DIR"] = str(deno_dir)

    original_path = env.get("PATH", "")
    new_path = f"{str(shim_dir)}{os.pathsep}{original_path}"
    env["PATH"] = new_path
    print(new_path)



    subprocess.run(["env"])

    try:
        result = subprocess.run(cmd,
                                check=True,
                                text=True,
                                capture_output=True,
                                env=env,
                                cwd=cwd)
        if result.stdout:
            print(result.stdout)
        if result.stderr:
            print(result.stderr, file=sys.stderr)
    except subprocess.CalledProcessError as e:
        print("Error running deno cache:", file=sys.stderr)
        print(f"Command: {' '.join(cmd)}", file=sys.stderr)
        print(f"Execution Directory: {cwd}", file=sys.stderr)
        print(f"PATH: {env.get('PATH')}", file=sys.stderr)
        print(f"Exit code: {e.returncode}", file=sys.stderr)
        if e.stdout:
            print(f"stdout: {e.stdout}", file=sys.stderr)
        if e.stderr:
            print(f"stderr: {e.stderr}", file=sys.stderr)
        raise


def main() -> int:
    try:
        args = parse_args()

        abs_deno_binary = args.deno_binary.absolute()
        print(abs_deno_binary)
        shim_dir = abs_deno_binary.parent

        abs_input_files = [p.resolve() for p in args.input]
        abs_deno_dir = args.deno_dir.resolve()

        with tempfile.TemporaryDirectory(prefix="deno_cache_build-") as tempdir_str:
            temp_dir = pathlib.Path(tempdir_str)

            # 1. Prepare the execution directory
            shutil.copytree(args.workspace_dir, temp_dir, dirs_exist_ok=True)
            # The logic to copy .mise.toml has been removed.

            # 2. Ensure the final output directory exists
            abs_deno_dir.mkdir(parents=True, exist_ok=True)

            # 3. Run the cache command *in* the temporary directory
            run_cache(shim_dir, abs_deno_dir, abs_input_files, cwd=temp_dir)

        return 0
    except Exception as e:
        print(f"Error: {str(e)}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
