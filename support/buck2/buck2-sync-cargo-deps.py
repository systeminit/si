#!/usr/bin/env python3
"""
Updates Cargo Dependencies As Buck2 Rust Third Party Targets.
"""
import argparse
from enum import StrEnum
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile


class TomlSection(StrEnum):
    Dependencies = "[dependencies]\n"
    WorkspaceDependencies = "[workspace.dependencies]\n"
    WorkspacePackage = "[workspace.package]\n"
    PatchCratesIO = "[patch.crates-io]\n"


class Comment(StrEnum):
    Begin = "# BEGIN: DEPENDENCIES\n"
    End = "# END: DEPENDENCIES\n"


REMEDIATE_MSG = "    Run: `buck2 run support/buck2:sync-cargo-deps` to resolve."


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--check",
        action="store_true",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()

    cargo_lock_updated = update_cargo_lock(
        Path("Cargo.toml"),
        Path("Cargo.lock"),
        args.check,
    )
    if cargo_lock_updated and args.check:
        print("xxx Rust Cargo.lock is not in sync with Cargo.toml")
        print(REMEDIATE_MSG)
        return 1

    cargo_toml_updated = update_third_party_rust_cargo_toml(
        Path("Cargo.toml"),
        Path("third-party/rust/Cargo.toml"),
        args.check,
    )
    if cargo_toml_updated and args.check:
        print("xxx Third-party Rust Cargo.toml is not in sync with Cargo.toml")
        print(REMEDIATE_MSG)
        return 1

    cargo_lock_updated = update_third_party_rust_cargo_lock(
        Path("third-party/rust/Cargo.toml"),
        Path("Cargo.lock"),
        Path("third-party/rust/Cargo.lock"),
        args.check,
    )
    if cargo_lock_updated and args.check:
        print("xxx Third-party Rust Cargo.lock is not in sync with Cargo.lock")
        print(REMEDIATE_MSG)
        return 1

    buck_file_updated = reindeer_buckify(
        Path("third-party/rust/BUCK"),
        args.check,
    )
    if buck_file_updated and args.check:
        print("xxx Third-party Rust BUCK file is not up to date")
        print(REMEDIATE_MSG)
        return 1

    if cargo_toml_updated or buck_file_updated:
        print("--- Cargo dependencies finished syncing with source changes")
        print("    Make sure to add these changes and commit to Git.")
    elif args.check:
        print("--- Cargo dependencies are in sync with third-party Rust")
    else:
        print("--- Cargo dependencies are in sync, with no sources changed")
        print("    Nothing to do!")

    return 0


def update_cargo_lock(
    cargo_toml_file: Path,
    cargo_lock_file: Path,
    check_mode: bool,
) -> bool:
    if check_mode:
        print("--- Checking {} in sync with {}".format(
            cargo_lock_file,
            cargo_toml_file,
        ))
    else:
        print("--- Updating {} from {}".format(
            cargo_lock_file,
            cargo_toml_file,
        ))

    with tempfile.NamedTemporaryFile(mode="w+") as backup_lock:
        # Make backup of Cargo.lock file
        with open(
                cargo_lock_file,
                encoding="utf-8",
        ) as src:
            shutil.copyfileobj(src, backup_lock)
            backup_lock.flush()
            backup_lock.seek(0)

        # https://users.rust-lang.org/t/check-if-the-cargo-lock-is-up-to-date-without-building-anything/91048/5
        cmd = [
            "cargo",
            "update",
            "--workspace",
            "--locked",
        ]
        result = subprocess.run(cmd)
        if result.returncode != 0:
            # Restore Cargo.lock file from backup
            with open(
                    cargo_lock_file,
                    mode="w+",
                    encoding="utf-8",
            ) as dst:
                backup_lock.seek(0)
                shutil.copyfileobj(backup_lock, dst)
                dst.flush()
        result.check_returncode()

        file_modified = False

        cmd = [
            "diff",
            "-u",
            backup_lock.name,
            cargo_lock_file,
        ]
        result = subprocess.run(cmd, capture_output=True)
        if result.returncode == 1:
            print("  - Regenerated {} has changed with diff:\n".format(
                cargo_lock_file))
            print(result.stdout.decode("ascii"))
            file_modified = True

            if check_mode:
                # Restore Cargo.lock file from backup
                with open(
                        cargo_lock_file,
                        mode="w+",
                        encoding="utf-8",
                ) as dst:
                    backup_lock.seek(0)
                    shutil.copyfileobj(backup_lock, dst)
                    dst.flush()
        elif result.returncode != 0:
            print("xxx diff command failed", file=sys.stderr)
            print("xxx --- stdout:", file=sys.stderr)
            print(result.stdout.decode("ascii"), file=sys.stderr)
            print("xxx --- stderr:", file=sys.stderr)
            print(result.stderr.decode("ascii"), file=sys.stderr)
        else:
            print("  - No changes detected for {}".format(cargo_lock_file))

    return file_modified


def update_third_party_rust_cargo_toml(
    src_cargo_toml_file: Path,
    dst_cargo_toml_file: Path,
    check_mode: bool,
) -> bool:
    if check_mode:
        print("--- Checking {} in sync with {}".format(
            dst_cargo_toml_file,
            src_cargo_toml_file,
        ))
    else:
        print("--- Updating {} from {}".format(
            dst_cargo_toml_file,
            src_cargo_toml_file,
        ))

    src_processed_secs = set()
    src_current_sec = None
    dst_print = True

    with open(src_cargo_toml_file, encoding="utf-8") as src:
        with open(dst_cargo_toml_file, mode="r+", encoding="utf-8") as dst:
            with tempfile.NamedTemporaryFile(mode="w+") as tmp:

                while True:
                    dst_line = dst.readline()
                    if dst_line == Comment.Begin:
                        dst_print = False
                        break
                    elif dst_print:
                        print(dst_line, end="", file=tmp)
                    elif dst_line == "":
                        raise EOFError(
                            "failed to find being commment in {}".format(
                                dst_cargo_toml_file))

                while True:
                    src_line = src.readline()
                    match src_line:
                        case TomlSection.WorkspaceDependencies:
                            if not src_current_sec and not src_processed_secs:
                                print(Comment.Begin.value, file=tmp)
                            if src_current_sec:
                                src_processed_secs.add(src_current_sec)
                            src_current_sec = TomlSection.WorkspaceDependencies
                            print(
                                TomlSection.Dependencies,
                                end="",
                                file=tmp,
                            )
                        case TomlSection.WorkspacePackage:
                            if not src_current_sec and not src_processed_secs:
                                print(Comment.Begin.value, file=tmp)
                            if src_current_sec:
                                src_processed_secs.add(src_current_sec)
                            src_current_sec = TomlSection.WorkspacePackage
                            print(
                                TomlSection.WorkspacePackage,
                                end="",
                                file=tmp,
                            )
                        case TomlSection.PatchCratesIO:
                            if not src_current_sec and not src_processed_secs:
                                print(Comment.Begin.value, file=tmp)
                            if src_current_sec:
                                src_processed_secs.add(src_current_sec)
                            src_current_sec = TomlSection.PatchCratesIO
                            print(
                                TomlSection.PatchCratesIO,
                                end="",
                                file=tmp,
                            )
                        case "":  # EOF
                            if src_current_sec:
                                src_processed_secs.add(src_current_sec)
                            break
                        case _:
                            if src_current_sec:
                                print(src_line, end="", file=tmp)

                if TomlSection.WorkspaceDependencies not in src_processed_secs:
                    raise EOFError(
                        "failed to find TOML '{}' section in '{}'".format(
                            TomlSection.WorkspaceDependencies.value.strip(),
                            src_cargo_toml_file,
                        ))

                print(f"\n{Comment.End}", end="", file=tmp)

                while True:
                    dst_line = dst.readline()
                    if dst_line == Comment.End and not dst_print:
                        dst_print = True
                    elif dst_line and dst_print:
                        print(dst_line, end="", file=tmp)
                    elif not dst_line:
                        break

                tmp.flush()

                file_modified = False

                cmd = [
                    "diff",
                    "-u",
                    dst_cargo_toml_file,
                    tmp.name,
                ]
                result = subprocess.run(cmd, capture_output=True)
                if result.returncode == 1:
                    print("  - Regenerated {} has changed with diff:\n".format(
                        dst_cargo_toml_file))
                    print(result.stdout.decode("ascii"))
                    file_modified = True

                    if not check_mode:
                        # Copy updated cargo file into dst
                        dst.seek(0)
                        dst.truncate(0)
                        tmp.seek(0)
                        shutil.copyfileobj(tmp, dst)
                        dst.flush()
                elif result.returncode != 0:
                    print("xxx diff command failed", file=sys.stderr)
                    print("xxx --- stdout:", file=sys.stderr)
                    print(result.stdout.decode("ascii"), file=sys.stderr)
                    print("xxx --- stderr:", file=sys.stderr)
                    print(result.stderr.decode("ascii"), file=sys.stderr)
                else:
                    print("  - No changes detected for {}".format(
                        dst_cargo_toml_file))

    return file_modified


def update_third_party_rust_cargo_lock(
    cargo_toml_file: Path,
    src_cargo_lock_file: Path,
    dst_cargo_lock_file: Path,
    check_mode: bool,
) -> bool:
    if check_mode:
        print("--- Checking {} in sync with {}".format(
            dst_cargo_lock_file,
            src_cargo_lock_file,
        ))
    else:
        print("--- Updating {} from {}".format(
            dst_cargo_lock_file,
            src_cargo_lock_file,
        ))

    with tempfile.TemporaryDirectory() as tempdir:
        tmp_cargo_toml_file = os.path.join(
            tempdir,
            os.path.basename(cargo_toml_file),
        )
        tmp_cargo_lock_file = os.path.join(
            tempdir,
            os.path.basename(src_cargo_lock_file),
        )

        shutil.copyfile(cargo_toml_file, tmp_cargo_toml_file)
        shutil.copyfile(src_cargo_lock_file, tmp_cargo_lock_file)

        cmd = [
            "cargo",
            "metadata",
            "--format-version",
            "1",
            "--manifest-path",
            tmp_cargo_toml_file,
        ]
        result = subprocess.run(cmd, capture_output=True)
        if result.returncode != 0:
            print("xxx cargo metadata command failed", file=sys.stderr)
            print("xxx --- stdout:", file=sys.stderr)
            print(result.stdout.decode("ascii"), file=sys.stderr)
            print("xxx --- stderr:", file=sys.stderr)
            print(result.stderr.decode("ascii"), file=sys.stderr)
        result.check_returncode()

        file_modified = False

        cmd = [
            "diff",
            "-u",
            dst_cargo_lock_file,
            tmp_cargo_lock_file,
        ]
        result = subprocess.run(cmd, capture_output=True)
        if result.returncode == 1:
            print("  - Regenerated {} has changed with diff:\n".format(
                dst_cargo_lock_file))
            print(result.stdout.decode("ascii"))
            file_modified = True

            if not check_mode:
                shutil.copyfile(tmp_cargo_lock_file, dst_cargo_lock_file)
        elif result.returncode != 0:
            print("xxx diff command failed", file=sys.stderr)
            print("xxx --- stdout:", file=sys.stderr)
            print(result.stdout.decode("ascii"), file=sys.stderr)
            print("xxx --- stderr:", file=sys.stderr)
            print(result.stderr.decode("ascii"), file=sys.stderr)
        else:
            print("  - No changes detected for {}".format(dst_cargo_lock_file))

    return file_modified


def reindeer_buckify(
    third_party_rust_buck_file: Path,
    check_mode: bool,
) -> bool:
    if check_mode:
        print("--- Checking {} with 'reindeer buckify'".format(
            third_party_rust_buck_file))
    else:
        print("--- Updating {} with 'reindeer buckify'".format(
            third_party_rust_buck_file))

    with tempfile.NamedTemporaryFile(mode="w+") as backup_buck:
        # Make backup of third party Rust BUCK file
        with open(
                third_party_rust_buck_file,
                encoding="utf-8",
        ) as src:
            shutil.copyfileobj(src, backup_buck)
            backup_buck.flush()
            backup_buck.seek(0)

        cmd = [
            "reindeer",
            "--third-party-dir",
            os.path.dirname(third_party_rust_buck_file),
            "buckify",
        ]
        result = subprocess.run(cmd)
        if result.returncode != 0:
            # Restore third party Rust BUCK file from backup
            with open(
                    third_party_rust_buck_file,
                    mode="w+",
                    encoding="utf-8",
            ) as dst:
                backup_buck.seek(0)
                shutil.copyfileobj(backup_buck, dst)
                dst.flush()
        result.check_returncode()

        file_modified = False

        cmd = [
            "diff",
            "-u",
            backup_buck.name,
            third_party_rust_buck_file,
        ]
        result = subprocess.run(cmd, capture_output=True)
        if result.returncode == 1:
            print("  - Regenerated {} has changed with diff:\n".format(
                third_party_rust_buck_file))
            print(result.stdout.decode("ascii"))
            file_modified = True

            if check_mode:
                # Restore third party Rust BUCK file from backup
                with open(
                        third_party_rust_buck_file,
                        mode="w+",
                        encoding="utf-8",
                ) as dst:
                    backup_buck.seek(0)
                    shutil.copyfileobj(backup_buck, dst)
                    dst.flush()
        elif result.returncode != 0:
            print("xxx diff command failed", file=sys.stderr)
            print("xxx --- stdout:", file=sys.stderr)
            print(result.stdout.decode("ascii"), file=sys.stderr)
            print("xxx --- stderr:", file=sys.stderr)
            print(result.stderr.decode("ascii"), file=sys.stderr)
        else:
            print("  - No changes detected for {}".format(
                third_party_rust_buck_file))

    return file_modified


if __name__ == "__main__":
    sys.exit(main())
