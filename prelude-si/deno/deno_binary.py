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
import tempfile
from typing import List, Optional

# Target mapping: canonical target string -> Deno target triple
TARGET_TO_DENO_TRIPLE = {
    "linux-x86_64": "x86_64-unknown-linux-gnu",
    "linux-aarch64": "aarch64-unknown-linux-gnu",
    "darwin-x86_64": "x86_64-apple-darwin",
    "darwin-aarch64": "aarch64-apple-darwin",
    "windows-x86_64": "x86_64-pc-windows-msvc",
    "windows-aarch64": "aarch64-pc-windows-msvc",
}


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
        "--src",
        action="append",
        help="Add a source file into the workspace (Buck2 materialized path)",
    )
    parser.add_argument(
        "--extra-src",
        action="append",
        default=[],
        metavar="DST=SRC",
        help=
        "Extra source file to copy into the workspace (format: src/file.ts=/path/to/file.ts)",
    )
    parser.add_argument(
        "--deno-json",
        type=pathlib.Path,
        default=None,
        help="Path to deno.json configuration file",
    )
    parser.add_argument(
        "--deno-lock",
        type=pathlib.Path,
        default=None,
        help="Path to deno.lock file",
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
    parser.add_argument(
        "--includes",
        nargs="*",
        default=[],
        help="List of files to include in the compilation.",
    )
    parser.add_argument(
        "--target",
        type=str,
        default=None,
        help=
        "Target platform string for cross-compilation (e.g., linux-x86_64, darwin-aarch64)",
    )
    return parser.parse_args()


def target_to_deno_triple(target_str: str) -> str:
    """Convert canonical target string to Deno's target triple format."""
    if target_str not in TARGET_TO_DENO_TRIPLE:
        raise ValueError(
            f"Unsupported target: {target_str}. "
            f"Valid targets: {', '.join(TARGET_TO_DENO_TRIPLE.keys())}")
    return TARGET_TO_DENO_TRIPLE[target_str]


def run_compile(
    deno_binary: pathlib.Path,
    input_path: pathlib.Path,
    output_path: pathlib.Path,
    permissions: List[str],
    flags: List[str],
    deno_dir: Optional[pathlib.Path],
    workspace_dir: Optional[pathlib.Path],
    includes: List[str] = None,
    target: Optional[str] = None,
) -> None:
    """Run deno compile with the specified arguments."""
    deno_binary_abs = deno_binary.resolve()
    output_path_abs = output_path.resolve()
    deno_dir_abs = deno_dir.resolve() if deno_dir else None
    cwd = workspace_dir.resolve() if workspace_dir else pathlib.Path.cwd()

    cmd = [str(deno_binary_abs), "compile", "--output", str(output_path_abs)]

    # Add target flag for cross-compilation
    if target:
        deno_triple = target_to_deno_triple(target)
        cmd.extend(["--target", deno_triple])

    cmd.extend([f"--{p}" for p in permissions])
    cmd.extend([f"--unstable-{f}" for f in flags])

    if includes:
        for include in includes:
            cmd.append("--include")
            cmd.append(include)

    cmd.append(str(input_path))

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
        print("Error compiling Deno binary:", file=sys.stderr)
        print(f"Command: {' '.join(cmd)}", file=sys.stderr)
        print(f"CWD: {cwd}", file=sys.stderr)
        print(f"Exit code: {e.returncode}", file=sys.stderr)
        if e.stdout:
            print(f"stdout: {e.stdout}", file=sys.stderr)
        if e.stderr:
            print(f"stderr: {e.stderr}", file=sys.stderr)
        raise


def main() -> int:
    tempdir = None
    try:
        args = parse_args()

        # Determine if we need to create a temporary workspace
        if not args.workspace_dir:
            # Create temporary build workspace
            tempdir = tempfile.mkdtemp(prefix="deno_build_")
            workspace_path = pathlib.Path(tempdir)

            # Copy deno.json to workspace root (if provided)
            if args.deno_json:
                shutil.copy2(args.deno_json, workspace_path / "deno.json")

            # Copy deno.lock to workspace root (if provided)
            if args.deno_lock:
                shutil.copy2(args.deno_lock, workspace_path / "deno.lock")

            # Copy all source files into workspace
            # Infer destination from source path (relative to cell root)
            cell_root = pathlib.Path.cwd()
            for src_path_str in (args.src or []):
                src_path = pathlib.Path(src_path_str).resolve()

                # Make relative to cell root to get destination path
                try:
                    rel_path = src_path.relative_to(cell_root)
                except ValueError:
                    # Source is outside cell root (shouldn't happen), use basename
                    rel_path = src_path.name
                dst_path = workspace_path / rel_path
                dst_path.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(src_path, dst_path)

            # Copy extra_src files/directories into workspace using DST=SRC mapping
            input_parent = workspace_path / args.input.parent
            for extra_src_mapping in (args.extra_src or []):
                dest, src = extra_src_mapping.split("=", 1)
                src_path = pathlib.Path(src)
                # Destination is relative to input file's parent directory
                dest_path = input_parent / dest
                dest_path.parent.mkdir(parents=True, exist_ok=True)

                # Handle both files and directories
                if src_path.is_dir():
                    shutil.copytree(src_path, dest_path, dirs_exist_ok=True)
                else:
                    shutil.copy2(src_path, dest_path)

            # Set workspace and input for compilation
            workspace_dir = workspace_path
            input_path = args.input  # Relative path within workspace
        else:
            # Workspace provided - use existing approach
            workspace_dir = args.workspace_dir
            input_path = args.input

            # Copy deno.json and deno.lock if provided
            if args.deno_json:
                shutil.copy2(args.deno_json, workspace_dir / "deno.json")
            if args.deno_lock:
                shutil.copy2(args.deno_lock, workspace_dir / "deno.lock")

            # Copy source files/directories into workspace
            cell_root = pathlib.Path.cwd()
            for src_path_str in (args.src or []):
                src_path = pathlib.Path(src_path_str).resolve()
                try:
                    rel_path = src_path.relative_to(cell_root)
                except ValueError:
                    rel_path = src_path.name
                dst_path = workspace_dir / rel_path
                dst_path.parent.mkdir(parents=True, exist_ok=True)

                # Handle both files and directories
                if src_path.is_dir():
                    shutil.copytree(src_path, dst_path, dirs_exist_ok=True)
                else:
                    shutil.copy2(src_path, dst_path)

            # Copy extra_src files/directories using DST=SRC mapping
            input_parent = workspace_dir / args.input.parent
            for extra_src_mapping in (args.extra_src or []):
                dest, src = extra_src_mapping.split("=", 1)
                src_path = pathlib.Path(src)
                # Destination is relative to input file's parent directory
                dest_path = input_parent / dest
                dest_path.parent.mkdir(parents=True, exist_ok=True)

                # Handle both files and directories
                if src_path.is_dir():
                    shutil.copytree(src_path, dest_path, dirs_exist_ok=True)
                else:
                    shutil.copy2(src_path, dest_path)

        # Build include files list for deno compile --include
        # Use the destination paths from the DST=SRC mappings
        include_files = [
            str(args.input.parent / mapping.split("=", 1)[0])
            for mapping in (args.extra_src or [])
        ]

        # Run compilation
        run_compile(
            args.deno_exe,
            input_path,
            args.output,
            args.permissions,
            args.unstable_flags,
            args.deno_dir,
            workspace_dir,
            include_files,
            args.target,
        )

        args.output.resolve().chmod(0o775)

        # Cleanup temp workspace if we created it
        if tempdir:
            shutil.rmtree(tempdir)

        return 0
    except Exception as e:
        # Cleanup temp directory on error
        if tempdir:
            shutil.rmtree(tempdir, ignore_errors=True)
        print(f"Error: {str(e)}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
