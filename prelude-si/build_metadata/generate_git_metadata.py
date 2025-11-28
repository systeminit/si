#!/usr/bin/env python3
"""
Generates Git metdata.
"""
import argparse
from datetime import datetime, timezone
import json
import subprocess
import sys
from typing import Any, Dict

ABBREVIATED_COMMIT_HASH = "abbreviated_commit_hash"
BRANCH_NAME = "branch"
CAL_VER = "cal_ver"
CANONICAL_VERSION = "canonical_version"
COMMITER_DATE_STRICT = "committer_date_strict_iso8601"
COMMITER_DATE_TIMESTAMP = "committer_date_timestamp"
COMMIT_HASH = "commit_hash"
IS_DIRTY = "is_dirty"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "output",
        nargs="?",
        default="-",
        help="Output file (value of `-` writes to stdout)",
    )
    args = parser.parse_args()

    data = {}
    data.update(parse_git_branch())
    data.update(parse_git_status())
    data.update(parse_git_show())
    finalize(data)

    if args.output == "-":
        json.dump(data, sys.stdout, sort_keys=True)
    else:
        with open(args.output, "w") as file:
            json.dump(data, file, sort_keys=True)

    return 0


def parse_git_branch() -> Dict[str, str]:
    git_branch_command = [
        "git",
        "branch",
        "--show-current",
    ]
    result = subprocess.run(git_branch_command, capture_output=True, text=True)
    result.check_returncode()

    return {BRANCH_NAME: result.stdout.strip()}


def parse_git_show() -> Dict[str, str]:
    format_str = json.dumps(
        {
            ABBREVIATED_COMMIT_HASH: "%h",
            COMMITER_DATE_STRICT: "%cI",
            COMMIT_HASH: "%H",
        },
        indent=0,
        separators=(',', ':'),
    ).replace("\n", "")
    git_show_cmd = [
        "git",
        "show",
        "--no-patch",
        "--abbrev=8",
        f"--format=format:{format_str}",
    ]
    result = subprocess.run(git_show_cmd, capture_output=True)
    result.check_returncode()

    return json.loads(result.stdout)


def parse_git_status() -> Dict[str, Any]:
    git_status_cmd = [
        "git",
        "status",
        "--porcelain",
        "--ignore-submodules",
        "-unormal",
    ]
    result = subprocess.run(git_status_cmd, capture_output=True)
    result.check_returncode()

    return {
        IS_DIRTY: True if result.stdout else False,
    }


def finalize(data: Dict[str, Any]):
    abbreviated_commit_hash = data.get(
        ABBREVIATED_COMMIT_HASH,
        "HASH-NOT-FOUND",
    )
    abbreviated_commit_hash = abbreviated_commit_hash[:8]
    dt_str = data.get(COMMITER_DATE_STRICT)
    is_dirty = data.get(IS_DIRTY)

    if dt_str:
        if is_dirty == True:
            # Set commit date to now since the repo is dirty
            dt_utc = datetime.utcnow()
        else:
            # Convert into UTC
            dt_utc = datetime.fromisoformat(dt_str).astimezone(timezone.utc)

        cal_ver = dt_utc.strftime("%Y%m%d.%H%M%S.0")
        canonical_version = f"{cal_ver}-sha.{abbreviated_commit_hash}"
        if is_dirty == True:
            canonical_version += "_dirty"
            data.update({
                COMMIT_HASH: "{}_dirty".format(data.get(COMMIT_HASH)),
            })

        data.update({
            CAL_VER: cal_ver,
            CANONICAL_VERSION: canonical_version,
            COMMITER_DATE_STRICT: dt_utc.strftime("%Y%m%dT%H:%M:%SZ"),
            COMMITER_DATE_TIMESTAMP: round(dt_utc.timestamp()),
        })


if __name__ == "__main__":
    sys.exit(main())
