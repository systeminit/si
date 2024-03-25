#!/usr/bin/env python3
"""
Runs Cypress E2E Suite against a stack
"""
import argparse
import os
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


class PlatformArchitecture(BaseEnum):
    Aarch64 = "aarch64"
    X86_64 = "x86_64"


class PlatformOS(BaseEnum):
    Darwin = "darwin"
    Linux = "linux"

def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    #parser.add_argument(
    #    "--build-context-dir",
    #    required=True,
    #    help="Path to build context directory",
    #)
    parser.add_argument(
        "--output",
        required=True,
        help="Output directory for results",
    )

    return parser.parse_args()


def main() -> int:
    args = parse_args()

    file = open(args.output, 'a')
    #Append content to the file
    file.write('\n Bananas')

    # Close the file
    file.close()

    return 0

    architecture = detect_architecture()
    os = detect_os()

    # optionally add a check that the stack is running
    # optionally add a check that cypress is installed + available in app/web
    # optionally add a check that the secrets are inside the .env or os.env
    # optionally(add arg for specific tests to run) for invocation like `buck2 run app/web:e2e-test -- modelling-functionality/create-component.cy.ts`
    # optionally add the videos or similar as an output target into the directory
    # optionally only build/deploy the stuff that has changed (how? I have no idea - JW)

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

    if (platform_os == "Darwin"):
        return PlatformOS.Darwin
    elif (platform_os == "Linux"):
        return PlatformOS.Linux
    else:
        print(
            f"xxx Failed to determine operating system or unsupported: {platform_os}",
            file=sys.stderr,
        )
        sys.exit(1)

if __name__ == "__main__":
    sys.exit(main())
