#!/usr/bin/env python3
"""
Builds a metadata file for a binary.
"""
import argparse
import os
import subprocess
import json
import sys
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


class PlatformArchitecture(BaseEnum):
    Aarch64 = "aarch64"
    X86_64 = "x86_64"


class PlatformOS(BaseEnum):
    Darwin = "darwin"
    Linux = "linux"
    Windows = "windows"


VARIANT = "binary"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--binary",
        required=True,
        help="Path to the binary to compute the b3sum",
    )
    parser.add_argument(
        "--git-info-json",
        required=True,
        help="Path to the Git metadata JSON file",
    )
    parser.add_argument(
        "--build-metadata-out-file",
        required=True,
        help="Path to write the build metadata JSON file",
    )
    parser.add_argument(
        "--name",
        required=True,
        help="Name of binary to build",
    )
    parser.add_argument(
        "--arch",
        required=True,
        choices=[arch.value for arch in PlatformArchitecture],
        help="Target architecture",
    )
    parser.add_argument(
        "--os",
        required=True,
        choices=[os.value for os in PlatformOS],
        help="Target operating system",
    )
    parser.add_argument(
        "--author",
        required=True,
        help="Author to be used in binary metadata",
    )
    parser.add_argument(
        "--source-url",
        required=True,
        help="Source code URL to be used in binary metadata",
    )
    parser.add_argument(
        "--license",
        required=True,
        help="Image license to be used in binary metadata",
    )

    return parser.parse_args()


def main() -> int:
    args = parse_args()

    git_info = load_git_info(args.git_info_json)

    # argparse validates these are valid enum values
    architecture = PlatformArchitecture(args.arch)
    os = PlatformOS(args.os)

    b3sum = compute_b3sum(args.binary)
    build_metadata = compute_build_metadata(
        git_info,
        args.name,
        architecture,
        os,
        b3sum,
    )
    write_json(args.build_metadata_out_file, build_metadata)

    return 0


def write_json(output: str, metadata: Union[Dict[str, str], List[str]]):
    with open(output, "w") as file:
        json.dump(metadata, file, sort_keys=True)


def load_git_info(git_info_file: str) -> Dict[str, str | int | bool]:
    with open(git_info_file) as file:
        return json.load(file)


def compute_binary_metadata(
    git_info: Dict[str, str | int | bool],
    name: str,
    author: str,
    source_url: str,
    license: str,
    architecture: PlatformArchitecture,
    platform_os: PlatformOS,
) -> Dict[str, str]:
    metadata = {
        "name": name,
        "version": git_info.get("canonical_version"),
        "author": author,
        "source_url": source_url,
        "license": license,
        "architecture": architecture.value,
        "os": platform_os.value,
        "commit": git_info.get("commit_hash"),
        "branch": git_info.get("branch"),
    }

    return metadata


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


def compute_build_metadata(
    git_info: Dict[str, str | int | bool],
    family: str,
    platform_arch: PlatformArchitecture,
    platform_os: PlatformOS,
    b3sum: str,
) -> Dict[str, str]:
    metadata = {
        "family": family,
        "variant": VARIANT,
        "version": git_info.get("canonical_version"),
        "arch": platform_arch.value,
        "os": platform_os.value,
        "commit": git_info.get("commit_hash"),
        "branch": git_info.get("branch"),
        "b3sum": b3sum,
    }

    return metadata


if __name__ == "__main__":
    sys.exit(main())
