#!/usr/bin/env python3
"""
Promotes an artifact to a channel in a destination.
"""
import argparse
import os
import re
import subprocess
import sys
from enum import Enum, EnumMeta
from typing import Any, Tuple
from urllib.parse import urlparse


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


Target = Tuple[PlatformOS, PlatformArch]


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
    ) -> None:
        self.family = family
        self.version = version
        self.variant = variant
        self.os = platform_os
        self.arch = platform_arch


def parse_args() -> tuple[
    argparse.Namespace,
    Destination,
    list[ArtifactMetadata],
]:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--destination",
        required=True,
        help="Destination [examples: {}]".format(", ".join([
            "s3://my-bucket",
            "gcs://bucket-name",
            "docker://docker.io",
        ])),
    )
    parser.add_argument(
        "--channel",
        required=True,
        help="Release channel",
    )
    parser.add_argument(
        "--family",
        required=True,
        help="Artifact family",
    )
    parser.add_argument(
        "--variant",
        required=True,
        type=Variant,
        help="Artifact variant [values: {}]".format(", ".join(
            [m.value for m in Variant])),
    )
    parser.add_argument(
        "--version",
        required=True,
        help="Artifact version",
    )
    parser.add_argument(
        "--target",
        action="append",
        choices=all_target_strs(),
        default=None,
        help="Artifact targets [default: {}]".format(linux_target_strs()),
    )
    parser.add_argument(
        "--organization",
        help="Artifact's organization",
    )
    parser.add_argument(
        "--cname",
        help="URL hostname for artifacts references",
    )
    parser.add_argument(
        "--cloudfront-distribution-id",
        default=os.environ.get("AWS_CLOUDFRONT_DISTRIBUTION_ID"),
        help="AWS Cloudfront distribution ID, used for invalidation",
    )

    args = parser.parse_args()
    destination = None
    for d in Destination:
        if args.destination.startswith(f"{d.value}://"):
            destination = d
    if destination is None:
        parser.error(f"destination scheme not supported: {args.destination}")

    targets = list(map(parse_target, args.target or linux_target_strs()))
    artifacts = list(map(lambda e: parse_metadata(args, e), targets))

    return (args, destination, artifacts)


def main() -> int:
    (args, destination, artifacts) = parse_args()

    match destination:
        case Destination.OCI:
            promote_in_oci_registry(
                args.channel,
                args.destination,
                artifacts,
                args.organization,
            )
        case Destination.S3:
            promote_in_s3(
                args.channel,
                args.destination,
                artifacts,
                args.cname,
                args.cloudfront_distribution_id,
            )

    return 0


def parse_metadata(
    args: argparse.Namespace,
    target: tuple[PlatformOS, PlatformArch],
) -> ArtifactMetadata:
    return ArtifactMetadata(
        args.family,
        args.version,
        args.variant,
        target[0],
        target[1],
    )


def promote_in_s3(
    channel: str,
    destination_prefix: str,
    artifacts: list[ArtifactMetadata],
    cname: str | None,
    cloudfront_distribution_id: str | None,
):
    if not artifacts:
        return

    bucket_name = re.sub(r"^s3://", "", destination_prefix)

    if cname is None:
        base_url = f"https://{bucket_name}.s3.amazonaws.com"
    else:
        base_url = f"https://{cname}"

    md = artifacts[0]
    print(
        f"--- Promoting /{md.family}/{md.version}/{md.variant.value}/* artifacts to '{channel}'"
    )

    artifact_urls = []

    for md in artifacts:
        src_path = object_store_path(md)
        src_url = "/".join([
            base_url,
            src_path,
        ])
        md.version = channel
        dst_path = object_store_path(md)
        dst_url = "/".join([
            base_url,
            dst_path,
        ])

        s3_put_object(
            bucket_name,
            src_url,
            dst_path,
        )
        s3_put_object(
            bucket_name,
            f"{src_url}.metadata.json",
            f"{dst_path}.metadata.json",
        )

        artifact_urls.append(dst_url)

    if cloudfront_distribution_id:
        cloudfront_invalidate_paths(
            cloudfront_distribution_id,
            channel,
            artifact_urls,
        )

    print("\n--- Artifacts promoted")
    for url in artifact_urls:
        print(f"  - {url}")


def promote_in_oci_registry(
    channel: str,
    destination_prefix: str,
    artifacts: list[ArtifactMetadata],
    org: str | None,
):
    if org is None:
        raise ValueError(
            "Missing '--organization' option for oci registry promotion")

    registry = destination_prefix.replace("oci://", "")
    image_arches = [
        map_platform_to_image_arch(md.os, md.arch) for md in artifacts
    ]

    md = artifacts[0]

    images_with_tags = [
        f"{registry}/{org}/{md.family}:{md.version}-{image_arch}"
        for image_arch in image_arches
    ]

    manifest_tag = f"{registry}/{org}/{md.family}:{channel}"

    print(f"--- Promoting images to '{manifest_tag}'")

    print("  - Creating a multi-arch manifest for the following images:")
    for image_with_tag in images_with_tags:
        print(f"      - {image_with_tag}")

    manifest_create_cmd = [
        "docker",
        "manifest",
        "create",
        manifest_tag,
    ]
    for image_with_tag in images_with_tags:
        manifest_create_cmd.append("--amend")
        manifest_create_cmd.append(image_with_tag)
    subprocess.run(manifest_create_cmd).check_returncode()

    print(f"  - Pushing manifest to {manifest_tag}")
    manifest_push_cmd = [
        "docker",
        "manifest",
        "push",
        "--purge",
        manifest_tag,
    ]
    subprocess.run(manifest_push_cmd).check_returncode()

    url = "/".join([
        "https://hub.docker.com",
        "r",
        md.family,
        "&".join([
            "tags?page=1",
            "ordering=last_updated",
            f"name={channel}",
        ]),
    ])

    print("\n--- Artifacts promoted")
    print(f"  - {url}")


def s3_put_object(
    bucket_name: str,
    src_url: str,
    dst_path: str,
):
    print(f"  - /{dst_path} -> {src_url}")

    cmd = [
        "aws",
        "s3api",
        "put-object",
        "--bucket",
        bucket_name,
        "--key",
        dst_path,
        "--website-redirect-location",
        src_url,
    ]
    subprocess.run(cmd).check_returncode()


def cloudfront_invalidate_paths(
    distribution_id: str,
    channel: str,
    artifact_urls: list[str],
):
    invalidation_paths = {
        cloudfront_invalidation_path(e)
        for e in artifact_urls
    }

    print(
        f"--- Invalidating AWS CloudFront paths for '{channel}' channel redirects"
    )
    for path in invalidation_paths:
        print(f"  - {path}")

    cmd = [
        "aws",
        "cloudfront",
        "create-invalidation",
        "--distribution-id",
        distribution_id,
        "--paths",
    ]
    cmd.extend(invalidation_paths)
    subprocess.run(cmd).check_returncode()


def all_target_strs() -> list[str]:
    return [f"{o.value}-{a.value}" for o in PlatformOS for a in PlatformArch]


def linux_target_strs() -> list[str]:
    return [f"{PlatformOS.Linux.value}-{a.value}" for a in PlatformArch]


def parse_target(s: str) -> Target:
    (o, a) = s.split("-", 1)
    return (PlatformOS(o), PlatformArch(a))


def cloudfront_invalidation_path(url_str: str) -> str:
    url = urlparse(url_str)
    path = "/".join([
        # Pop off `$os/$arch/$filename` from the URL path
        os.path.dirname(os.path.dirname(os.path.dirname(url.path))),
        "*",
    ])
    # Builds a "/$family/$channel/$variant/*" path string
    return path


def artifact_name(md: ArtifactMetadata) -> str:
    prefix = f"{md.family}-{md.version}-{md.variant.value}-{md.os.value}-{md.arch.value}"

    match md.variant:
        case Variant.Binary:
            match md.os:
                case PlatformOS.Darwin | PlatformOS.Linux:
                    return f"{prefix}.tar.gz"
                case PlatformOS.Windows:
                    return f"{prefix}.zip"
                case _:
                    raise TypeError(f"unsupport Platform type: {md.os}")
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


if __name__ == "__main__":
    sys.exit(main())
