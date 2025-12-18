#!/usr/bin/env python3
"""
Creates binary artifact archives (tar.gz or zip) with configurable layouts.
"""
import argparse
import json
import os
import subprocess
import sys
import tempfile
import shutil
from pathlib import Path
from enum import Enum, EnumMeta
from typing import Any, Dict, List, Union


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


class PlatformArch(BaseEnum):
    Aarch64 = "aarch64"
    X86_64 = "x86_64"


class PlatformOS(BaseEnum):
    Darwin = "darwin"
    Linux = "linux"
    Windows = "windows"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--name",
        required=True,
        help="Name of the binary",
    )
    parser.add_argument(
        "--binary",
        required=True,
        help="Path to the binary file",
    )
    parser.add_argument(
        "--git-info-json",
        required=True,
        help="Path to the Git metadata JSON file",
    )
    parser.add_argument(
        "--artifact-out-file",
        required=True,
        help="Path to write the output archive",
    )
    parser.add_argument(
        "--pkg-metadata-out-file",
        required=True,
        help="Path to the metadata.json file",
    )
    parser.add_argument(
        "--arch",
        required=True,
        choices=[arch.value for arch in PlatformArch],
        help="Target architecture",
    )
    parser.add_argument(
        "--os",
        required=True,
        choices=[os.value for os in PlatformOS],
        help="Target operating system (determines archive format)",
    )
    parser.add_argument(
        "--author",
        required=True,
        help="Image author to be used in image metadata",
    )
    parser.add_argument(
        "--source-url",
        required=True,
        help="Source code URL to be used in image metadata",
    )
    parser.add_argument(
        "--license",
        required=True,
        help="Image license to be used in image metadata",
    )
    parser.add_argument(
        "--binary-name",
        required=False,
        help="""Binary name for usr/local/share path (required with
        --usr-local-bin)""",
    )
    parser.add_argument(
        "--usr-local-bin",
        action="store_true",
        help="Use usr/local/bin structure (default: flat)",
    )

    return parser.parse_args()


def main() -> int:
    args = parse_args()

    git_info = load_git_info(args.git_info_json)

    # Validate usr-local-bin requires binary-name
    if args.usr_local_bin and not args.binary_name:
        print("xxx --usr-local-bin requires --binary-name", file=sys.stderr)
        return 1

    # Determine archive format based on OS (validated by argparse)
    platform_os = PlatformOS(args.os)
    platform_arch = PlatformArch(args.arch)

    use_zip = platform_os == PlatformOS.Windows

    b3sum = compute_b3sum(args.binary)
    pkg_metadata = compute_pkg_metadata(
        git_info,
        args.name,
        args.author,
        args.source_url,
        args.license,
        platform_os,
        platform_arch,
        b3sum,
    )
    write_json(args.pkg_metadata_out_file, pkg_metadata)

    if args.usr_local_bin:
        create_nested_archive(args, use_zip)
    else:
        create_flat_archive(args, use_zip)

    return 0


def write_json(output: str, metadata: Union[Dict[str, str], List[str]]):
    with open(output, "w") as file:
        json.dump(metadata, file, sort_keys=True)


def load_git_info(git_info_file: str) -> Dict[str, str | int | bool]:
    with open(git_info_file) as file:
        return json.load(file)


def compute_b3sum(artifact_file: str) -> str:
    cmd = [
        "b3sum",
        "--no-names",
        artifact_file,
    ]
    result = subprocess.run(cmd, capture_output=True)
    # Print out stderr from process if it failed
    if result.returncode != 0:
        sys.stderr.write(result.stderr.decode("ascii"))
    result.check_returncode()
    b3sum = result.stdout.decode("ascii").rstrip()

    return b3sum


def compute_pkg_metadata(
    git_info: Dict[str, str | int | bool],
    name: str,
    author: str,
    source_url: str,
    license: str,
    platform_os: PlatformOS,
    platform_arch: PlatformArch,
    b3sum: str,
) -> Dict[str, str]:
    metadata = {
        "name": name,
        "version": git_info.get("canonical_version"),
        "author": author,
        "source_url": source_url,
        "license": license,
        "architecture": platform_arch.value,
        "os": platform_os.value,
        "commit": git_info.get("commit_hash"),
        "branch": git_info.get("branch"),
        "b3sum": b3sum,
    }

    return metadata


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
            shutil.copy2(
                args.pkg_metadata_out_file,
                tmpdir_path / "metadata.json",
            )

            # Create zip from temp directory
            cmd = [
                "zip",
                "-j",  # junk paths (store files at root)
                os.path.abspath(args.artifact_out_file),
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
            args.artifact_out_file,
            "--transform",
            f"s|.*{os.path.basename(args.binary)}|{binary_basename}|",
            "--transform",
            "s|.*metadata.json|metadata.json|",
            args.binary,
            args.pkg_metadata_out_file,
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
            shutil.copy2(
                args.pkg_metadata_out_file,
                share_dir / "metadata.json",
            )

            # Create zip from temp directory
            cmd = [
                "zip",
                "-r",
                os.path.abspath(args.artifact_out_file),
                "usr",
            ]
            subprocess.run(cmd, cwd=tmpdir, check=True)
    else:
        # Create tar.gz archive with transforms
        cmd = [
            "tar",
            "-czf",
            args.artifact_out_file,
            "--transform",
            f"s|.*{binary_basename}|usr/local/bin/{binary_basename}|",
            "--transform",
            f"s|.*metadata.json|usr/local/share/{args.binary_name}/metadata.json|",
            args.binary,
            args.pkg_metadata_out_file,
        ]

        result = subprocess.run(cmd, capture_output=True)
        if result.returncode != 0:
            sys.stderr.write(result.stderr.decode("utf-8"))
            sys.exit(1)


if __name__ == "__main__":
    sys.exit(main())
