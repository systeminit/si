# Deno Binary Artifacts

## Overview

Deno binary artifacts package compiled Deno binaries into distributable archives
(tar.gz or zip) with metadata sidecars for version tracking and promotion.

## Usage

### Adding to Your Package

In your `BUCK` file:

```python
load(
    "@prelude-si//:macros.bzl",
    "deno_binary",
    "deno_binary_artifact",
)

deno_binary(
    name = "my-cli",
    main = "main.ts",
    srcs = glob(["src/**/*.ts"]),
    permissions = ["allow-read", "allow-write"],
)

deno_binary_artifact(
    name = "my-cli",
    binary = ":my-cli",
)
```

### Building Artifacts

```bash
# Build for current platform
buck2 build //path/to:my-cli-binary-artifact

# Cross-compile for specific platform
buck2 build //path/to:my-cli-binary-artifact \
    --target-platforms=prelude-si//platforms:linux-x86_64

# Build using platform-specific alias
buck2 build //path/to:my-cli-binary-artifact-darwin-aarch64

# Build all platforms
buck2 build \
  //path/to:my-cli-binary-artifact-linux-x86_64 \
  //path/to:my-cli-binary-artifact-linux-aarch64 \
  //path/to:my-cli-binary-artifact-darwin-x86_64 \
  //path/to:my-cli-binary-artifact-darwin-aarch64 \
  //path/to:my-cli-binary-artifact-windows-x86_64
```

### Publishing Artifacts

```bash
# Publish single platform (requires AWS credentials)
buck2 run //path/to:publish-my-cli-binary-artifact-linux-x86_64

# Publish with explicit platform
buck2 run //path/to:publish-my-cli-binary-artifact \
    --target-platforms=prelude-si//platforms:linux-x86_64
```

### Promoting Artifacts

```bash
# Promote to stable channel (requires AWS credentials)
buck2 run //path/to:promote-my-cli-binary-artifact-linux-x86_64
```

## Artifact Structure

### Unix Platforms (Linux, macOS)

Archives use `.tar.gz` format with flat structure:

```
my-cli-2024.11.28.001-binary-linux-x86_64.tar.gz
├── my-cli              # Binary
└── metadata.json       # Metadata sidecar
```

### Windows Platforms

Archives use `.zip` format with flat structure:

```
my-cli-2024.11.28.001-binary-windows-x86_64.zip
├── my-cli.exe          # Binary
└── metadata.json       # Metadata sidecar
```

## Metadata Format

The `metadata.json` file contains:

```json
{
  "family": "my-cli",
  "variant": "binary",
  "version": "2024.11.28.001",
  "arch": "x86_64",
  "os": "linux",
  "commit": "abc123...",
  "branch": "main",
  "b3sum": "def456..."
}
```

## Supported Platforms

- linux-x86_64
- linux-aarch64
- darwin-x86_64 (Intel Macs)
- darwin-aarch64 (Apple Silicon)
- windows-x86_64

**Note:** Windows ARM64 (windows-aarch64) is not supported because Deno itself
does not provide a compilation target for `aarch64-pc-windows-msvc`.

## Architecture

Deno binary artifacts use generic artifact infrastructure shared with Rust
binaries:

- `prelude-si/artifact/generate_binary_metadata.py` - Metadata generation
- `prelude-si/artifact/create_binary_archive.py` - Archive creation
- Platform information from Deno toolchain (`target_os`, `target_arch`)

## Differences from Rust Binary Artifacts

1. **Archive Structure**: Flat (binary and metadata at root) vs Rust's
   usr/local/bin layout
2. **Platform Detection**: Toolchain-provided vs host detection
3. **Cross-Compilation**: Supported from day one vs host-only

## CI Integration

CI automatically builds all platform artifacts when `bin/si` changes are
detected. The BXL script filters for `artifact_publish` rule kind, and si-ci
schedules platform-specific builds on appropriate agents.
