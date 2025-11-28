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
            "gcs://bucket-name",
            "docker://docker.io",
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
        case Destination.S3:
            url = "/".join([args.destination, object_store_path(md)])
            print("--- Publishing {}".format(os.path.basename(url)))
            s3_upload(args.artifact_file, url)
            s3_upload(args.metadata_file, url + ".metadata.json")
            s3_report_metadata(md, url, args.cname)

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


def s3_upload(artifact_path, s3_url):
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

    return None


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


if __name__ == "__main__":
    sys.exit(main())
