#!/usr/bin/env python3
"""
Publishes an artifact to our Production S3 Bucket for mass Distribution
"""
import argparse
import subprocess
import sys
import os
import json
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


def sync_to_s3(localfile, url):
    aws_cli_command = ['aws', 's3', 'cp', localfile, f'{url}']
    subprocess.run(aws_cli_command, check=True)
    print(f'Successfully synced {localfile} to {url}')

def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--artifact-file",
        required=True,
        help="Path to the artifact file to be actioned on.",
    )
    parser.add_argument(
        "--metadata-file",
        required=True,
        help="Path to the metadata of the artifact to be actioned on.",
    )
    parser.add_argument(
        "--family",
        required=True,
        help="Family of the artifact.",
    )
    parser.add_argument(
        "--variant",
        required=True,
        help="Variant of the artifact.",
    )
    parser.add_argument(
        "--destination",
        required=True,
        help="Variant of the artifact.",
    )
    return parser.parse_args()


def load_json_file(json_file: str) -> Dict[str, str | int | bool]:
    with open(json_file) as file:
        return json.load(file)


def craft_url(bucket: str, metadata: Dict[str, str]):
    # This avoids having to remove the last / off the end of the path, even
    # though it's a bit awkward

    crafted_url = "/".join([bucket, metadata["family"], metadata["os"], metadata["architecture"], metadata["variant"],metadata["name"]])
    return crafted_url

def main() -> int:
    args = parse_args()
    metadata = load_json_file(args.metadata_file)
    url = craft_url(args.destination, metadata)
    sync_to_s3(args.artifact_file, url)
    return 0

if __name__ == "__main__":
    sys.exit(main())
