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

    return parser.parse_args()


def main() -> int:
    args = parse_args()

    load_artifact(args.artifact_file)
    tags = load_tag_metadata(args.tag_metadata_file)
    upload_image(tags)
    report_metadata(args.label_metadata_file, tags)

    return 0


def load_tag_metadata(tag_metadata_file: str) -> List[str]:
    with open(tag_metadata_file) as file:
        tags = json.load(file)

    return tags


def load_artifact(artifact_file: str):
    cmd = [
        "docker",
        "image",
        "load",
        "--input",
        artifact_file,
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
