#!/usr/bin/env python3
"""
Runs Cypress E2E Suite against a stack
"""
import argparse
import os
import sys
from enum import Enum, EnumMeta
from typing import Any
import subprocess
import urllib.request
import time


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


class PlatformArchitecture(BaseEnum):
    Aarch64 = "aarch64"
    X86_64 = "x86_64"


class PlatformOS(BaseEnum):
    Darwin = "darwin"
    Linux = "linux"
    Windows = "windows"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--output",
        required=True,
        help="Output for results/logs/recordings",
    )
    parser.add_argument(
        "--tests",
        help="Path to tests, default is 'cypress/e2e/**'",
        action="store",
    )
    parser.add_argument(
        "--web-endpoint",
        help=
        "Endpoint (<scheme><hostname><path>:<port> to test against, defaults to http://localhost:8080",
        default="http://localhost:8080",
        action="store",
    )

    return parser.parse_args()


def clear_directory(directory):
    try:
        for root, dirs, files in os.walk(directory, topdown=False):
            for name in files:
                file_path = os.path.join(root, name)
                os.remove(file_path)
            for name in dirs:
                dir_path = os.path.join(root, name)
                os.rmdir(dir_path)
        print(
            f"All files and subdirectories in '{directory}' cleared successfully."
        )
    except Exception as e:
        print(f"Error occurred while clearing directory '{directory}': {e}")


def print_last_50_lines(file_path):
    # Open the file for reading
    with open(file_path, "r") as file:
        # Read the last 50 lines using file.readlines() and slicing
        lines = file.readlines()[-50:]

        # Print the last 50 lines
        for line in lines:
            print(line, end="")  # Print each line without adding extra newline


def run_cypress_tests(directory_path, tests, output_file):

    # Get the absolute path of the output file
    output_file_path = os.path.abspath(os.path.join(os.getcwd(), output_file))

    clear_directory(output_file_path.rsplit('/', 1)[0])

    # Ensure the directory of the output file exists, create if necessary
    os.makedirs(os.path.dirname(output_file_path), exist_ok=True)

    command = f"ls {output_file_path.rsplit('/', 1)[0]}"
    process = subprocess.run(command, shell=True)

    # Run the Cypress tests using subprocess and redirect output to the specified file
    with open(output_file_path, "a") as output:
        command = f"cd app/web && npx cypress run --spec {tests} --config videosFolder={output_file_path.rsplit('/', 1)[0]}\/videos"
        process = subprocess.run(command,
                                 shell=True,
                                 stdout=output,
                                 stderr=subprocess.PIPE)

    print_last_50_lines(output_file)

    # Check the exit code
    if process.returncode != 0:
        exit(1)


def validate_cypress_install(output_file):

    # Get the absolute path of the output file
    output_file_path = os.path.abspath(os.path.join(os.getcwd(), output_file))

    # Ensure the directory of the output file exists, create if necessary
    os.makedirs(os.path.dirname(output_file_path), exist_ok=True)

    # Clear the output file if it already exists
    if os.path.exists(output_file_path):
        open(output_file_path, 'w').close()

    # Run the Cypress tests using subprocess and redirect output to the specified file
    with open(output_file_path, "a") as output:
        command = f"cd app/web && npx cypress verify"
        process = subprocess.run(command,
                                 shell=True,
                                 stdout=output,
                                 stderr=subprocess.PIPE)

    # Check the exit code
    if process.returncode != 0:
        print_last_50_lines(output_file)
        exit(1)


def health_check(endpoint, timeout):
    start_time = time.time()
    while True:
        try:
            response = urllib.request.urlopen(endpoint)
            if response.getcode() == 200:
                print("Endpoint is healthy:", endpoint)
                return 0  # Exit code 0 indicates success
        except urllib.error.URLError as e:
            print(f"Error occurred: {e}")

        if time.time() - start_time >= timeout:
            print("Timeout reached. Endpoint is not healthy:", endpoint)
            return 1  # Exit code 1 indicates failure

        print("Endpoint not yet healthy. Retrying in 5 seconds...")
        time.sleep(5)


def main() -> int:
    args = parse_args()

    detect_architecture()
    detect_os()
    validate_cypress_install(args.output)

    directory_path = "app/web"
    tests = args.tests

    health_check(args.web_endpoint, 60)
    run_cypress_tests(directory_path, tests, args.output)

    return 0


# Possible machine architecture detection comes from reading the Rustup shell
# script installer--thank you for your service!
# See: https://github.com/rust-lang/rustup/blob/master/rustup-init.sh
def detect_architecture() -> PlatformArchitecture:
    machine = os.uname().machine

    if (machine == "amd64" or machine == "x86_64" or machine == "x86-64"
            or machine == "x64"):
        return PlatformArchitecture.X86_64
    elif (machine == "arm64" or machine == "aarch64" or machine == "arm64v8"):
        return PlatformArchitecture.Aarch64
    else:
        print(
            f"xxx Failed to determine architecure or unsupported: {machine}",
            file=sys.stderr,
        )
        sys.exit(1)


# Possible machine operating system detection comes from reading the Rustup shell
# script installer--thank you for your service!
# See: https://github.com/rust-lang/rustup/blob/master/rustup-init.sh
def detect_os() -> PlatformOS:
    platform_os = os.uname().sysname

    match platform_os:
        case "Darwin":
            return PlatformOS.Darwin
        case "Linux":
            return PlatformOS.Linux
        case "Windows":
            return PlatformOS.Windows
        case _:
            print(
                f"xxx Failed to determine operating system or unsupported: {platform_os}",
                file=sys.stderr,
            )
            sys.exit(1)


if __name__ == "__main__":
    sys.exit(main())
