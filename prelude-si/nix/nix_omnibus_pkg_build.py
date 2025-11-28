#!/usr/bin/env python3
"""
Builds a package of Nix packages with a primary program entrypoint.
"""
import argparse
import glob
import os
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
    Windows = "windows"


VARIANT = "omnibus"


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
        help="Path to write the image artifact file",
    )
    parser.add_argument(
        "--build-metadata-out-file",
        required=True,
        help="Path to write the build metadata JSON file",
    )
    parser.add_argument(
        "--pkg-metadata-out-file",
        required=True,
        help="Path to write the package metadata JSON file",
    )
    parser.add_argument(
        "--build-context-dir",
        required=True,
        help="Path to build context directory",
    )
    parser.add_argument(
        "--name",
        required=True,
        help="Name of package to build",
    )
    parser.add_argument(
        "--author",
        required=True,
        help="Author to be used in package metadata",
    )
    parser.add_argument(
        "--source-url",
        required=True,
        help="Source code URL to be used in package metadata",
    )
    parser.add_argument(
        "--license",
        required=True,
        help="Image license to be used in package metadata",
    )

    return parser.parse_args()


def main() -> int:
    args = parse_args()

    git_info = load_git_info(args.git_info_json)
    architecture = detect_architecture()
    os = detect_os()

    nix_store_pkg_path = build_nix_package(args.name, args.build_context_dir)
    nix_pkgs_closure = compute_nix_pkgs_closure(nix_store_pkg_path)
    pkg_metadata = compute_pkg_metadata(
        git_info,
        args.name,
        args.author,
        args.source_url,
        args.license,
        architecture,
        os,
        nix_store_pkg_path,
        nix_pkgs_closure,
    )
    build_tar_archive(
        args.artifact_out_file,
        nix_store_pkg_path,
        nix_pkgs_closure,
        pkg_metadata,
    )

    write_json(args.pkg_metadata_out_file, pkg_metadata)

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


def build_nix_package(name: str, build_context_dir: str) -> str:
    cmd = [
        "nix",
        "build",
        "--extra-experimental-features",
        "nix-command flakes impure-derivations ca-derivations",
        "--option",
        "filter-syscalls",
        "false",
        "--print-build-logs",
        f".#{name}",
    ]

    # We are purposefully escaping the default `$TMPDIR` location as Buck2 sets
    # `$TMPDIR` to be under the `buck-out/` directory. Unfortunately (for us in
    # this context), `nix build` will recursely look up the directory tree
    # searching for a `.git/` directory so we avoid this by running our build
    # under the system's `/tmp/` directory. Sorry folks!
    with tempfile.TemporaryDirectory(
            prefix="/tmp/nix_pkg_tar_build-") as tempdir:
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


def compute_nix_pkgs_closure(nix_store_pkg_path: str) -> List[str]:
    cmd = [
        "nix-store",
        "--query",
        "--requisites",
        nix_store_pkg_path,
    ]

    result = subprocess.run(cmd, capture_output=True)
    # Print out stderr from process if it failed
    if result.returncode != 0:
        sys.stderr.write(result.stderr.decode("ascii"))
    result.check_returncode()

    pkgs_closure = result.stdout.decode("ascii").splitlines()
    pkgs_closure.sort()

    return pkgs_closure


def build_tar_archive(
    out_file: str,
    nix_store_pkg_path: str,
    pkgs_closure: List[str],
    pkg_metadata: Dict[str, str],
):
    temp_file = os.path.join(
        os.path.dirname(out_file),
        ".{}".format(os.path.basename(out_file)).rstrip(".gz"),
    )

    tar_create_cmd = [
        "tar",
        "-cpf",
        temp_file,
    ]
    tar_create_cmd.extend(pkgs_closure)
    subprocess.run(tar_create_cmd).check_returncode()

    with tempfile.TemporaryDirectory() as tempdir:
        bin_dir = os.path.join(
            tempdir,
            "usr",
            "local",
            "bin",
        )
        cache_dir = os.path.join(tempdir, "cache")
        lib_dir = os.path.join(tempdir, "lib")
        lib64_dir = os.path.join(tempdir, "lib64")
        metadata_dir = os.path.join(
            tempdir,
            "etc",
            "nix-omnibus",
            pkg_metadata.get("name", "UNKNOWN-NAME"),
            pkg_metadata.get("version", "UNKNOWN-VERSION"),
        )

        os.makedirs(bin_dir, exist_ok=True)
        os.makedirs(metadata_dir, exist_ok=True)

        binaries = glob.glob(os.path.join(nix_store_pkg_path, "bin", "*"))
        for binary in binaries:
            os.symlink(binary, os.path.join(bin_dir, os.path.basename(binary)))

        if os.path.exists(os.path.join(nix_store_pkg_path, "cache")):
            os.makedirs(cache_dir, exist_ok=True)
            for root, _, files in os.walk(os.path.join(nix_store_pkg_path,
                                                       "cache"),
                                          followlinks=True):
                relative_root = os.path.relpath(
                    root, os.path.join(nix_store_pkg_path, "cache"))
                destination_root = os.path.join(cache_dir, relative_root)
                os.makedirs(destination_root, exist_ok=True)

                for file in files:
                    source = os.path.join(root, file)
                    destination = os.path.join(destination_root, file)

                    # If it's a symlink, get the real target
                    if os.path.islink(source):
                        target = os.readlink(source)
                        os.symlink(target, destination)
                    else:
                        os.symlink(source, destination)

        # This ensures that if the omnibus pkg bubbles up lib64, it gets
        # included assuming it is needed in environments that require
        # specifiying a dynamic linker
        if os.path.exists(os.path.join(nix_store_pkg_path, "lib")):
            os.makedirs(lib_dir, exist_ok=True)
            for root, _, files in os.walk(os.path.join(nix_store_pkg_path,
                                                       "lib"),
                                          followlinks=True):
                for file in files:
                    source = os.path.join(root, file)
                    # If it's a symlink, get the real target
                    if os.path.islink(source):
                        target = os.readlink(source)
                    else:
                        target = source
                    os.symlink(target, os.path.join(lib_dir, file))

            # Create lib64 -> lib symlink
            os.symlink("lib", lib64_dir)

        write_json(os.path.join(metadata_dir, "metadata.json"), pkg_metadata)

        tar_append_cmd = [
            "tar",
            "-rpf",
            os.path.abspath(temp_file),
            "--owner=0",
            "--group=0",
            "usr",
            "etc",
        ]
        if os.path.exists(os.path.join(tempdir, "lib")):
            tar_append_cmd.extend(["lib", "lib64"])

        if os.path.exists(os.path.join(nix_store_pkg_path, "cache")):
            tar_append_cmd.extend(["cache"])

        subprocess.run(tar_append_cmd, cwd=tempdir).check_returncode()

    # Remember that by default `gzip` creates a *new* file and appends the
    # `.gz` file extension, so in terms of this operation it looks like a
    # rename/move
    gzip_cmd = [
        "gzip",
        "-9",
        "-f",
        temp_file,
    ]
    subprocess.run(gzip_cmd).check_returncode()

    # Atomically move temp file to out file
    os.rename(f"{temp_file}.gz", out_file)


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

    match platform_os:
        case "Darwin":
            return PlatformOS.Darwin
        case "Linux":
            return PlatformOS.Linux
        case "Windows":
            return PlatformOS.Windows
        case _:
            print(
                f"xxx Failed to determine operating system or unsupported: {platform_os}",
                file=sys.stderr,
            )
            sys.exit(1)


def load_git_info(git_info_file: str) -> Dict[str, str | int | bool]:
    with open(git_info_file) as file:
        return json.load(file)


def compute_pkg_metadata(
    git_info: Dict[str, str | int | bool],
    name: str,
    author: str,
    source_url: str,
    license: str,
    architecture: PlatformArchitecture,
    platform_os: PlatformOS,
    nix_store_pkg_path: str,
    pkgs_closure: List[str],
) -> Dict[str, str]:
    binaries = glob.glob(os.path.join(nix_store_pkg_path, "bin", "*"))

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
        "binaries": binaries,
        "nix_closure": pkgs_closure
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
