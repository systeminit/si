#!/usr/bin/env python3
import os
import shutil
import subprocess
import stat
import sys

NEEDED_CMDS = ["gh"]
PROGRAM = "si-ci"
REPO = "systeminit/si-ci"


def main() -> int:
    for needed_cmd in NEEDED_CMDS:
        if not shutil.which(needed_cmd):
            print(
                f"xxx Required command not found: '{needed_cmd}', aborting.",
                file=sys.stderr,
            )
            return 1

    bin = os.path.join(
        os.path.dirname(os.path.dirname(os.path.realpath(__file__))),
        "tmp",
        "bin",
        PROGRAM,
    )

    download_program(bin)

    # Swap in `bin` as argv[0]
    sys.argv.pop(0)
    sys.argv.insert(0, bin)

    # Call `execvp` as program, passing in arguments
    os.execvp(sys.argv[0], sys.argv)


def download_program(dst: str):
    bin_release_name = "{}-{}-{}".format(
        PROGRAM,
        os.uname().sysname.lower(),
        detect_architecture(),
    )
    tmp_dst = os.path.join(os.path.dirname(dst),
                           ".download.{}".format(os.path.basename(dst)))

    if os.path.isfile(tmp_dst):
        os.remove(tmp_dst)

    os.makedirs(os.path.dirname(dst), exist_ok=True)

    download_program_cmd = [
        "gh",
        "release",
        "download",
        "--repo",
        REPO,
        "--pattern",
        bin_release_name,
        "--output",
        tmp_dst,
    ]
    exit_code = subprocess.call(download_program_cmd, stdout=sys.stderr)
    if exit_code != 0:
        print(f"xxx Failed to download {bin_release_name}, aborting.",
              file=sys.stderr)
        sys.exit(exit_code)

    os.chmod(
        tmp_dst, stat.S_IWUSR
        | stat.S_IRUSR
        | stat.S_IXUSR
        | stat.S_IRGRP
        | stat.S_IXGRP
        | stat.S_IROTH
        | stat.S_IXOTH)

    os.replace(tmp_dst, dst)


# Possible machine architecture detection comes from reading the Rustup shell
# script installer--thank you for your service!
# See: https://github.com/rust-lang/rustup/blob/master/rustup-init.sh
def detect_architecture() -> str:
    machine = os.uname().machine

    if (machine == "amd64" or machine == "x86_64" or machine == "x86-64"
            or machine == "x64"):
        return "x86_64"
    elif (machine == "arm64" or machine == "aarch64" or machine == "arm64v8"):
        return "aarch64"
    else:
        print(
            f"xxx Failed to determine architecure or unsupported: {machine}",
            file=sys.stderr,
        )
        sys.exit(1)


if __name__ == "__main__":
    sys.exit(main())
