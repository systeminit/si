#!/usr/bin/env python3
"""
Runs Deno tests.
"""
import argparse
import os
import pathlib
import shutil
import subprocess
import sys
from typing import List


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--deno-binary",
        required=True,
        type=pathlib.Path,
        help="The path to the deno binary",
    )
    parser.add_argument(
        "--input",
        action="append",
        required=True,
        type=pathlib.Path,
        help="The test file to run (can be specified multiple times)",
    )
    parser.add_argument(
        "--extra-srcs",
        nargs='*',
        default=[],
        help="Sources that will be copied into the source tree",
    )
    parser.add_argument(
        "--filter",
        type=str,
        help="Run tests with this string or pattern in the test name",
    )
    parser.add_argument("--ignore",
                        nargs="*",
                        default=[],
                        help="List of files or directories to ignore")
    parser.add_argument("--parallel",
                        action="store_true",
                        help="Run tests in parallel")
    parser.add_argument("--watch",
                        action="store_true",
                        help="Watch for file changes and restart tests")
    parser.add_argument(
        "--permissions",
        nargs="*",
        default=[],
        help="List of Deno permissions to grant (e.g., read write net)",
    )
    parser.add_argument(
        "--unstable-flags",
        nargs="*",
        default=[],
        help="List of unstable flags to enable",
    )
    parser.add_argument(
        "--env",
        nargs="*",
        default=[],
        help="Environment variables to set (format: KEY=VALUE)",
    )
    parser.add_argument("--no-check",
                        action="store_true",
                        help="Skip type checking")

    return parser.parse_args()


def parse_permissions(perms: List[str]) -> List[str]:
    """Convert permission names to Deno CLI flags."""
    return [f"--{perm}" for perm in perms]


def parse_unstable_flags(flags: List[str]) -> List[str]:
    """Convert unstable flag names to Deno CLI flags."""
    return [f"--unstable-{flag}" for flag in flags]


def run_tests(
    deno_binary: str,
    input_paths: List[str],
    filter_pattern: str | None,
    flags: List[str],
    ignore_paths: List[str],
    parallel: bool,
    permissions: List[str],
    watch: bool,
    env_vars: List[str],
    no_check: bool,
) -> None:
    """Run deno test with the specified arguments."""
    cmd = [str(deno_binary), "test"]

    # Auto-detect deno.json in the test file's directory tree
    if input_paths:
        test_file = pathlib.Path(input_paths[0]).resolve()
        # Walk up the directory tree to find deno.json
        current_dir = test_file.parent
        while current_dir != current_dir.parent:
            deno_config = current_dir / "deno.json"
            if deno_config.exists():
                cmd.extend(["--config", str(deno_config)])
                break
            current_dir = current_dir.parent

    if filter_pattern:
        cmd.extend(["--filter", filter_pattern])

    for ignore in ignore_paths:
        cmd.append(f"--ignore={ignore}")

    if parallel:
        cmd.append("--parallel")

    if watch:
        cmd.append("--watch")

    if no_check:
        cmd.append("--no-check")

    if permissions:
        cmd.extend(permissions)

    if flags:
        cmd.extend(flags)

    cmd.extend(input_paths)

    # Parse environment variables
    env = os.environ.copy()
    for env_var in env_vars:
        if "=" in env_var:
            key, value = env_var.split("=", 1)
            env[key] = value

    try:
        result = subprocess.run(cmd, check=True, text=True, env=env)
        if result.stdout:
            print(result.stdout)
    except subprocess.CalledProcessError as e:
        print("Error running deno test:", file=sys.stderr)
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

        # Copy extra sources to src/ directory relative to project root
        # This makes generated files (like bundle.js) available to source files
        if args.extra_srcs and args.input:
            # Find the project root by locating deno.json
            test_file = pathlib.Path(args.input[0]).resolve()
            project_root = None
            current_dir = test_file.parent
            while current_dir != current_dir.parent:
                deno_config = current_dir / "deno.json"
                if deno_config.exists():
                    project_root = current_dir
                    break
                current_dir = current_dir.parent

            if project_root:
                # Copy extra sources to the src/ directory
                src_dir = project_root / "src"
                src_dir.mkdir(parents=True, exist_ok=True)
                for src in args.extra_srcs:
                    src_path = pathlib.Path(src)
                    target_path = src_dir / src_path.name
                    shutil.copy2(src, target_path)
            else:
                # Fallback to old behavior if no deno.json found
                input_dir = pathlib.Path(args.input[0]).resolve().parent
                input_dir.mkdir(parents=True, exist_ok=True)
                for src in args.extra_srcs:
                    shutil.copy2(src, input_dir)

        input_paths = []
        for input_path in args.input:
            abs_path = os.path.abspath(input_path)
            if not os.path.exists(abs_path):
                print(f"Error: Input path not found: {abs_path}",
                      file=sys.stderr)
                return 1
            input_paths.append(abs_path)

        permissions_list = parse_permissions(args.permissions)
        flags_list = parse_unstable_flags(args.unstable_flags)

        run_tests(
            args.deno_binary,
            input_paths,
            args.filter,
            flags_list,
            args.ignore,
            args.parallel,
            permissions_list,
            args.watch,
            args.env,
            args.no_check,
        )

        print("Tests completed successfully.")
        return 0

    except Exception as e:
        print(f"Error: {str(e)}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
