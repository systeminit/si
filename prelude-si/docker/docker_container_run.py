#!/usr/bin/env python3
"""
Executes a `docker container run`.
"""
import json
import os
import sys
from typing import List, Tuple


def parse_args() -> Tuple[str, List[str], List[str]]:
    this_prog = sys.argv.pop(0)
    if sys.argv.pop(0) != "--tags-file":
        print(
            f"xxx usage: {this_prog} --tags-file TAGS_FILE ARGV...",
            file=sys.stderr,
        )
    tags_file = sys.argv.pop(0)
    docker_args = []
    while True:
        arg = sys.argv.pop(0)
        if arg == "--":
            break
        else:
            docker_args.append(arg)

    return (tags_file, docker_args, sys.argv)


def main() -> int:
    tags_file, docker_args, args = parse_args()

    tags = load_tags(tags_file)

    cmd = [
        "docker",
        "container",
        "run",
        "--rm",
        "--tty",
        "--interactive",
    ]
    cmd.extend(docker_args)
    cmd.append(tags[0])
    cmd.extend(args)

    print("--- Running container: {}".format(" ".join(cmd)))
    os.execvp(cmd[0], cmd)


def load_tags(tags_file: str) -> List[str]:
    with open(tags_file) as file:
        tags = json.load(file)
        return tags


if __name__ == "__main__":
    sys.exit(main())
