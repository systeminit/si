#!/usr/bin/env python3
"""
Downloads a Deno target runtime for cross-compilation.

Uses Deno's built-in cross-compilation support to download the target
runtime binary without performing an actual compilation.
"""
import argparse
import pathlib
import subprocess
import sys
import tempfile

# Target mapping: canonical target string -> Deno target triple
TARGET_TO_DENO_TRIPLE = {
    "linux-x86_64": "x86_64-unknown-linux-gnu",
    "linux-aarch64": "aarch64-unknown-linux-gnu",
    "darwin-x86_64": "x86_64-apple-darwin",
    "darwin-aarch64": "aarch64-apple-darwin",
    "windows-x86_64": "x86_64-pc-windows-msvc",
    "windows-aarch64": "aarch64-pc-windows-msvc",
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--deno-exe",
        required=True,
        type=pathlib.Path,
        help="The path to the deno executable",
    )
    parser.add_argument(
        "--target",
        required=True,
        type=str,
        help="Target platform string (e.g., linux-x86_64)",
    )
    parser.add_argument(
        "--output-dir",
        required=True,
        type=pathlib.Path,
        help="Directory to store the downloaded runtime",
    )
    return parser.parse_args()


def target_to_deno_triple(target_str: str) -> str:
    """Convert canonical target string to Deno's target triple format."""
    if target_str not in TARGET_TO_DENO_TRIPLE:
        raise ValueError(
            f"Unsupported target: {target_str}. "
            f"Valid targets: {', '.join(TARGET_TO_DENO_TRIPLE.keys())}")
    return TARGET_TO_DENO_TRIPLE[target_str]


def download_target_runtime(
    deno_exe: pathlib.Path,
    target: str,
    output_dir: pathlib.Path,
) -> None:
    """Download the target runtime by performing a minimal cross-compilation."""
    deno_exe_abs = deno_exe.resolve()
    output_dir_abs = output_dir.resolve()
    output_dir_abs.mkdir(parents=True, exist_ok=True)

    deno_triple = target_to_deno_triple(target)

    with tempfile.TemporaryDirectory() as temp_deno_dir:
        temp_deno_path = pathlib.Path(temp_deno_dir)

        # Create a minimal TypeScript file
        temp_script = temp_deno_path / "minimal.ts"
        temp_script.write_text('console.log("test");')

        # Perform compilation to trigger runtime download
        temp_binary = temp_deno_path / "output"

        cmd = [
            str(deno_exe_abs),
            "compile",
            "--target",
            deno_triple,
            "--output",
            str(temp_binary),
            str(temp_script),
        ]

        env = {"DENO_DIR": str(temp_deno_path), "DENO_NO_PACKAGE_JSON": "1"}

        try:
            subprocess.run(
                cmd,
                check=True,
                capture_output=True,
                text=True,
                env=env,
            )
        except subprocess.CalledProcessError as e:
            print(f"Error downloading target runtime: {e}", file=sys.stderr)
            print(f"Command: {' '.join(cmd)}", file=sys.stderr)
            if e.stdout:
                print(f"stdout: {e.stdout}", file=sys.stderr)
            if e.stderr:
                print(f"stderr: {e.stderr}", file=sys.stderr)
            raise

        # Copy the downloaded runtime from DENO_DIR/dl to output
        dl_dir = temp_deno_path / "dl"
        if dl_dir.exists():
            import shutil
            for item in dl_dir.iterdir():
                dest = output_dir_abs / item.name
                if item.is_file():
                    shutil.copy2(item, dest)
                else:
                    shutil.copytree(item, dest, dirs_exist_ok=True)


def main() -> int:
    try:
        args = parse_args()
        download_target_runtime(args.deno_exe, args.target, args.output_dir)
        return 0
    except Exception as e:
        print(f"Error: {str(e)}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
