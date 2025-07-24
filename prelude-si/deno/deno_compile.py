#!/usr/bin/env python3
"""
Builds a portable, standalone deno binary.
"""
import argparse
import os
import pathlib
import shutil
import stat
import subprocess
import sys
from typing import List


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input",
                        required=True,
                        type=pathlib.Path,
                        help="The path to compile, ideally a index.ts file")
    parser.add_argument(
        "--extra-srcs",
        nargs='*',
        default=[],
        help="Sources that will be copied into the source tree",
    )
    parser.add_argument(
        "--output",
        required=True,
        type=pathlib.Path,
        help="The target directory for outputting the artifact")
    parser.add_argument(
        "--permissions",
        nargs='*',
        default=[],
        help="List of Deno permissions to grant (e.g., read write net)",
    )
    parser.add_argument(
        "--unstable-flags",
        nargs='*',
        default=[],
        help="List of unstable flags to enable",
    )

    return parser.parse_args()


def parse_permissions(perms: List[str]) -> List[str]:
    """Convert permission names to Deno CLI flags."""
    return [f'--{perm}' for perm in perms]


def parse_unstable_flags(flags: List[str]) -> List[str]:
    """Convert unstable flag names to Deno CLI flags."""
    return [f'--unstable-{flag}' for flag in flags]


def run_compile(input_path: str, output_path: str, permissions: List[str],
                flags: List[str], includes: List[str]) -> None:
    """Run deno compile with the specified arguments."""
    os.makedirs(os.path.dirname(output_path), exist_ok=True)
    cmd = ["deno", "compile", "--output", output_path]

    if permissions:
        cmd.extend(permissions)

    if flags:
        cmd.extend(flags)

    if includes:
        for include in includes:
            cmd.append("--include")
            cmd.append(include)

    cmd.append(input_path)

    try:
        subprocess.run(cmd, check=True, capture_output=True, text=True)
    except subprocess.CalledProcessError as e:
        print("Error compiling Deno binary:", file=sys.stderr)
        print(f"Command: {' '.join(cmd)}", file=sys.stderr)
        print(f"Exit code: {e.returncode}", file=sys.stderr)
        print(f"stdout: {e.stdout}", file=sys.stderr)
        print(f"stderr: {e.stderr}", file=sys.stderr)
        raise


def main() -> int:
    try:
        args = parse_args()

        abs_input_path = os.path.abspath(args.input)
        abs_output_path = os.path.abspath(args.output)

        if not os.path.exists(abs_input_path):
            print(f"Error: Input file not found: {abs_input_path}",
                  file=sys.stderr)
            return 1

        os.makedirs(os.path.dirname(abs_output_path), exist_ok=True)

        permissions_list = parse_permissions(args.permissions)
        flags_list = parse_unstable_flags(args.unstable_flags)
        for src in args.extra_srcs:
            shutil.copy2(src, args.input.resolve().parent)

        include_files = [
            os.path.join(args.input.resolve().parent, os.path.basename(src))
            for src in args.extra_srcs
        ]
        run_compile(abs_input_path, abs_output_path, permissions_list,
                    flags_list, include_files)

        os.chmod(
            abs_output_path,
            stat.S_IRUSR
            | stat.S_IXUSR
            | stat.S_IRGRP
            | stat.S_IXGRP
            | stat.S_IROTH
            | stat.S_IXOTH,
        )

        return 0

    except Exception as e:
        print(f"Error: {str(e)}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
