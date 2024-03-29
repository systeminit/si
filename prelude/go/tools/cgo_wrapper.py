#!/usr/bin/env python3
# Copyright (c) Meta Platforms, Inc. and affiliates.
#
# This source code is licensed under both the MIT license found in the
# LICENSE-MIT file in the root directory of this source tree and the Apache
# License, Version 2.0 found in the LICENSE-APACHE file in the root directory
# of this source tree.

# pyre-unsafe

import argparse
import os
import subprocess
import sys
import tempfile
from pathlib import Path


def main(argv):
    parser = argparse.ArgumentParser(fromfile_prefix_chars="@")
    parser.add_argument("--cgo", action="append", default=[])
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--cpp", action="append", default=[])
    parser.add_argument("--env-cc", action="append", default=[])
    parser.add_argument("--env-ldflags", action="append", default=[])
    parser.add_argument("srcs", type=Path, nargs="*")
    args = parser.parse_args(argv[1:])

    output = args.output.resolve(strict=False)
    os.makedirs(output, exist_ok=True)

    env = os.environ.copy()
    env["CC"] = " ".join(args.env_cc)
    env["CGO_LDFLAGS"] = " ".join(args.env_ldflags)

    cmd = []
    cmd.extend(args.cgo)
    # cmd.append("-importpath={}")
    # cmd.append("-srcdir={}")
    cmd.append(f"-objdir={output}")
    # cmd.append(cgoCompilerFlags)
    cmd.append("--")
    # cmd.append(cxxCompilerFlags)

    if args.cpp:
        with tempfile.NamedTemporaryFile("w", delete=False) as argsfile:
            for arg in args.cpp:
                print(arg, file=argsfile)
                argsfile.flush()
        cmd.append("@" + argsfile.name)

    cmd.extend(args.srcs)
    return subprocess.call(cmd, env=env)


sys.exit(main(sys.argv))
