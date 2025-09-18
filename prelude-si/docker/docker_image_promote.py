#!/usr/bin/env python3
"""
Promotes a multi-arch Docker image to a tag.
"""
import argparse
import subprocess
import sys
from enum import Enum, EnumMeta
from typing import Any, List


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


class DockerArchitecture(BaseEnum):
    Amd64 = "amd64"
    Arm64v8 = "arm64v8"


DEFAULT_MULTI_ARCH = [
    DockerArchitecture.Amd64.value,
    DockerArchitecture.Arm64v8.value,
]
DEFAULT_STABLE_TAG = "stable"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--stable-tag",
        default=DEFAULT_STABLE_TAG,
        help="Stable tag name [default: '{}']".format(DEFAULT_STABLE_TAG),
    )
    parser.add_argument(
        "--update-stable-tag",
        action="store_true",
        help="Whether or not to update stable tag",
    )
    parser.add_argument(
        "--multi-arch",
        action="append",
        default=[],
        help="""Specify the exact multi-arch platforms that are supported
        [default: {}]""".format(DEFAULT_MULTI_ARCH),
    )
    parser.add_argument(
        "image_name",
        help="Docker image name, minus tag",
    )
    parser.add_argument(
        "src_tag",
        help="Docker image tag to promote",
    )

    return parser.parse_args()


def main() -> int:
    args = parse_args()

    if len(args.multi_arch) == 0:
        multi_arches = DEFAULT_MULTI_ARCH
    else:
        multi_arches = args.multi_arch

    for multi_arch in multi_arches:
        if not multi_arch in DockerArchitecture:
            print(
                f"xxx Multi-arch value is invalid: {multi_arch}",
                file=sys.stderr,
            )
            sys.exit(1)

    src_prefix = "{}:{}".format(args.image_name, args.src_tag)
    dst = src_prefix

    promote(src_prefix, dst, multi_arches)

    if args.update_stable_tag:
        dst = "{}:{}".format(args.image_name, args.stable_tag)
        promote(src_prefix, dst, multi_arches)

    image_url = ("https://hub.docker.com/r/{}/" +
                 "tags?page=1&ordering=last_updated&name={}").format(
                     args.image_name,
                     args.src_tag,
                 )

    print("\n--- Image promoted\n")
    print(f"    Docker Hub Image URL : {image_url}")

    if args.update_stable_tag:
        image_url = ("https://hub.docker.com/r/{}/" +
                     "tags?page=1&ordering=last_updated&name={}").format(
                         args.image_name,
                         args.stable_tag,
                     )
        print(f"    Docker Hub Stable Image URL : {image_url}")

    return 0


def promote(src_prefix: str, dst: str, multi_arches: List[DockerArchitecture]):
    srcs = list(map(lambda arch: f"{src_prefix}-{arch}", multi_arches))

    print(f"--- Promoting '{src_prefix}-*' images to '{dst}'")

    print(f"  - Creating manifest list '{dst}' for the following images:")
    for src in srcs:
        print(f"      - {src}")

    imagetools_cmd = [
        "docker",
        "buildx",
        "imagetools",
        "create",
        "--tag",
        dst,
    ]
    imagetools_cmd.extend(srcs)

    print(
        f"  - Creating and pushing manifest list with: {' '.join(imagetools_cmd)}"
    )
    subprocess.run(imagetools_cmd).check_returncode()


if __name__ == "__main__":
    sys.exit(main())
