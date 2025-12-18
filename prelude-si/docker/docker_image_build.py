#!/usr/bin/env python3
"""
Invokes a `docker image build`.
"""
import argparse
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


class PlatformArch(BaseEnum):
    Aarch64 = "aarch64"
    X86_64 = "x86_64"


class PlatformOS(BaseEnum):
    Darwin = "darwin"
    Linux = "linux"
    Windows = "windows"


class ImageArch(BaseEnum):
    Amd64 = "amd64"
    Arm64v8 = "arm64v8"


VARIANT = "container"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--artifact-out-file",
        required=True,
        help="Path to write the image artifact file",
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
        help="Name of the image to build, from the full name {org}/{name}",
    )
    parser.add_argument(
        "--org",
        required=True,
        help="Org of the image to build, from the full name {org}/{name}",
    )
    parser.add_argument(
        "--label-metadata-out-file",
        required=True,
        help="Path to write the label metadata JSON file",
    )
    parser.add_argument(
        "--tag-metadata-out-file",
        required=True,
        help="Path to write the tag metadata JSON file",
    )
    parser.add_argument(
        "--docker-context-dir",
        required=True,
        help="Path to Docker context directory",
    )
    parser.add_argument(
        "--build-arg",
        action="append",
        help="Build arg options passed to `docker build`",
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
        help="Target operating system",
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

    return parser.parse_args()


def main() -> int:
    args = parse_args()

    git_info = load_git_info(args.git_info_json)

    # argparse validates these are valid enum values
    arch = PlatformArch(args.arch)
    os = PlatformOS(args.os)

    image_arch = map_platform_to_image_arch(os, arch)

    label_metadata = compute_label_metadata(
        git_info,
        image_arch,
        args.org,
        args.name,
        args.author,
        args.source_url,
        args.license,
    )
    tag_metadata = compute_tag_metadata(label_metadata, image_arch)

    build_image(
        args.docker_context_dir,
        label_metadata,
        args.build_arg or [],
        tag_metadata,
    )
    write_json(args.label_metadata_out_file, label_metadata)
    write_json(args.tag_metadata_out_file, tag_metadata)

    write_artifact(args.artifact_out_file, tag_metadata)

    b3sum = compute_b3sum(args.artifact_out_file)
    build_metadata = compute_build_metadata(label_metadata, image_arch, b3sum)
    write_json(args.build_metadata_out_file, build_metadata)

    return 0


def write_json(output: str, metadata: Union[Dict[str, str], List[str]]):
    with open(output, "w") as file:
        json.dump(metadata, file, sort_keys=True)


def load_git_info(git_info_file: str) -> Dict[str, str | int | bool]:
    with open(git_info_file) as file:
        return json.load(file)


def build_image(
    cwd: str,
    metadata: Dict[str, str | str],
    build_args: List[str],
    tags: List[str],
):
    cmd = [
        "docker",
        "image",
        "build",
    ]
    for key, value in metadata.items():
        cmd.append("--label")
        cmd.append(f"{key}={value}")
    for build_arg in build_args:
        cmd.append("--build-arg")
        # If the build_arg already contains '=', pass it as-is
        # otherwise, pass just the key (for empty values).
        # This allows docker to look up env vars from the host
        if '=' in build_arg and not build_arg.endswith('='):
            cmd.append(build_arg)
        else:
            cmd.append(build_arg.rstrip('='))
    for tag in tags:
        cmd.append("--tag")
        cmd.append(tag)
    cmd.append("--file")
    cmd.append("Dockerfile")
    cmd.append(".")

    print("--- Build image with: {}".format(" ".join(cmd)))
    subprocess.run(cmd, cwd=cwd).check_returncode()


def write_artifact(output: str, tag_metadata: List[str]):
    cmd = [
        "docker",
        "save",
        "--output",
        output,
    ]
    cmd.extend(tag_metadata)

    print("--- Creating image archive with: {}".format(" ".join(cmd)))
    subprocess.run(cmd).check_returncode()


def compute_label_metadata(
    git_info: Dict[str, str | int | bool],
    image_arch: ImageArch,
    image_org: str,
    image_name: str,
    author: str,
    source_url: str,
    license: str,
) -> Dict[str, str]:
    full_image_name = f"{image_org}/{image_name}"

    created = git_info.get("committer_date_strict_iso8601")
    revision = git_info.get("commit_hash")
    canonical_version = git_info.get("canonical_version")

    commit_url = "{}/commit/{}".format(
        source_url.removesuffix(".git"),
        revision,
    )

    image_url = ("https://hub.docker.com/r/{}/" +
                 "tags?page=1&ordering=last_updated&name={}-{}").format(
                     full_image_name,
                     canonical_version,
                     image_arch.value,
                 )

    metadata = {
        "name": full_image_name,
        "maintainer": author,
        "org.opencontainers.image.version": canonical_version,
        "org.opencontainers.image.authors": author,
        "org.opencontainers.image.licenses": license,
        "org.opencontainers.image.source": source_url,
        "org.opencontainers.image.revision": revision,
        "org.opencontainers.image.created": created,
        "com.systeminit.image.architecture": image_arch.value,
        "com.systeminit.image.image_url": image_url,
        "com.systeminit.image.commit_url": commit_url,
    }

    return metadata


def compute_tag_metadata(
    label_metadata: Dict[str, str],
    image_arch: ImageArch,
) -> List[str]:
    metadata = [
        "{}:{}-{}".format(
            label_metadata.get("name"),
            label_metadata.get("org.opencontainers.image.version"),
            image_arch.value,
        ),
    ]

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


def compute_build_metadata_v2(
    git_info: Dict[str, str | int | bool],
    family: str,
    platform_arch: PlatformArch,
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


def map_platform_to_image_arch(
    os: PlatformOS,
    arch: PlatformArch,
) -> ImageArch:
    if os != PlatformOS.Linux:
        raise ValueError(f"Unsupported platform operation system: {os.value}")

    # Map to image arch names
    image_arch_mapping = {
        PlatformArch.X86_64: ImageArch.Amd64,
        PlatformArch.Aarch64: ImageArch.Arm64v8,
    }

    if arch not in image_arch_mapping:
        raise ValueError(f"Unsupported platform architecture: {arch.value}")

    return image_arch_mapping[arch]


def compute_build_metadata(
    label_metadata: Dict[str, str],
    image_arch: ImageArch,
    b3sum: str,
) -> Dict[str, str]:
    metadata = {
        "kind":
        "docker_image",
        "name":
        "{}--{}--{}.tar".format(
            label_metadata.get("name", "UNKNOWN_NAME").replace("/", "--"),
            label_metadata.get("org.opencontainers.image.version"),
            image_arch.value,
        ),
        "version":
        label_metadata.get("org.opencontainers.image.version"),
        "architecture":
        image_arch.value,
        "commit":
        label_metadata.get("org.opencontainers.image.revision"),
        "b3sum":
        b3sum,
    }

    return metadata


if __name__ == "__main__":
    sys.exit(main())
