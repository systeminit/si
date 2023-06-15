#!/usr/bin/env python3
"""
Invokes `docker image push` commands.
"""
import argparse
import json
import subprocess
import sys
from typing import Dict, List


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--archive-file",
        required=True,
        help="Path to the image archive file",
    )
    parser.add_argument(
        "--tags-file",
        required=True,
        help="Path to the tags JSON file",
    )
    parser.add_argument(
        "--metadata-file",
        required=True,
        help="Path to the metadata JSON file",
    )

    return parser.parse_args()


def main() -> int:
    args = parse_args()

    load_archive(args.archive_file)
    tags = loads_tags(args.tags_file)
    upload_image(tags)
    report_metadata(args.metadata_file)

    return 0


def loads_tags(tags_file: str) -> List[str]:
    with open(tags_file) as file:
        tags = json.load(file)

    return tags


def load_archive(archive_file: str):
    cmd = [
        "docker",
        "image",
        "load",
        "--input",
        archive_file,
    ]
    print("--- Loading image archive with: {}".format(" ".join(cmd)))
    subprocess.run(cmd).check_returncode()


def upload_image(tags: List[str]):
    cmd_prefix = [
        "docker",
        "image",
        "push",
    ]

    print("--- Uploading image via tags")
    for tag in tags:
        cmd = cmd_prefix + [tag]
        print(f"  - Pushing tag '{tag}'")
        subprocess.run(cmd).check_returncode()


def report_metadata(metadata_file: str):
    with open(metadata_file) as file:
        metadata: Dict[str, str] = json.load(file)

    print("\n--- Image released\n")

    rows = {
        "name": "Image Name",
        "org.opencontainers.image.version": "Version",
        "org.opencontainers.image.revision": "Revision",
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


if __name__ == "__main__":
    sys.exit(main())
