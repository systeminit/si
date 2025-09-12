#!/usr/bin/env python3
"""
Invokes `docker image push` commands.
"""
import argparse
import json
import subprocess
import sys
from typing import Dict, List
import os


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--artifact-file",
        required=True,
        help="Path to the image artifact file",
    )
    parser.add_argument(
        "--tag-metadata-file",
        required=True,
        help="Path to the tag metadata JSON file",
    )
    parser.add_argument(
        "--label-metadata-file",
        required=True,
        help="Path to the label metadata JSON file",
    )
    parser.add_argument(
        "--docker-context-dir",
        required=True,
        help="Path to Docker context directory",
    )
    parser.add_argument(
        "--build-arg",
        action="append",
        help="Build arg options passed to buildx",
    )

    return parser.parse_args()


def main() -> int:
    args = parse_args()

    tags = load_tag_metadata(args.tag_metadata_file)
    label_metadata = load_label_metadata(args.label_metadata_file)
    upload_image_with_buildx(tags, args.docker_context_dir, args.build_arg
                             or [], label_metadata)
    report_metadata(args.label_metadata_file, tags)

    return 0


def load_tag_metadata(tag_metadata_file: str) -> List[str]:
    with open(tag_metadata_file) as file:
        tags = json.load(file)

    return tags


def load_label_metadata(label_metadata_file: str) -> Dict[str, str]:
    with open(label_metadata_file) as file:
        return json.load(file)


def upload_image_with_buildx(tags: List[str], docker_context_dir: str,
                             build_args: List[str], label_metadata: Dict[str,
                                                                         str]):
    cmd = [
        "docker",
        "buildx",
        "build",
        "--push",
    ]

    for key, value in label_metadata.items():
        cmd.append("--label")
        cmd.append(f"{key}={value}")

    for build_arg in build_args:
        cmd.append("--build-arg")
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

    print("--- Pushing image with buildx: {}".format(" ".join(cmd)))
    subprocess.run(cmd, cwd=docker_context_dir).check_returncode()


def report_metadata(label_metadata_file: str, tags: List[str]):
    with open(label_metadata_file) as file:
        metadata: Dict[str, str] = json.load(file)

    print("\n--- Image released\n")

    rows = {
        "name": "Image Name",
        "org.opencontainers.image.version": "Version",
        "org.opencontainers.image.revision": "Revision",
        "com.systeminit.image.architecture": "Architecture",
        "com.systeminit.image.commit_url": "Commit URL",
        "com.systeminit.image.image_url": "Docker Hub Image URL",
    }
    header_max_len = max(len(name) for name in rows.values())

    for key, name in rows.items():
        print("    {0:<{1}} : {2}".format(
            name,
            header_max_len,
            metadata.get(key),
        ))
    for i, tag in enumerate(tags):
        print("    {0:<{1}} : {2}".format(
            "Tags" if i == 0 else "",
            header_max_len,
            tag,
        ))


if __name__ == "__main__":
    sys.exit(main())
