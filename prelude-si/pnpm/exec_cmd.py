#!/usr/bin/env python3
"""
Runs a program, optionally in a directory.
"""
import argparse
import os
import sys

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--cwd",
        help="Directory under which to run the program",
    )
    parser.add_argument(
        "args",
        help="Program and arguments",
        nargs=argparse.REMAINDER,
    )

    args = parser.parse_args()
    args.args.pop(0)

    cmd = []
    for arg in args.args:
        if arg.endswith("::abspath"):
            cmd.append(os.path.abspath(arg.removesuffix("::abspath")))
        elif arg.endswith("::relpath"):
            if not args.cwd:
                print("Cannot compute relative path, --cwd not set")
                sys.exit(1)
            cmd.append(
                os.path.relpath(os.path.abspath(arg.removesuffix("::relpath")),
                                args.cwd))
        else:
            cmd.append(arg)

    print("--- Executing: cmd='{}', cwd={}".format(" ".join(cmd), args.cwd))
    os.chdir(args.cwd)
    os.execvp(cmd[0], cmd)
