#!/usr/bin/env python3

import argparse
import os
import shutil
import subprocess
import sys
from pathlib import Path


def main():
    parser = argparse.ArgumentParser(description="Install mise tools")
    parser.add_argument("--mise-tools-dir", required=True, help="Output directory for mise tools")
    parser.add_argument("--mise-bootstrap", required=True, help="Output path for mise bootstrap script")
    parser.add_argument("--package", action="append", dest="packages", default=[], help="Package to install")

    args = parser.parse_args()

    mise_tools_dir = Path(args.mise_tools_dir)
    mise_bootstrap_path = Path(args.mise_bootstrap)

    # Parse rust version from packages
    rust_version = None
    for package in args.packages:
        if package.startswith('rust@'):
            rust_version = package.split('@', 1)[1]
            break

    # Create the output directories
    mise_tools_dir.mkdir(parents=True, exist_ok=True)
    mise_bootstrap_path.parent.mkdir(parents=True, exist_ok=True)

    # Create the mise bootstrap script based on the working example
    bootstrap_content = '''#!/usr/bin/env bash
set -eu

__mise_bootstrap() {
    local script_dir=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
    local project_dir=$( cd -- "$( dirname -- "$script_dir" )" &> /dev/null && pwd )
    export MISE_BOOTSTRAP_PROJECT_DIR="$project_dir"
    local localized_dir="$project_dir/mise-tools"
    export MISE_BOOTSTRAP_PROJECT_DIR="$project_dir"
    export MISE_DATA_DIR="$localized_dir"
    export MISE_CONFIG_DIR="$localized_dir"
    export MISE_CACHE_DIR="$localized_dir/cache"
    export MISE_STATE_DIR="$localized_dir/state"
    export MISE_INSTALL_PATH="$localized_dir/mise-2025.6.2"
    export MISE_TRUSTED_CONFIG_PATHS="$project_dir${MISE_TRUSTED_CONFIG_PATHS:+:$MISE_TRUSTED_CONFIG_PATHS}"
    export MISE_IGNORED_CONFIG_PATHS="$HOME/.config/mise${MISE_IGNORED_CONFIG_PATHS:+:$MISE_IGNORED_CONFIG_PATHS}"

    # Set isolated rustup and cargo homes
    export MISE_RUSTUP_HOME="$localized_dir/rustup"
    export MISE_CARGO_HOME="$localized_dir/cargo"

    # These will be set by mise when using rust tools
    export RUSTUP_HOME="$MISE_RUSTUP_HOME"
    export CARGO_HOME="$MISE_CARGO_HOME"
    RUST_TOOLCHAIN_PLACEHOLDER
    install() {
        #!/bin/sh
        set -eu

        #region logging setup
        if [ "${MISE_DEBUG-}" = "true" ] || [ "${MISE_DEBUG-}" = "1" ]; then
          debug() {
            echo "$@" >&2
          }
        else
          debug() {
            :
          }
        fi

        if [ "${MISE_QUIET-}" = "1" ] || [ "${MISE_QUIET-}" = "true" ]; then
          info() {
            :
          }
        else
          info() {
            echo "$@" >&2
          }
        fi

        error() {
          echo "$@" >&2
          exit 1
        }
        #endregion

        #region environment setup
        get_os() {
          os="$(uname -s)"
          if [ "$os" = Darwin ]; then
            echo "macos"
          elif [ "$os" = Linux ]; then
            echo "linux"
          else
            error "unsupported OS: $os"
          fi
        }

        get_arch() {
          musl=""
          if type ldd >/dev/null 2>/dev/null; then
            libc=$(ldd /bin/ls | grep 'musl' | head -1 | cut -d ' ' -f1)
            if [ -n "$libc" ]; then
              musl="-musl"
            fi
          fi
          arch="$(uname -m)"
          if [ "$arch" = x86_64 ]; then
            echo "x64$musl"
          elif [ "$arch" = aarch64 ] || [ "$arch" = arm64 ]; then
            echo "arm64$musl"
          elif [ "$arch" = armv7l ]; then
            echo "armv7$musl"
          else
            error "unsupported architecture: $arch"
          fi
        }

        get_ext() {
          if [ -n "${MISE_INSTALL_EXT:-}" ]; then
            echo "$MISE_INSTALL_EXT"
          elif [ -n "${MISE_VERSION:-}" ] && echo "$MISE_VERSION" | grep -q '^v2024'; then
            # 2024 versions don't have zstd tarballs
            echo "tar.gz"
          elif tar_supports_zstd; then
            echo "tar.zst"
          elif command -v zstd >/dev/null 2>&1; then
            echo "tar.zst"
          else
            echo "tar.gz"
          fi
        }

        tar_supports_zstd() {
          # tar is bsdtar or version is >= 1.31
          if tar --version | grep -q 'bsdtar' && command -v zstd >/dev/null 2>&1; then
            true
          elif tar --version | grep -q '1\\.(3[1-9]|[4-9][0-9]'; then
            true
          else
            false
          fi
        }

        shasum_bin() {
          if command -v shasum >/dev/null 2>&1; then
            echo "shasum"
          elif command -v sha256sum >/dev/null 2>&1; then
            echo "sha256sum"
          else
            error "mise install requires shasum or sha256sum but neither is installed. Aborting."
          fi
        }

        get_checksum() {
          version=$1
          os="$(get_os)"
          arch="$(get_arch)"
          ext="$(get_ext)"
          url="https://github.com/jdx/mise/releases/download/v${version}/SHASUMS256.txt"

          # For current version use static checksum otherwise
          # use checksum from releases
          if [ "$version" = "v2025.6.2" ]; then
            checksum_linux_x86_64="bc293c2bda9b690a5209713a90e5abb34c0b92b3fb11e5345cdf8b8079065956  ./mise-v2025.6.2-linux-x64.tar.gz"
            checksum_linux_x86_64_musl="d26844a7d27da66c8057ac307b912e1cff740492ef9461b594286d32bdf6a4fe  ./mise-v2025.6.2-linux-x64-musl.tar.gz"
            checksum_linux_arm64="e373ff57004d832c065db24645d72302f8b8a0d27ea66372e0050fb26824e2df  ./mise-v2025.6.2-linux-arm64.tar.gz"
            checksum_linux_arm64_musl="87e36cacfe8d73d6aa3399b1b05c82006ba4225cec1e60adfe09ca4376abfd37  ./mise-v2025.6.2-linux-arm64-musl.tar.gz"
            checksum_linux_armv7="7c8b2505d67ab0a0b089fa8755a33b997d789e63f94557b6e3678a1a865c34be  ./mise-v2025.6.2-linux-armv7.tar.gz"
            checksum_linux_armv7_musl="840499ff85c5d5cdb6719deffc7ded081ef194afaf1917b0c049c807da485d69  ./mise-v2025.6.2-linux-armv7-musl.tar.gz"
            checksum_macos_x86_64="0c47313411044a79449ce275a8f9b3e356e8929a59426d0603c270e86fe6245a  ./mise-v2025.6.2-macos-x64.tar.gz"
            checksum_macos_arm64="e3ded5b0e68fb4ddba7d6bdff623d2b6ac27e94c1ef6db77a9a4dfb3e6789631  ./mise-v2025.6.2-macos-arm64.tar.gz"
            checksum_linux_x86_64_zstd="c056c9ea9c13a6f68e9a299da572ac3086b9f6efa26a057d4a8189f9cefedaaf  ./mise-v2025.6.2-linux-x64.tar.zst"
            checksum_linux_x86_64_musl_zstd="6fd494cbf55d1a10c54cadb4d01d8cb9197d954ce7119e1a3adf77cb01ebd8e4  ./mise-v2025.6.2-linux-x64-musl.tar.zst"
            checksum_linux_arm64_zstd="e10703c62d44435542158f90e922897140a92d29bef66794b72c50e1707ecbc6  ./mise-v2025.6.2-linux-arm64.tar.zst"
            checksum_linux_arm64_musl_zstd="ffde350f372877de781838c1c26a4e65ce1102adf6f4334c3ae012a62ad58e15  ./mise-v2025.6.2-linux-arm64-musl.tar.zst"
            checksum_linux_armv7_zstd="2052e619424c5a3ccc836c62bfece1364262b88e3922c7a976060d9b8fef924a  ./mise-v2025.6.2-linux-armv7.tar.zst"
            checksum_linux_armv7_musl_zstd="ba6ce52d6909d9a39cae33554d26ed662e0d2f97b01f04f4a1f335798b3080b8  ./mise-v2025.6.2-linux-armv7-musl.tar.zst"
            checksum_macos_x86_64_zstd="7a8e12c2d8985e3c9b9d7aecc6925d8c8d122bbf0b559ef467b2d504d192eacb  ./mise-v2025.6.2-macos-x64.tar.zst"
            checksum_macos_arm64_zstd="24415b1cbabde0149459ff6fe8b55aca9732a2d95df654b49d669f49b405aa36  ./mise-v2025.6.2-macos-arm64.tar.zst"

            # TODO: refactor this, it's a bit messy
            if [ "$(get_ext)" = "tar.zst" ]; then
              if [ "$os" = "linux" ]; then
                if [ "$arch" = "x64" ]; then
                  echo "$checksum_linux_x86_64_zstd"
                elif [ "$arch" = "x64-musl" ]; then
                  echo "$checksum_linux_x86_64_musl_zstd"
                elif [ "$arch" = "arm64" ]; then
                  echo "$checksum_linux_arm64_zstd"
                elif [ "$arch" = "arm64-musl" ]; then
                  echo "$checksum_linux_arm64_musl_zstd"
                elif [ "$arch" = "armv7" ]; then
                  echo "$checksum_linux_armv7_zstd"
                elif [ "$arch" = "armv7-musl" ]; then
                  echo "$checksum_linux_armv7_musl_zstd"
                else
                  warn "no checksum for $os-$arch"
                fi
              elif [ "$os" = "macos" ]; then
                if [ "$arch" = "x64" ]; then
                  echo "$checksum_macos_x86_64_zstd"
                elif [ "$arch" = "arm64" ]; then
                  echo "$checksum_macos_arm64_zstd"
                else
                  warn "no checksum for $os-$arch"
                fi
              else
                warn "no checksum for $os-$arch"
              fi
            else
              if [ "$os" = "linux" ]; then
                if [ "$arch" = "x64" ]; then
                  echo "$checksum_linux_x86_64"
                elif [ "$arch" = "x64-musl" ]; then
                  echo "$checksum_linux_x86_64_musl"
                elif [ "$arch" = "arm64" ]; then
                  echo "$checksum_linux_arm64"
                elif [ "$arch" = "arm64-musl" ]; then
                  echo "$checksum_linux_arm64_musl"
                elif [ "$arch" = "armv7" ]; then
                  echo "$checksum_linux_armv7"
                elif [ "$arch" = "armv7-musl" ]; then
                  echo "$checksum_linux_armv7_musl"
                else
                  warn "no checksum for $os-$arch"
                fi
              elif [ "$os" = "macos" ]; then
                if [ "$arch" = "x64" ]; then
                  echo "$checksum_macos_x86_64"
                elif [ "$arch" = "arm64" ]; then
                  echo "$checksum_macos_arm64"
                else
                  warn "no checksum for $os-$arch"
                fi
              else
                warn "no checksum for $os-$arch"
              fi
            fi
          else
            if command -v curl >/dev/null 2>&1; then
              debug ">" curl -fsSL "$url"
              checksums="$(curl --compressed -fsSL "$url")"
            else
              if command -v wget >/dev/null 2>&1; then
                debug ">" wget -qO - "$url"
                stderr=$(mktemp)
                checksums="$(wget -qO - "$url")"
              else
                error "mise standalone install specific version requires curl or wget but neither is installed. Aborting."
              fi
            fi
            # TODO: verify with minisign or gpg if available

            checksum="$(echo "$checksums" | grep "$os-$arch.$ext")"
            if ! echo "$checksum" | grep -Eq "^([0-9a-f]{32}|[0-9a-f]{64})"; then
              warn "no checksum for mise $version and $os-$arch"
            else
              echo "$checksum"
            fi
          fi
        }

        #endregion

        download_file() {
          url="$1"
          filename="$(basename "$url")"
          cache_dir="$(mktemp -d)"
          file="$cache_dir/$filename"

          info "mise: installing mise..."

          if command -v curl >/dev/null 2>&1; then
            debug ">" curl -#fLo "$file" "$url"
            curl -#fLo "$file" "$url"
          else
            if command -v wget >/dev/null 2>&1; then
              debug ">" wget -qO "$file" "$url"
              stderr=$(mktemp)
              wget -O "$file" "$url" >"$stderr" 2>&1 || error "wget failed: $(cat "$stderr")"
            else
              error "mise standalone install requires curl or wget but neither is installed. Aborting."
            fi
          fi

          echo "$file"
        }

        install_mise() {
          version="${MISE_VERSION:-v2025.6.2}"
          version="${version#v}"
          os="$(get_os)"
          arch="$(get_arch)"
          ext="$(get_ext)"
          install_path="${MISE_INSTALL_PATH:-$HOME/.local/bin/mise}"
          install_dir="$(dirname "$install_path")"
          tarball_url="https://github.com/jdx/mise/releases/download/v${version}/mise-v${version}-${os}-${arch}.${ext}"

          cache_file=$(download_file "$tarball_url")
          debug "mise-setup: tarball=$cache_file"

          debug "validating checksum"
          cd "$(dirname "$cache_file")" && get_checksum "$version" | "$(shasum_bin)" -c >/dev/null

          # extract tarball
          mkdir -p "$install_dir"
          rm -rf "$install_path"
          cd "$(mktemp -d)"
          if [ "$(get_ext)" = "tar.zst" ] && ! tar_supports_zstd; then
            zstd -d -c "$cache_file" | tar -xf -
          else
            tar -xf "$cache_file"
          fi
          mv mise/bin/mise "$install_path"
          info "mise: installed successfully to $install_path"
        }

        after_finish_help() {
          case "${SHELL:-}" in
          */zsh)
            info "mise: run the following to activate mise in your shell:"
            info "echo \\"eval \\\\\\"\\\\\\$($install_path activate zsh)\\\\\\"\\necho"
            info ""
            info "mise: run \\`mise doctor\\` to verify this is setup correctly"
            ;;
          */bash)
            info "mise: run the following to activate mise in your shell:"
            info "echo \\"eval \\\\\\"\\\\\\$($install_path activate bash)\\\\\\"\\necho"
            info ""
            info "mise: run \\`mise doctor\\` to verify this is setup correctly"
            ;;
          */fish)
            info "mise: run the following to activate mise in your shell:"
            info "echo \\"$install_path activate fish | source\\necho"
            info ""
            info "mise: run \\`mise doctor\\` to verify this is setup correctly"
            ;;
          *)
            info "mise: run \\`$install_path --help\\` to get started"
            ;;
          esac
        }

        install_mise
        if [ "${MISE_INSTALL_HELP-}" != 0 ]; then
          after_finish_help
        fi

        cd "$MISE_BOOTSTRAP_PROJECT_DIR"
    }
    local MISE_INSTALL_HELP=0
    test -f "$MISE_INSTALL_PATH" || install
}
__mise_bootstrap
exec "$MISE_INSTALL_PATH" "$@"
'''

    # Add RUST_TOOLCHAIN environment variable if rust version is specified
    if rust_version:
        rust_toolchain_line = f'\n    # Set rust toolchain version\n    export RUST_TOOLCHAIN="{rust_version}"'
    else:
        rust_toolchain_line = ""

    # Replace placeholder with actual rust toolchain export
    bootstrap_content = bootstrap_content.replace("RUST_TOOLCHAIN_PLACEHOLDER", rust_toolchain_line)

    # Write the bootstrap script
    with open(mise_bootstrap_path, 'w') as f:
        f.write(bootstrap_content)

    # Make it executable
    os.chmod(mise_bootstrap_path, 0o755)

    # Set up environment for mise
    env = os.environ.copy()
    project_dir = mise_bootstrap_path.parent.parent

    # Create isolated rustup and cargo directories
    rustup_home = mise_tools_dir / 'rustup'
    cargo_home = mise_tools_dir / 'cargo'
    rustup_home.mkdir(parents=True, exist_ok=True)
    cargo_home.mkdir(parents=True, exist_ok=True)

    env.update({
        'MISE_BOOTSTRAP_PROJECT_DIR': str(project_dir),
        'MISE_DATA_DIR': str(mise_tools_dir),
        'MISE_CONFIG_DIR': str(mise_tools_dir),
        'MISE_CACHE_DIR': str(mise_tools_dir / 'cache'),
        'MISE_STATE_DIR': str(mise_tools_dir / 'state'),
        'MISE_INSTALL_PATH': str(mise_tools_dir / 'mise-2025.6.2'),
        'MISE_TRUSTED_CONFIG_PATHS': str(project_dir),
        'MISE_INSTALL_HELP': '0',
        # Set isolated rustup and cargo homes
        'MISE_RUSTUP_HOME': str(rustup_home),
        'MISE_CARGO_HOME': str(cargo_home),
        'RUSTUP_HOME': str(rustup_home),
        'CARGO_HOME': str(cargo_home),
    })

    # Add RUST_TOOLCHAIN if rust version is specified
    if rust_version:
        env['RUST_TOOLCHAIN'] = rust_version

    # Ensure mise is installed first
    print("Installing mise...", file=sys.stderr)
    subprocess.run([str(mise_bootstrap_path), '--version'], env=env, check=True)

    # Ensure we are always installing the latest version of the plugins
    print("Updating plugins...", file=sys.stderr)
    subprocess.run([str(mise_bootstrap_path), 'clear', 'cache'], env=env, check=True)
    subprocess.run([str(mise_bootstrap_path), 'clear', 'cache'], env=env, check=True)
    subprocess.run([str(mise_bootstrap_path), 'clear', 'cache'], env=env, check=True)
    subprocess.run([str(mise_bootstrap_path), 'clear', 'cache'], env=env, check=True)
    subprocess.run([str(mise_bootstrap_path), 'plugins', 'update'], env=env, check=True)

    # Install each requested package
    for package in args.packages:
        print(f"Installing {package}...", file=sys.stderr)
        subprocess.run([str(mise_bootstrap_path), 'use', "-g", package], env=env, check=True)

    print("Ensuring shims can shim...", file=sys.stderr)
    subprocess.run([str(mise_bootstrap_path), 'reshim'], env=env, check=True)

    print("Mise installation completed successfully", file=sys.stderr)



if __name__ == "__main__":
    main()
