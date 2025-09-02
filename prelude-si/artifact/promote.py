#!/usr/bin/env python3
"""
Promotes an artifact to a channel in an object store such as AWS S3.
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
    S3 = "s3"


class PlatformArch(BaseEnum):
    Aarch64 = "aarch64"
    X86_64 = "x86_64"


class PlatformOS(BaseEnum):
    Darwin = "darwin"
    Linux = "linux"


Target = Tuple[PlatformOS, PlatformArch]


class Variant(BaseEnum):
    Binary = "binary"
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
        case Destination.S3:
            s3_promote(
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


def s3_promote(
    channel: str,
    s3_bucket: str,
    artifacts: list[ArtifactMetadata],
    cname: str | None,
    cloudfront_distribution_id: str | None,
):
    if not artifacts:
        return

    bucket_name = re.sub(r"^s3://", "", s3_bucket)

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
            return f"{prefix}.tar.gz"
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


if __name__ == "__main__":
    sys.exit(main())
