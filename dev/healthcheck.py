#!/usr/bin/env python3
"""
Detects and reports the max number of open files allowed per process.
"""
import argparse
import subprocess
import shutil
import sys

MIN_OPENFILES_VALUE = 1024


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
            "docker",
            "compose",
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
