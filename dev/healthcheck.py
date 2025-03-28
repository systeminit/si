#!/usr/bin/env python3
"""
Detects and reports the max number of open files allowed per process.
"""
import argparse
import subprocess
import shutil
import sys

# Why so many??
#
# Currently there are a few reasons:
# - Our in-memory/on-disk caching solution (foyer) drives the number of open
#   files handles up, and we run 3 copies in the dev stack
# - Database connection pools are numerous (3x per process and they default to a
#   number based on core count)
#
# If and when the math changes, this value can be *dramatically* decreased, but
# until then, it's a high value ;)
MIN_OPENFILES_VALUE = 16384

MIN_IONOTIFY_WATCHES = 65536 * 2


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.parse_args()

    remediations = 0

    section("Running health check on system for developing in the SI project")

    docker_cmd = detect_docker_command()
    remediations += docker_cmd
    if docker_cmd == 0:
        remediations += detect_docker_engine()
        remediations += detect_docker_compose()
    remediations += detect_nix_bash()
    remediations += detect_openfiles_soft_limit()
    if sys.platform.lower().startswith("linux"):
        remediations += detect_ionotify_user_watches()

    blank()
    if remediations > 0:
        section(f"Health check complete with _{remediations}_ "
                "suggested remediation(s)")
    else:
        section(f"Health check complete with **no** suggested remediations.")

    # If there are remediations, exit non-zero so scripts can detect that the
    # current setup may not be healthy
    return remediations


def detect_docker_command() -> int:
    result = detect_command("docker")
    return result


def detect_docker_engine() -> int:
    result = subprocess.run(["docker", "info"], capture_output=True)
    if result.returncode == 0:
        info("Detected running Docker Engine")
        return 0
    else:
        warn("Failed to detect running Docker Engine")
        indent("Output from `docker info` stderr:")
        indent("-----")
        for line in result.stderr.splitlines():
            indent(line.decode("ascii"))
        indent("-----")
        indent("Ensure that the Docker Engine is running and try again")
        return 1


def detect_docker_compose() -> int:
    result = subprocess.run(
        [
            "docker-compose",
            "version",
        ],
        capture_output=True,
    )
    if result.returncode == 0:
        info("Detected Docker Compose is installed")
        return 0
    else:
        warn("Failed to detect Docker Compose installation")
        indent("Output from `docker compose version` stderr:")
        indent("-----")
        for line in result.stderr.splitlines():
            indent(line.decode("ascii"))
        indent("-----")
        indent("Ensure that the Docker Compose is running and try again")
        return 1


def detect_nix_bash() -> int:
    cmd = "bash"
    result = shutil.which(cmd)
    if result and result.startswith("/nix/store/"):
        info(f"Found `{cmd}` in Nix environment")
        return 0
    elif result:
        warn(f"Failed to find `{cmd}` in Nix environment (cmd={result})")
        indent("Ensure that your direnv setup is correct or that you have ran "
               "`nix develop`")
        return 1
    else:
        warn(f"Failed to find `{cmd}` in Nix environment or on system")
        indent("Ensure that your direnv setup is correct or that you have ran "
               "`nix develop`")
        return 1


def detect_openfiles_soft_limit() -> int:
    soft_limit = current_openfiles_soft_limit()

    if soft_limit < MIN_OPENFILES_VALUE:
        new = MIN_OPENFILES_VALUE
        warn("Low value for max open files soft limit detected in "
             f"current shell (current={soft_limit})")
        blank()
        indent("To set a value for this shell session **only** run:")
        indent(f"ulimit -Sn {new}", amount=2)
        blank()
        indent(
            "To make a **permanent** change for all new shell sessions run:")
        indent(f"echo 'ulimit -Sn {new}' >>\"$HOME/.profile\"", amount=2)
        return 1
    else:
        info("Reasonable value for max open files soft limit detected in "
             f"current shell (current={soft_limit})")
        return 0


def detect_ionotify_user_watches() -> int:
    value = current_ionotify_user_watches_limit()

    if value < MIN_IONOTIFY_WATCHES:
        warn("Low value for `fs.inotify.max_user_watches` kernel setting "
             f"current value is (current={value})")
        blank()
        indent("To increase this value, run: ")
        blank()
        warn("This command requires root permissions (sudo)")
        blank()
        indent(
            f"sudo sysctl fs.inotify.max_user_watches={MIN_IONOTIFY_WATCHES}")
        blank()
        indent("To make this **permanent**, concult your Linux distro's "
               "documentation")
        return 1
    else:
        info("Reasonable value for `fs.inotify.max_user_watches` "
             f"(current={value})")
        return 0


def detect_command(cmd: str) -> int:
    result = shutil.which(cmd)
    if result:
        info(f"Detected `{cmd}` command (cmd={result})")
        return 0
    else:
        warn(f"Failed to find `{cmd}` on PATH and is required for development")
        return 1


def current_openfiles_soft_limit() -> int:
    result = subprocess.run(
        ["bash", "-c", "ulimit -Sn"],
        capture_output=True,
    )
    result.check_returncode()
    return int(result.stdout.strip().decode("ascii"))


def current_ionotify_user_watches_limit() -> int:
    result = subprocess.run(
        ["sysctl", "fs.inotify.max_user_watches"],
        capture_output=True,
    )
    result.check_returncode()
    lines = result.stdout.strip().splitlines()
    _, value = lines[0].split(b"=")
    return int(value)


def section(msg: str):
    print(f"--- {msg}")


def info(msg: str):
    print(f"  - {msg}")


def warn(msg: str):
    print(f"  x {msg}")


def indent(msg: str, amount=1):
    print("{:>{width}}{}".format("", msg, width=amount * 4))


def blank():
    print("")


if __name__ == "__main__":
    sys.exit(main())
