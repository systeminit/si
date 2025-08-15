#!/usr/bin/env python3
"""
Checks the format of TOML files.
"""
import argparse
import os
from pathlib import Path
import subprocess
import sys


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--check",
        action="store_true",
    )
    parser.add_argument(
        "--root-dir",
        required=True,
        help="Path to the top of the sources tree",
    )
    parser.add_argument(
        "--taplo-path",
        help="Path to the taplo binary",
        default="taplo",
    )
    parser.add_argument(
        "--cargo-path",
        help="Path to the cargo binary",
        default="cargo",
    )
    parser.add_argument(
        "--cargo-sort-path",
        help="Path to the cargo-sort binary",
        default="cargo-sort",
    )
    parser.add_argument(
        "srcs",
        nargs="+",
        help="Source to process",
    )

    return parser.parse_args()


def main() -> int:
    print(sys.argv)
    args = parse_args()

    failed_cargo_tomls = []
    failed_tomls = []

    for file in args.srcs or []:
        path = Path(file)
        abs_path = os.path.join(args.root_dir, path)

        if os.path.isdir(abs_path):
            continue

        if any(path.name == x for x in [".taplo.toml", "tomlfmt.toml"]):
            continue

        if path.name == "Cargo.toml":
            # Only run cargo-sort if the binary is actually available (not a dummy)
            if "dummy" not in args.cargo_sort_path:
                returncode = cargo_sort_file(path, args.root_dir, args.check,
                                             args.cargo_sort_path)
                if returncode != 0:
                    failed_cargo_tomls.append(path)
            else:
                print(
                    f"  - {path} [SKIPPED: cargo-sort not available for this platform]"
                )

        returncode = taplo_fmt_file(path, args.root_dir, args.check,
                                    args.taplo_path)
        if returncode != 0:
            failed_tomls.append(path)

    if failed_cargo_tomls or failed_tomls:
        if args.check:
            print("\nxxx Some TOML files failed their format checks\nxxx",
                  file=sys.stderr)
        else:
            print("\nxxx Some TOML files failed to format\nxxx",
                  file=sys.stderr)

        if failed_cargo_tomls:
            print("xxx Cargo.toml files:", file=sys.stderr)
            for cargo_toml in failed_cargo_tomls:
                print("xxx    - {}".format(cargo_toml), file=sys.stderr)
        if failed_tomls:
            print("xxx TOML files:", file=sys.stderr)
            for toml in failed_tomls:
                print("xxx    - {}".format(toml), file=sys.stderr)

        print("\nxxx Consider running the appropriate Buck2 'fix' target",
              "if defined, such as `:fix-format-toml`",
              file=sys.stderr)

        return 1
    else:
        if args.check:
            print("\n--- TOML files passed format checks")
        else:
            print("\n--- TOML files formated", file=sys.stderr)

        return 0


def cargo_sort_file(cargo_toml_path: Path, cwd: Path, is_check: bool,
                    cargo_sort_path: str) -> int:
    cmd = [
        cargo_sort_path,
    ]
    if is_check:
        cmd.extend([
            "--check",
            "--check-format",
        ])
    cmd.append(os.path.join(".", cargo_toml_path))

    print("  - {} [`{}`]".format(cargo_toml_path, " ".join(cmd)))

    print("[== output: start ==]")
    result = subprocess.run(cmd, cwd=cwd)
    print("[== output: end ==]")

    return result.returncode


def taplo_fmt_file(toml_path: Path, cwd: Path, is_check: bool,
                   taplo_path: str) -> int:
    cmd = [
        taplo_path,
        "fmt",
    ]
    if is_check:
        cmd.extend([
            "--check",
        ])
    cmd.append(os.path.join(".", toml_path))

    print("  - {} [`{}`]".format(toml_path, " ".join(cmd)))

    print("[== output: start ==]")
    result = subprocess.run(cmd, cwd=cwd)
    print("[== output: end ==]")

    return result.returncode


if __name__ == "__main__":
    sys.exit(main())
