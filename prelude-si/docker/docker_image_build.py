#!/usr/bin/env python3
"""
Invokes a `docker image build`.
"""
import argparse
import os
import subprocess
import json
import sys
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


class BaseEnum(Enum, metaclass=MetaEnum):
    pass


class DockerArchitecture(BaseEnum):
    Amd64 = "amd64"
    Arm64v8 = "arm64v8"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--git-info-json",
        required=True,
        help="Path to the Git metadata JSON file",
    )
    parser.add_argument(
        "--artifact-out-file",
        required=True,
        help="Path to write the image artifact file",
    )
    parser.add_argument(
        "--build-metadata-out-file",
        required=True,
        help="Path to write the build metadata JSON file",
    )
    parser.add_argument(
        "--label-metadata-out-file",
        required=True,
        help="Path to write the label metadata JSON file",
    )
    parser.add_argument(
        "--tag-metadata-out-file",
        required=True,
        help="Path to write the tag metadata JSON file",
    )
    parser.add_argument(
        "--docker-context-dir",
        required=True,
        help="Path to Docker context directory",
    )
    parser.add_argument(
        "--image-name",
        required=True,
        help="Name of image to build",
    )
    parser.add_argument(
        "--build-arg",
        action="append",
        help="Build arg options passed to `docker build`",
    )
    parser.add_argument(
        "--author",
        required=True,
        help="Image author to be used in image metadata",
    )
    parser.add_argument(
        "--source-url",
        required=True,
        help="Source code URL to be used in image metadata",
    )
    parser.add_argument(
        "--license",
        required=True,
        help="Image license to be used in image metadata",
    )

    return parser.parse_args()


def main() -> int:
    args = parse_args()

    git_info = load_git_info(args.git_info_json)
    architecture = detect_architecture()
    label_metadata = compute_label_metadata(
        git_info,
        architecture,
        args.image_name,
        args.author,
        args.source_url,
        args.license,
    )
    tag_metadata = compute_tag_metadata(label_metadata, architecture)

    build_image(
        args.docker_context_dir,
        label_metadata,
        args.build_arg or [],
        tag_metadata,
        args.artifact_out_file,
    )
    write_json(args.label_metadata_out_file, label_metadata)
    write_json(args.tag_metadata_out_file, tag_metadata)

    verify_artifact(args.artifact_out_file, tag_metadata)

    b3sum = compute_b3sum(args.artifact_out_file)
    build_metadata = compute_build_metadata(label_metadata, architecture,
                                            b3sum)
    write_json(args.build_metadata_out_file, build_metadata)

    return 0


def build_image(
    cwd: str,
    metadata: Dict[str, str | str],
    build_args: List[str],
    tags: List[str],
    output_file: str,
):
    # Set up buildx builder with docker-container driver
    subprocess.run(
        ["docker", "buildx", "create", "--name", "local-builder", "--use"],
        check=False)  # Don't fail if builder already exists

    # Convert to absolute path and ensure the output directory exists
    abs_output_file = os.path.abspath(output_file)
    output_dir = os.path.dirname(abs_output_file)
    if output_dir:
        os.makedirs(output_dir, exist_ok=True)

    cmd = [
        "docker",
        "buildx",
        "build",
        "--output",
        f"type=docker,dest={abs_output_file}",
    ]
    for key, value in metadata.items():
        cmd.append("--label")
        cmd.append(f"{key}={value}")
    for build_arg in build_args:
        cmd.append("--build-arg")
        # If the build_arg already contains '=', pass it as-is
        # otherwise, pass just the key (for empty values).
        # This allows docker to look up env vars from the host
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

    print("--- Build image with: {}".format(" ".join(cmd)))
    subprocess.run(cmd, cwd=cwd).check_returncode()


def write_json(output: str, metadata: Union[Dict[str, str], List[str]]):
    with open(output, "w") as file:
        json.dump(metadata, file, sort_keys=True)


def verify_artifact(output: str, tag_metadata: List[str]):
    # Artifact is already created by buildx build --output, no need to run docker save
    print("--- Image archive already created at: {}".format(output))
    # Verify the file exists
    if not os.path.exists(output):
        raise FileNotFoundError(
            f"Expected artifact file {output} was not created by buildx build")


# Possible machine architecture detection comes from reading the Rustup shell
# script installer--thank you for your service!
# See: https://github.com/rust-lang/rustup/blob/master/rustup-init.sh
def detect_architecture() -> DockerArchitecture:
    machine = os.uname().machine

    if (machine == "amd64" or machine == "x86_64" or machine == "x86-64"
            or machine == "x64"):
        return DockerArchitecture.Amd64
    elif (machine == "arm64" or machine == "aarch64" or machine == "arm64v8"):
        return DockerArchitecture.Arm64v8
    else:
        print(
            f"xxx Failed to determine architecure or unsupported: {machine}",
            file=sys.stderr,
        )
        sys.exit(1)


def load_git_info(git_info_file: str) -> Dict[str, str | int | bool]:
    with open(git_info_file) as file:
        return json.load(file)


def compute_label_metadata(
    git_info: Dict[str, str | int | bool],
    architecture: DockerArchitecture,
    image_name: str,
    author: str,
    source_url: str,
    license: str,
) -> Dict[str, str]:
    created = git_info.get("committer_date_strict_iso8601")
    revision = git_info.get("commit_hash")
    canonical_version = git_info.get("canonical_version")

    commit_url = "{}/commit/{}".format(
        source_url.removesuffix(".git"),
        revision,
    )

    if git_info.get("is_dirty") and isinstance(revision, str) and isinstance(
            canonical_version, str):
        revision += "-dirty"
        canonical_version += "-dirty"

    image_url = ("https://hub.docker.com/r/{}/" +
                 "tags?page=1&ordering=last_updated&name={}-{}").format(
                     image_name,
                     canonical_version,
                     architecture.value,
                 )

    metadata = {
        "name": image_name,
        "maintainer": author,
        "org.opencontainers.image.version": canonical_version,
        "org.opencontainers.image.authors": author,
        "org.opencontainers.image.licenses": license,
        "org.opencontainers.image.source": source_url,
        "org.opencontainers.image.revision": revision,
        "org.opencontainers.image.created": created,
        "com.systeminit.image.architecture": architecture.value,
        "com.systeminit.image.image_url": image_url,
        "com.systeminit.image.commit_url": commit_url,
    }

    return metadata


def compute_tag_metadata(
    label_metadata: Dict[str, str],
    architecture: DockerArchitecture,
) -> List[str]:
    metadata = [
        "{}:{}-{}".format(
            label_metadata.get("name"),
            label_metadata.get("org.opencontainers.image.version"),
            architecture.value,
        ),
        "{}:sha-{}-{}".format(
            label_metadata.get("name"),
            label_metadata.get("org.opencontainers.image.revision"),
            architecture.value,
        ),
    ]

    return metadata


def compute_b3sum(artifact_file: str) -> str:
    cmd = [
        "b3sum",
        "--no-names",
        artifact_file,
    ]
    result = subprocess.run(cmd, capture_output=True)
    # Print out stderr from process if it failed
    if result.returncode != 0:
        sys.stderr.write(result.stderr.decode("ascii"))
    result.check_returncode()
    b3sum = result.stdout.decode("ascii").rstrip()

    return b3sum


def compute_build_metadata(
    label_metadata: Dict[str, str],
    architecture: DockerArchitecture,
    b3sum: str,
) -> Dict[str, str]:
    metadata = {
        "kind":
        "docker_image",
        "name":
        "{}--{}--{}.tar".format(
            label_metadata.get("name", "UNKNOWN_NAME").replace("/", "--"),
            label_metadata.get("org.opencontainers.image.version"),
            architecture.value,
        ),
        "version":
        label_metadata.get("org.opencontainers.image.version"),
        "architecture":
        architecture.value,
        "commit":
        label_metadata.get("org.opencontainers.image.revision"),
        "b3sum":
        b3sum,
    }

    return metadata


if __name__ == "__main__":
    sys.exit(main())
