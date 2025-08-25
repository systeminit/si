"""Hermetic Rust toolchain distribution rules.

Downloads and manages Rust toolchains without external dependencies.
Similar to the zig toolchain approach but for Rust.
"""

load("@prelude//rust:rust_toolchain.bzl", "PanicRuntime", "RustToolchainInfo")

# Rust release information with checksums
_RUST_RELEASES = {
    "1.88.0": {
        "x86_64-unknown-linux-gnu": {
            "url": "https://static.rust-lang.org/dist/rust-1.88.0-x86_64-unknown-linux-gnu.tar.xz",
            "sha256": "7b5437c1d18a174faae253a18eac22c32288dccfc09ff78d5ee99b7467e21bca",
        },
        "aarch64-unknown-linux-gnu": {
            "url": "https://static.rust-lang.org/dist/rust-1.88.0-aarch64-unknown-linux-gnu.tar.xz",
            "sha256": "d5decc46123eb888f809f2ee3b118d13586a37ffad38afaefe56aa7139481d34",
        },
        "x86_64-apple-darwin": {
            "url": "https://static.rust-lang.org/dist/rust-1.88.0-x86_64-apple-darwin.tar.xz",
            "sha256": "421d34e45b9a17a51cf32351332f5c2a9dc944aad36d23bedc526912fcbd2fec",
        },
        "aarch64-apple-darwin": {
            "url": "https://static.rust-lang.org/dist/rust-1.88.0-aarch64-apple-darwin.tar.xz",
            "sha256": "9d64ea19e4051422428991b2c66bf108699f1ff11cc090466474902efad4db96",
        },
    },
    "nightly-2025-04-17": {
        "x86_64-unknown-linux-gnu": {
            "url": "https://static.rust-lang.org/dist/2025-04-17/rust-nightly-x86_64-unknown-linux-gnu.tar.xz",
            "sha256": "7e7b4e92954e64fb7f936a7295d7a25baa35b3ce29546d2ef0855a16c6cad68f",
        },
        "aarch64-unknown-linux-gnu": {
            "url": "https://static.rust-lang.org/dist/2025-04-17/rust-nightly-aarch64-unknown-linux-gnu.tar.xz",
            "sha256": "5749da11e45d7c492ac1a13d1463388af4153529c0ba3b81a244dcc7bd5b9d3d",
        },
        "x86_64-apple-darwin": {
            "url": "https://static.rust-lang.org/dist/2025-04-17/rust-nightly-x86_64-apple-darwin.tar.xz",
            "sha256": "7ede406613f27eb6c330d6291510d1e69814f50626d585c8f8d27eaaa2749c1a",
        },
        "aarch64-apple-darwin": {
            "url": "https://static.rust-lang.org/dist/2025-04-17/rust-nightly-aarch64-apple-darwin.tar.xz",
            "sha256": "739f2510266353e5a92ced9dbdd6f7de75132f72c53d833bd72086a4cc978519",
        },
    },
}

RustDistributionInfo = provider(
    fields = {
        "version": provider_field(typing.Any, default = None),
        "target": provider_field(typing.Any, default = None),
        "rustc": provider_field(typing.Any, default = None),
        "cargo": provider_field(typing.Any, default = None),
        "rustdoc": provider_field(typing.Any, default = None),
        "clippy": provider_field(typing.Any, default = None),
        "rustfmt": provider_field(typing.Any, default = None),
        "std_lib": provider_field(typing.Any, default = None),
    },
)

def _get_rust_release(version: str, target: str):
    if version not in _RUST_RELEASES:
        fail("Unknown Rust version '{}'. Available versions: {}".format(
            version,
            ", ".join(_RUST_RELEASES.keys()),
        ))

    rust_version = _RUST_RELEASES[version]
    if target not in rust_version:
        fail("Unsupported target '{}' for Rust {}. Supported targets: {}".format(
            target,
            version,
            ", ".join(rust_version.keys()),
        ))

    return rust_version[target]

def _rust_distribution_impl(ctx: AnalysisContext) -> list[Provider]:
    """Extract Rust toolchain from downloaded archive."""

    # Path to component directories in the extracted archive
    archive_path = cmd_args(ctx.attrs.archive[DefaultInfo].default_outputs[0])

    # Just create a directory output that contains everything - keep it simple like zig
    rust_dist = ctx.actions.declare_output("rust", dir=True)

    # Create setup script to avoid escaping issues
    setup_script = ctx.actions.declare_output("setup.sh")

    # Determine rust directory name pattern based on version
    if ctx.attrs.version.startswith("nightly"):
        rust_dir = "rust-nightly-{}".format(ctx.attrs.target)
    else:
        rust_dir = "rust-{}-{}".format(ctx.attrs.version, ctx.attrs.target)
    
    ctx.actions.write(
        setup_script,
        [
            "#!/bin/bash",
            "set -e",
            "ARCHIVE_DIR=$1",
            "OUTPUT_DIR=$3",
            "",
            "# Create directories",
            "mkdir -p $OUTPUT_DIR/bin $OUTPUT_DIR/lib",
            "",
            "# Copy rustc libraries",
            cmd_args("cp -r $ARCHIVE_DIR/{}/rustc/lib/* $OUTPUT_DIR/lib/".format(rust_dir)),
            "# Copy standard library to proper location in rustlib",
            cmd_args("cp -r $ARCHIVE_DIR/{}/rust-std-{}/lib/rustlib/* $OUTPUT_DIR/lib/rustlib/".format(rust_dir, ctx.attrs.target)),
            cmd_args("cp $ARCHIVE_DIR/{}/rustc/bin/rustc $OUTPUT_DIR/bin/rustc-actual".format(rust_dir)),
            cmd_args("cp $ARCHIVE_DIR/{}/cargo/bin/cargo $OUTPUT_DIR/bin/cargo-actual".format(rust_dir)),
            cmd_args("cp $ARCHIVE_DIR/{}/rustc/bin/rustdoc $OUTPUT_DIR/bin/rustdoc-actual".format(rust_dir)),
            cmd_args("cp $ARCHIVE_DIR/{}/clippy-preview/bin/clippy-driver $OUTPUT_DIR/bin/clippy-driver-actual".format(rust_dir)),
            cmd_args("cp $ARCHIVE_DIR/{}/rustfmt-preview/bin/rustfmt $OUTPUT_DIR/bin/rustfmt".format(rust_dir)),
            "",
            "# Create wrapper scripts",
            "cat > $OUTPUT_DIR/bin/rustc << 'EOF'",
            "#!/bin/bash",
            "set -e",
            'SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"',
            'export LD_LIBRARY_PATH="$SCRIPT_DIR/../lib:$LD_LIBRARY_PATH"',
            'exec "$SCRIPT_DIR/rustc-actual" "$@"',
            "EOF",
            "chmod +x $OUTPUT_DIR/bin/rustc",
            "",
            "cat > $OUTPUT_DIR/bin/cargo << 'EOF'",
            "#!/bin/bash",
            "set -e",
            'SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"',
            'export LD_LIBRARY_PATH="$SCRIPT_DIR/../lib:$LD_LIBRARY_PATH"',
            'exec "$SCRIPT_DIR/cargo-actual" "$@"',
            "EOF",
            "chmod +x $OUTPUT_DIR/bin/cargo",
            "",
            "cat > $OUTPUT_DIR/bin/rustdoc << 'EOF'",
            "#!/bin/bash",
            "set -e",
            'SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"',
            'export LD_LIBRARY_PATH="$SCRIPT_DIR/../lib:$LD_LIBRARY_PATH"',
            'exec "$SCRIPT_DIR/rustdoc-actual" "$@"',
            "EOF",
            "chmod +x $OUTPUT_DIR/bin/rustdoc",
            "",
            "cat > $OUTPUT_DIR/bin/clippy-driver << 'EOF'",
            "#!/bin/bash",
            "set -e",
            'SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"',
            'export LD_LIBRARY_PATH="$SCRIPT_DIR/../lib:$LD_LIBRARY_PATH"',
            'exec "$SCRIPT_DIR/clippy-driver-actual" "$@"',
            "EOF",
            "chmod +x $OUTPUT_DIR/bin/clippy-driver",
        ],
        is_executable = True,
    )

    ctx.actions.run([
        setup_script, archive_path, "unused", rust_dist.as_output()
    ], category = "rust_setup")

    # Individual binary references point to files in the dist directory
    rustc = cmd_args(rust_dist, "/bin/rustc", delimiter="")
    cargo = cmd_args(rust_dist, "/bin/cargo", delimiter="")
    rustdoc = cmd_args(rust_dist, "/bin/rustdoc", delimiter="")
    clippy = cmd_args(rust_dist, "/bin/clippy-driver", delimiter="")
    rustfmt = cmd_args(rust_dist, "/bin/rustfmt", delimiter="")

    return [
        DefaultInfo(default_output = rust_dist),
        RustDistributionInfo(
            version = ctx.attrs.version,
            target = ctx.attrs.target,
            rustc = rustc,
            cargo = cargo,
            rustdoc = rustdoc,
            clippy = clippy,
            rustfmt = rustfmt,
            std_lib = cmd_args(rust_dist, "/lib", delimiter=""),
        ),
    ]

rust_distribution = rule(
    impl = _rust_distribution_impl,
    attrs = {
        "version": attrs.string(),
        "target": attrs.string(),
        "archive": attrs.dep(providers = [DefaultInfo]),
    },
)

def _http_archive_impl(ctx: AnalysisContext) -> list[Provider]:
    """Download and extract a tar.xz archive using bundled static xz binary."""
    url = ctx.attrs.urls[0]

    # Download archive
    archive = ctx.actions.declare_output("archive.tar.xz")
    ctx.actions.download_file(archive.as_output(), url, sha256 = ctx.attrs.sha256)

    # Extract archive using system xz command (should be available in buildpack-deps)
    output = ctx.actions.declare_output(ctx.label.name, dir = True)
    script = [
        "mkdir -p $1",
        "tar xJf $2 -C $1 --strip-components=0"
    ]
    ctx.actions.run([
        "sh", "-c", "; ".join(script), "--", output.as_output(), archive
    ], category = "extract_rust")

    return [DefaultInfo(default_output = output)]

_http_archive = rule(
    impl = _http_archive_impl,
    attrs = {
        "urls": attrs.list(attrs.string()),
        "sha256": attrs.string(default = ""),
    },
)

def _host_target() -> str:
    """Determine the host target triple."""
    arch = host_info().arch
    os = host_info().os

    if arch.is_x86_64:
        arch_str = "x86_64"
    elif arch.is_aarch64:
        arch_str = "aarch64"
    else:
        fail("Unsupported host architecture")

    if os.is_linux:
        return arch_str + "-unknown-linux-gnu"
    elif os.is_macos:
        return arch_str + "-apple-darwin"
    else:
        fail("Unsupported host OS")

def download_rust_distribution(
        name: str,
        version: str,
        target: [None, str] = None):
    """Download a Rust distribution.

    Args:
        name: Name of the target
        version: Rust version (e.g., "1.86.0")
        target: Target triple (defaults to host)
    """
    if target == None:
        target = _host_target()

    release = _get_rust_release(version, target)
    archive_name = name + "-archive"

    _http_archive(
        name = archive_name,
        urls = [release["url"]],
        sha256 = release["sha256"],
    )

    rust_distribution(
        name = name,
        version = version,
        target = target,
        archive = ":" + archive_name,
    )

def _hermetic_rust_toolchain_impl(ctx: AnalysisContext) -> list[Provider]:
    """Create a hermetic Rust toolchain from a distribution."""

    dist = ctx.attrs.distribution[RustDistributionInfo]

    # Create command-line wrappers for the tools
    rustc_cmd = cmd_args([dist.rustc])
    rustdoc_cmd = cmd_args([dist.rustdoc])
    clippy_cmd = cmd_args([dist.clippy])

    return [
        DefaultInfo(),
        RustToolchainInfo(
            allow_lints = ctx.attrs.allow_lints,
            clippy_driver = RunInfo(args = clippy_cmd),
            clippy_toml = ctx.attrs.clippy_toml[DefaultInfo].default_outputs[0] if ctx.attrs.clippy_toml else None,
            compiler = RunInfo(args = rustc_cmd),
            default_edition = ctx.attrs.default_edition,
            panic_runtime = PanicRuntime("unwind"),
            deny_lints = ctx.attrs.deny_lints,
            doctests = ctx.attrs.doctests,
            nightly_features = ctx.attrs.nightly_features,
            report_unused_deps = ctx.attrs.report_unused_deps,
            rustc_binary_flags = ctx.attrs.rustc_binary_flags,
            rustc_flags = ctx.attrs.rustc_flags,
            rustc_target_triple = ctx.attrs.rustc_target_triple if ctx.attrs.rustc_target_triple else dist.target,
            rustc_test_flags = ctx.attrs.rustc_test_flags,
            rustdoc = RunInfo(args = rustdoc_cmd),
            rustdoc_flags = ctx.attrs.rustdoc_flags,
            warn_lints = ctx.attrs.warn_lints,
        ),
    ]

hermetic_rust_toolchain = rule(
    impl = _hermetic_rust_toolchain_impl,
    attrs = {
        "distribution": attrs.exec_dep(providers = [RustDistributionInfo]),
        "allow_lints": attrs.list(attrs.string(), default = []),
        "clippy_toml": attrs.option(attrs.dep(providers = [DefaultInfo]), default = None),
        "default_edition": attrs.option(attrs.string(), default = None),
        "deny_lints": attrs.list(attrs.string(), default = []),
        "doctests": attrs.bool(default = False),
        "nightly_features": attrs.bool(default = False),
        "report_unused_deps": attrs.bool(default = False),
        "rustc_binary_flags": attrs.list(attrs.arg(), default = []),
        "rustc_flags": attrs.list(attrs.arg(), default = []),
        "rustc_target_triple": attrs.string(default = ""),
        "rustc_test_flags": attrs.list(attrs.arg(), default = []),
        "rustdoc_flags": attrs.list(attrs.arg(), default = []),
        "warn_lints": attrs.list(attrs.string(), default = []),
    },
    is_toolchain_rule = True,
)
