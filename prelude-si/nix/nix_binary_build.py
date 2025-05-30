#!/usr/bin/env python3
"""
Builds a binary built with Nix but not dependent on Nix packages.
"""
import argparse
import glob
import os
from pathlib import Path
import shutil
import subprocess
import json
import sys
from enum import Enum, EnumMeta
import tempfile
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


VARIANT = "binary"

NIX_PKG_NAME_SUFFIX = "-standalone"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--git-info-json",
        required=True,
        help="Path to the Git metadata JSON file",
    )
    parser.add_argument(
        "--artifact-out-file",
        required=True,
        help="Path to write the binary artifact file",
    )
    parser.add_argument(
        "--build-metadata-out-file",
        required=True,
        help="Path to write the build metadata JSON file",
    )
    parser.add_argument(
        "--binary-metadata-out-file",
        required=True,
        help="Path to write the binary metadata JSON file",
    )
    parser.add_argument(
        "--build-context-dir",
        required=True,
        help="Path to build context directory",
    )
    parser.add_argument(
        "--name",
        required=True,
        help="Name of binary to build",
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
    architecture = detect_architecture()
    os = detect_os()

    nix_store_pkg_path = build_nix_binary(args.name, args.build_context_dir)
    binary_metadata = compute_binary_metadata(
        git_info,
        args.name,
        args.author,
        args.source_url,
        args.license,
        architecture,
        os,
    )
    write_binary(
        args.name,
        Path(args.artifact_out_file),
        Path(nix_store_pkg_path),
    )
    write_json(args.binary_metadata_out_file, binary_metadata)

    b3sum = compute_b3sum(args.artifact_out_file)
    build_metadata = compute_build_metadata(
        git_info,
        args.name,
        architecture,
        os,
        b3sum,
    )
    write_json(args.build_metadata_out_file, build_metadata)

    return 0


def build_nix_binary(name: str, build_context_dir: str) -> str:
    cmd = [
        "nix",
        "build",
        "--impure",
        "--extra-experimental-features",
        "nix-command flakes impure-derivations ca-derivations",
        "--option",
        "filter-syscalls",
        "false",
        "--print-build-logs",
        f".#{name}{NIX_PKG_NAME_SUFFIX}",
    ]

    # We are purposefully escaping the default `$TMPDIR` location as Buck2 sets
    # `$TMPDIR` to be under the `buck-out/` directory. Unfortunately (for us in
    # this context), `nix build` will recursely look up the directory tree
    # searching for a `.git/` directory so we avoid this by running our build
    # under the system's `/tmp/` directory. Sorry folks!
    with tempfile.TemporaryDirectory(
            prefix="/tmp/nix_binary_build-") as tempdir:
        root_dir = os.path.join(tempdir, "root")

        # Copy build context into tempdir
        shutil.copytree(
            build_context_dir,
            root_dir,
            symlinks=True,
        )

        # If we find a workspace dir, let's get the contents into the root
        workspace = os.path.join(root_dir, "workspace")
        if os.path.isdir(workspace):
            print("--- Found a workspace dir, moving contents into root")
            move_contents_up(workspace)

        print("--- Build nix package with: '{}'".format(" ".join(cmd)))
        # Create parent directories
        subprocess.run(cmd, cwd=root_dir).check_returncode()

        nix_store_pkg_path = os.readlink(os.path.join(root_dir, "result"))

    return nix_store_pkg_path


def write_binary(name: str, out_file: Path, nix_store_pkg_path: Path):
    src = nix_store_pkg_path.joinpath("bin", name)
    shutil.copyfile(src, out_file)
    shutil.copymode(src, out_file)


def write_json(output: str, metadata: Union[Dict[str, str], List[str]]):
    with open(output, "w") as file:
        json.dump(metadata, file, sort_keys=True)


# Possible machine architecture detection comes from reading the Rustup shell
# script installer--thank you for your service!
# See: https://github.com/rust-lang/rustup/blob/master/rustup-init.sh
def detect_architecture() -> PlatformArchitecture:
    machine = os.uname().machine

    if (machine == "amd64" or machine == "x86_64" or machine == "x86-64"
            or machine == "x64"):
        return PlatformArchitecture.X86_64
    elif (machine == "arm64" or machine == "aarch64" or machine == "arm64v8"):
        return PlatformArchitecture.Aarch64
    else:
        print(
            f"xxx Failed to determine architecure or unsupported: {machine}",
            file=sys.stderr,
        )
        sys.exit(1)


# Possible machine operating system detection comes from reading the Rustup shell
# script installer--thank you for your service!
# See: https://github.com/rust-lang/rustup/blob/master/rustup-init.sh
def detect_os() -> PlatformOS:
    platform_os = os.uname().sysname

    if (platform_os == "Darwin"):
        return PlatformOS.Darwin
    elif (platform_os == "Linux"):
        return PlatformOS.Linux
    else:
        print(
            f"xxx Failed to determine operating system or unsupported: {platform_os}",
            file=sys.stderr,
        )
        sys.exit(1)


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


def move_contents_up(directory):
    parent_dir = os.path.dirname(directory)
    shutil.copytree(directory, parent_dir, dirs_exist_ok=True)
    shutil.rmtree(directory)


if __name__ == "__main__":
    sys.exit(main())
