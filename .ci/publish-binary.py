#!/usr/bin/env python3
"""
Publishes a release of a binary.
"""
import argparse
import subprocess
import sys
from typing import List, Optional


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--binary",
        default="si-ci",
        help="Name of binary program",
    )
    parser.add_argument(
        "--version",
        required=True,
        help="Version string used to create release tag (i.e. \"v$VERSION\")",
    )
    parser.add_argument(
        "--verbose",
        action="store_true",
        help="Increases verbosity of program output",
    )
    parser.add_argument(
        "assets",
        help="File assets to include in GitHub release",
        nargs=argparse.REMAINDER,
    )

    return parser.parse_args()


def main() -> int:
    args = parse_args()

    tag = create_git_tag(args.binary, args.version, args.verbose)
    create_github_release(tag, args.assets or [], args.verbose)

    return 0


def create_git_tag(program: str, version: str, verbose: bool) -> str:
    tag = f"{version}"

    section(f":git: Creating Git tag: {tag}")
    git_tag_cmd = [
        "git",
        "tag",
        "--annotate",
        tag,
        "--message",
        f"release: {program} {version}",
    ]
    if verbose:
        debug(cmd_args_str(git_tag_cmd))
    run_or_die(git_tag_cmd)

    info("Pushing Git tag to origin")
    git_push_cmd = [
        "git",
        "push",
        "origin",
        tag,
    ]
    if verbose:
        debug(cmd_args_str(git_push_cmd))
    run_or_die(git_push_cmd)

    return tag


def create_github_release(tag: str, assets: List[str], verbose: bool):
    section(f":github: Creating GitHub release: {tag}")
    for asset in assets:
        info(f"Including asset: {asset}")

    cmd = [
        "gh",
        "release",
        "create",
        "--draft",
        "--generate-notes",
        tag,
    ] + assets
    if verbose:
        debug(cmd_args_str(cmd))
    run_or_die(cmd)


def cmd_args_str(cmd: List[str]) -> str:
    return " ".join(map(lambda s: f"'{s}'", cmd))


def section(msg: str):
    print(f"--- {msg}", file=sys.stdout)


def info(msg: str):
    print(f"  - {msg}", file=sys.stdout)


def debug(msg: str):
    print(f"  + {msg}", file=sys.stderr)


def run_or_die(cmd: List[str], cwd: Optional[str] = None):
    exit_code = subprocess.call(cmd, cwd=cwd)
    if exit_code != 0:
        msg = "xxx Command exited non-zero ({}): {}\nxxx Program aborted.".format(
            exit_code,
            " ".join(cmd),
        )
        print(msg, file=sys.stderr)
        sys.exit(exit_code)


if __name__ == "__main__":
    sys.exit(main())
