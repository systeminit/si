#!/usr/bin/env python3
"""
Publishes an artifact to a destination.
"""
import argparse
import os
import pathlib
import re
import subprocess
import sys
import json
from enum import Enum, EnumMeta
from typing import Any, Dict


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


class Destination(BaseEnum):
    OCI = "oci"
    S3 = "s3"


class PlatformArch(BaseEnum):
    Aarch64 = "aarch64"
    X86_64 = "x86_64"


class PlatformOS(BaseEnum):
    Darwin = "darwin"
    Linux = "linux"
    Windows = "windows"


class Variant(BaseEnum):
    Binary = "binary"
    Container = "container"
    Omnibus = "omnibus"
    Rootfs = "rootfs"


class ArtifactMetadata(object):

    def __init__(
        self,
        family: str,
        version: str,
        variant: Variant,
        platform_os: PlatformOS,
        platform_arch: PlatformArch,
        b3sum: str,
        commit: str,
        extra: Dict[str, Any],
    ) -> None:
        self.family = family
        self.version = version
        self.variant = variant
        self.os = platform_os
        self.arch = platform_arch
        self.b3sum = b3sum
        self.commit = commit
        self.extra = extra


def parse_args() -> tuple[argparse.Namespace, Destination]:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--destination",
        required=True,
        help="Destination [examples: {}]".format(", ".join([
            "s3://my-bucket",
            "oci://docker.io",
            "gcs://bucket-name",
        ])),
    )
    parser.add_argument(
        "--artifact-file",
        required=True,
        type=pathlib.Path,
        help="Path to the artifact file to be actioned on.",
    )
    parser.add_argument(
        "--metadata-file",
        required=True,
        type=pathlib.Path,
        help="Path to the metadata of the artifact to be actioned on.",
    )
    parser.add_argument(
        "--cname",
        help="URL hostname for artifacts references",
    )

    args = parser.parse_args()
    destination = None
    for d in Destination:
        if args.destination.startswith(f"{d.value}://"):
            destination = d
    if destination is None:
        parser.error(f"destination scheme not supported: {args.destination}")

    return (args, destination)


def main() -> int:
    (args, destination) = parse_args()

    md = load_metadata(args.metadata_file)

    match destination:
        case Destination.OCI:
            publish_to_oci_registry(
                md,
                args.artifact_file,
                args.destination,
            )
        case Destination.S3:
            publish_to_s3(
                md,
                args.artifact_file,
                args.metadata_file,
                args.destination,
                args.cname,
            )

    return 0


def load_metadata(json_file: str) -> ArtifactMetadata:
    with open(json_file) as file:
        data = json.load(file)
    return ArtifactMetadata(
        data.pop("family"),
        data.pop("version"),
        Variant(data.pop("variant")),
        PlatformOS(data.pop("os")),
        PlatformArch(data.pop("arch")),
        data.pop("b3sum"),
        data.pop("commit"),
        data,
    )


def publish_to_s3(
    md: ArtifactMetadata,
    artifact_path: pathlib.Path,
    metadata_path: pathlib.Path,
    destination_prefix: str,
    cname: str | None,
):
    url = "/".join([destination_prefix, object_store_path(md)])

    print("--- Publishing {}".format(os.path.basename(url)))
    s3_upload(artifact_path, url)
    s3_upload(metadata_path, url + ".metadata.json")
    s3_report_metadata(md, url, cname)


def publish_to_oci_registry(
    md: ArtifactMetadata,
    artifact_path: pathlib.Path,
    destination_prefix: str,
):
    registry = destination_prefix.replace("oci://", "")
    image_arch = map_platform_to_image_arch(md.os, md.arch)
    image_with_tag = f"{registry}/{md.family}:{md.version}-{image_arch}"

    print(f"--- Publishing {image_with_tag}")
    load_and_push_container_image(artifact_path, image_with_tag)
    container_report_metadata(md, registry, image_arch)


def s3_upload(artifact_path: pathlib.Path, s3_url: str):
    cmd = [
        "aws",
        "s3",
        "cp",
        artifact_path,
        s3_url,
    ]
    print(f"  - Uploading to {s3_url}")
    subprocess.run(cmd).check_returncode()


def s3_report_metadata(md: ArtifactMetadata, url: str, cname: str | None):
    print("\n--- Artifact published\n")

    if cname:
        url = re.sub(r"^s3://[^/]+/", f"https://{cname}/", url)

    rows = {
        "Family": md.family,
        "Version": md.version,
        "Variant": md.variant.value,
        "OS": md.os.value,
        "Arch": md.arch.value,
        "Blake3Sum": md.b3sum,
        "Revision": md.commit,
        "Artifact URL": url,
        "Metadata URL": f"{url}.metadata.json",
    }
    header_max_len = max(len(name) for name in rows.keys())

    for name, value in rows.items():
        print("    {0:<{1}} : {2}".format(
            name,
            header_max_len,
            value,
        ))


def artifact_name(md: ArtifactMetadata) -> str:
    prefix = "-".join([
        md.family,
        md.version,
        md.variant.value,
        md.os.value,
        md.arch.value,
    ])

    match md.variant:
        case Variant.Binary:
            match md.os:
                case PlatformOS.Darwin | PlatformOS.Linux:
                    return f"{prefix}.tar.gz"
                case PlatformOS.Windows:
                    return f"{prefix}.zip"
        case Variant.Omnibus:
            return f"{prefix}.tar.gz"
        case Variant.Rootfs:
            return f"{prefix}.ext4"
        case _:
            raise TypeError(f"unsupport Variant type: {md.variant}")


def object_store_path(md: ArtifactMetadata) -> str:
    return "/".join([
        md.family,
        md.version,
        md.variant.value,
        md.os.value,
        md.arch.value,
        artifact_name(md),
    ])


def map_platform_to_image_arch(os: PlatformOS, arch: PlatformArch) -> str:
    if os != PlatformOS.Linux:
        raise ValueError(f"Unsupported platform operation system: {os.value}")

    # Map to Docker arch names
    docker_arch_mapping = {
        PlatformArch.X86_64: "amd64",
        PlatformArch.Aarch64: "arm64v8",
    }

    if arch not in docker_arch_mapping:
        raise ValueError(f"Unsupported platform architecture: {arch.value}")

    return docker_arch_mapping[arch]


def load_and_push_container_image(
    artifact_path: pathlib.Path,
    image_with_tag: str,
):
    # Load OCI tarball into Docker Engine
    load_cmd = [
        "docker",
        "load",
        "--input",
        str(artifact_path),
    ]
    print(f"  - Loading OCI tarball: {artifact_path}")
    subprocess.run(load_cmd).check_returncode()

    push_cmd = [
        "docker",
        "push",
        image_with_tag,
    ]
    print(f"  - Pushing container image: {image_with_tag}")
    subprocess.run(push_cmd).check_returncode()


def container_report_metadata(
    md: ArtifactMetadata,
    registry: str,
    image_arch: str,
):
    print("\n--- Artifact published\n")

    url = None
    if registry == "docker.io":
        url = "/".join([
            "https://hub.docker.com",
            "r",
            md.family,
            "&".join([
                "tags?page=1",
                "ordering=last_updated",
                f"name={md.version}-{image_arch}",
            ]),
        ])
    else:
        raise ValueError(f"Unsupported container registry: {registry}")

    rows = {
        "Family": md.family,
        "Version": md.version,
        "Variant": md.variant.value,
        "OS": md.os.value,
        "Arch": md.arch.value,
        "Blake3Sum": md.b3sum,
        "Revision": md.commit,
        "Artifact URL": url,
    }
    header_max_len = max(len(name) for name in rows.keys())

    for name, value in rows.items():
        print("    {0:<{1}} : {2}".format(
            name,
            header_max_len,
            value,
        ))


if __name__ == "__main__":
    sys.exit(main())
