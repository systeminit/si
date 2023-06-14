#!/usr/bin/env python3
"""
Promotes a Docker image tag (based on Git SHA) to a stable tag.
"""
import argparse
import subprocess
import sys


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--stable-tag",
        default="stable",
        help="Stable tag name [default: `stable`]",
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

    src = "{}:sha-{}".format(args.image_name, args.src_tag)
    dst = "{}:{}".format(args.image_name, args.stable_tag)

    print(f"--- Promoting '{src}' to '{dst}'")

    print(f"  - Pulling image tagged with {src}")
    subprocess.run(["docker", "image", "pull", src]).check_returncode()

    print(f"  - Tagging image {src} -> {dst}")
    subprocess.run(["docker", "image", "tag", src, dst]).check_returncode()

    print(f"  - Pushing image tag {dst}")
    subprocess.run(["docker", "image", "push", dst]).check_returncode()

    return 0


if __name__ == "__main__":
    sys.exit(main())
