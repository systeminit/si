#!/usr/bin/env python3
"""
Creates binary artifact archives (tar.gz or zip) with configurable layouts.
"""
import argparse
import os
import subprocess
import sys
import tempfile
import shutil
from pathlib import Path
from enum import Enum, EnumMeta
from typing import Any


# A slightly more Rust-y feeling enum
# Thanks to: https://stackoverflow.com/a/65225753
class MetaEnum(EnumMeta):

    def __contains__(self: type[Any], member: object) -> bool:
        try:
            self(member)
        except ValueError:
            return False
        return True


class BaseEnum(Enum, metaclass=MetaEnum):
    pass


class PlatformOS(BaseEnum):
    Darwin = "darwin"
    Linux = "linux"
    Windows = "windows"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--binary",
        required=True,
        help="Path to the binary file",
    )
    parser.add_argument(
        "--metadata",
        required=True,
        help="Path to the metadata.json file",
    )
    parser.add_argument(
        "--output",
        required=True,
        help="Path to write the output archive",
    )
    parser.add_argument(
        "--os",
        required=True,
        choices=[os.value for os in PlatformOS],
        help="Target operating system (determines archive format)",
    )
    parser.add_argument(
        "--binary-name",
        required=False,
        help=
        "Binary name for usr/local/share path (required with --usr-local-bin)",
    )
    parser.add_argument(
        "--usr-local-bin",
        action="store_true",
        help="Use usr/local/bin structure (default: flat)",
    )

    return parser.parse_args()


def create_flat_archive(args: argparse.Namespace, use_zip: bool) -> None:
    """Create archive with flat structure (binary and metadata.json at root)."""
    binary_basename = os.path.basename(args.binary)

    if use_zip:
        # Create zip archive with flat structure
        # Note: zip doesn't support --transform, so we need a temp directory
        with tempfile.TemporaryDirectory() as tmpdir:
            tmpdir_path = Path(tmpdir)

            # Copy binary with correct name
            shutil.copy2(args.binary, tmpdir_path / binary_basename)
            # Copy metadata with standard name
            shutil.copy2(args.metadata, tmpdir_path / "metadata.json")

            # Create zip from temp directory
            cmd = [
                "zip",
                "-j",  # junk paths (store files at root)
                os.path.abspath(args.output),
                str(tmpdir_path / binary_basename),
                str(tmpdir_path / "metadata.json"),
            ]
            result = subprocess.run(cmd, capture_output=True)
            if result.returncode != 0:
                sys.stderr.write(result.stderr.decode("utf-8"))
                sys.exit(1)
    else:
        # Create tar.gz archive
        cmd = [
            "tar",
            "-czf",
            args.output,
            "--transform",
            f"s|.*{os.path.basename(args.binary)}|{binary_basename}|",
            "--transform",
            "s|.*metadata.json|metadata.json|",
            args.binary,
            args.metadata,
        ]

        result = subprocess.run(cmd, capture_output=True)
        if result.returncode != 0:
            sys.stderr.write(result.stderr.decode("utf-8"))
            sys.exit(1)


def create_nested_archive(args: argparse.Namespace, use_zip: bool) -> None:
    """Create archive with usr/local/bin structure (legacy Rust artifact layout)."""
    binary_basename = os.path.basename(args.binary)

    if use_zip:
        # Create zip archive with directory structure
        with tempfile.TemporaryDirectory() as tmpdir:
            bin_dir = Path(tmpdir) / "usr" / "local" / "bin"
            share_dir = Path(
                tmpdir) / "usr" / "local" / "share" / args.binary_name

            bin_dir.mkdir(parents=True)
            share_dir.mkdir(parents=True)

            # Copy files
            shutil.copy2(args.binary, bin_dir / binary_basename)
            shutil.copy2(args.metadata, share_dir / "metadata.json")

            # Create zip from temp directory
            cmd = [
                "zip",
                "-r",
                os.path.abspath(args.output),
                "usr",
            ]
            subprocess.run(cmd, cwd=tmpdir, check=True)
    else:
        # Create tar.gz archive with transforms
        cmd = [
            "tar",
            "-czf",
            args.output,
            "--transform",
            f"s|.*{binary_basename}|usr/local/bin/{binary_basename}|",
            "--transform",
            f"s|.*metadata.json|usr/local/share/{args.binary_name}/metadata.json|",
            args.binary,
            args.metadata,
        ]

        result = subprocess.run(cmd, capture_output=True)
        if result.returncode != 0:
            sys.stderr.write(result.stderr.decode("utf-8"))
            sys.exit(1)


def main() -> int:
    args = parse_args()

    # Validate usr-local-bin requires binary-name
    if args.usr_local_bin and not args.binary_name:
        print("xxx --usr-local-bin requires --binary-name", file=sys.stderr)
        return 1

    # Determine archive format based on OS (validated by argparse)
    platform_os = PlatformOS(args.os)
    use_zip = platform_os == PlatformOS.Windows

    if args.usr_local_bin:
        create_nested_archive(args, use_zip)
    else:
        create_flat_archive(args, use_zip)

    return 0


if __name__ == "__main__":
    sys.exit(main())
