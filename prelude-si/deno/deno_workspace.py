#!/usr/bin/env python3
"""
Creates a Deno workspace by reading the root deno.json, copying member
package files, and running `deno cache` on all source files that are
explicitly part of the workspace members.
"""
import argparse
import json
import os
import pathlib
import shutil
import subprocess
import sys


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--root-config",
        required=True,
        type=pathlib.Path,
        help="The root deno.json file.",
    )
    parser.add_argument(
        "--workspace-dir",
        required=True,
        type=pathlib.Path,
        help="The output directory for the workspace.",
    )
    parser.add_argument(
        "--deno-binary",
        required=True,
        type=pathlib.Path,
        help="The path to the deno binary or wrapper script.",
    )
    parser.add_argument(
        "--deno-dir",
        required=True,
        type=pathlib.Path,
        help="The output directory for the Deno cache.",
    )
    parser.add_argument(
        "--src",
        action="append",
        default=[],
        type=pathlib.Path,
        help="A directory of source files to include.",
    )
    return parser.parse_args()


def run_cache(
    deno_binary: pathlib.Path,
    deno_dir: pathlib.Path,
    workspace_dir: pathlib.Path,
    inputs: list[pathlib.Path],
) -> None:
    """Run `deno cache` on a given set of input files."""
    if not inputs:
        print("No input files found to cache. Skipping.")
        return

    cmd = [
        str(deno_binary), "cache", "--config", "deno.json",
        "--node-modules-dir"
    ]
    cmd.extend([str(p) for p in inputs])

    env = os.environ.copy()
    env["DENO_DIR"] = str(deno_dir)

    try:
        result = subprocess.run(cmd,
                                check=True,
                                text=True,
                                capture_output=True,
                                env=env,
                                cwd=workspace_dir)
        if result.stdout:
            print(result.stdout)
        if result.stderr:
            print(result.stderr, file=sys.stderr)
        print(
            f"Successfully cached {len(inputs)} files and created node_modules directory."
        )
    except subprocess.CalledProcessError as e:
        print("Error running deno cache:", file=sys.stderr)
        print(f"Command: {' '.join(cmd)}", file=sys.stderr)
        print(f"Exit code: {e.returncode}", file=sys.stderr)
        if e.stdout:
            print(f"stdout: {e.stdout}", file=sys.stderr)
        if e.stderr:
            print(f"stderr: {e.stderr}", file=sys.stderr)
        raise


def main() -> int:
    try:
        args = parse_args()
        workspace_dir = args.workspace_dir.resolve()
        workspace_dir.mkdir(parents=True, exist_ok=True)

        shutil.copy2(args.root_config, workspace_dir / "deno.json")

        with open(args.root_config) as f:
            root_config_json = json.load(f)
        workspace_member_globs = root_config_json.get("workspace", [])
        member_paths = [g.lstrip("./") for g in workspace_member_globs]

        for src_dir in args.src:
            if not src_dir.is_dir():
                continue

            member_path_str = None
            for member in member_paths:
                if member in str(src_dir):
                    member_path_str = member
                    break

            if not member_path_str:
                continue

            dest_root = workspace_dir / member_path_str
            dest_root.mkdir(parents=True, exist_ok=True)

            shutil.copytree(src_dir, dest_root, dirs_exist_ok=True)

        print(f"Workspace assembled at: {workspace_dir}")

        files_to_cache = []
        for suffix in (".ts", ".tsx", ".js", ".mjs"):
            files_to_cache.extend(workspace_dir.rglob(f"*{suffix}"))

        files_to_cache_relative = [
            p.relative_to(workspace_dir) for p in files_to_cache
        ]

        print(
            f"Found {len(files_to_cache_relative)} source files in workspace to cache."
        )

        abs_deno_dir = args.deno_dir.resolve()
        abs_deno_dir.mkdir(parents=True, exist_ok=True)

        run_cache(
            args.deno_binary.absolute(),
            abs_deno_dir,
            workspace_dir,
            files_to_cache_relative,
        )

        return 0
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        import traceback

        traceback.print_exc()
        return 1


if __name__ == "__main__":
    sys.exit(main())
