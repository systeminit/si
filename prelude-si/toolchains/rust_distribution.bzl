"""Hermetic Rust toolchain distribution rules using S3 artifacts.

Downloads pre-packaged Rust toolchains from si-artifacts-prod.
Much simpler than the original - no complex extraction or wrapper scripts needed.
"""

load("@prelude//rust:rust_toolchain.bzl", "PanicRuntime", "RustToolchainInfo")
load("@prelude-si//toolchains:common.bzl", "create_download_distribution_function", "create_distribution_provider")
load("@prelude-si//toolchains:extraction.bzl", "ToolchainExtractionInfo")

# Rust version checksums for our S3 artifacts
_RUST_S3_CHECKSUMS = {
    "1.88.0": {
        "linux": {
            "x86_64": "1b3609cf31991835b8384e719f129606ff076ab9ddd1bb2b6695e845f668df35",
            "aarch64": "b31e784911e4f642271a7ba658d0158bbb56e2ba93a6b0170ad51d75fa9c6fa8",
        },
        "darwin": {
            "x86_64": "847e986ec6a9f11c1c48b1e2bd4e623ea92e1a2fd552d43456c8c64d189eaac0",
            "aarch64": "825a4f3843d6b30b0f1fa875b78e58505f461ddf3ba556b61ba3eb28056870f7",
        },
    },
    "nightly-2025-04-17": {
        "linux": {
            "x86_64": "9bd371fd9857e853af6ec66757795540e43ff6edec72c8d16271cf1fe267014a",
            "aarch64": "e8a89ccae40bc7dae32111f814533835a3e4cca38d632d4442d1ed8a1e5a5157",
        },
        "darwin": {
            "x86_64": "319e86e737a12484267263573b8cbfe90e6666d3b4c66a1d9f870404c3f754ac",
            "aarch64": "01dc3ae527d42d0324cdc6b2cfae22c725c55aa6433b0ed8b70a41be66113654",
        },
    },
}

# Create provider using shared utility
RustDistributionInfo = create_distribution_provider({
    "rustc": provider_field(typing.Any, default = None),
    "cargo": provider_field(typing.Any, default = None),
    "rustdoc": provider_field(typing.Any, default = None),
    "clippy": provider_field(typing.Any, default = None),
    "rustfmt": provider_field(typing.Any, default = None),
    "std_lib": provider_field(typing.Any, default = None),
})

def _rust_distribution_impl(ctx: AnalysisContext) -> list[Provider]:
    """Create Rust distribution from extracted S3 toolchain."""

    extraction = ctx.attrs.extraction[ToolchainExtractionInfo]
    extraction_default = ctx.attrs.extraction[DefaultInfo]

    # With unified structure, binaries are directly available
    rustc = cmd_args(extraction.bin_dir, "/rustc", delimiter="")
    cargo = cmd_args(extraction.bin_dir, "/cargo", delimiter="")
    rustdoc = cmd_args(extraction.bin_dir, "/rustdoc", delimiter="")
    clippy = cmd_args(extraction.bin_dir, "/clippy-driver", delimiter="")
    rustfmt = cmd_args(extraction.bin_dir, "/rustfmt", delimiter="")
    std_lib = extraction.lib_dir

    return [
        DefaultInfo(default_output = extraction_default.default_outputs[0]),
        RustDistributionInfo(
            version = ctx.attrs.version,
            target = ctx.attrs.target,
            rustc = rustc,
            cargo = cargo,
            rustdoc = rustdoc,
            clippy = clippy,
            rustfmt = rustfmt,
            std_lib = std_lib,
        ),
    ]

rust_distribution = rule(
    impl = _rust_distribution_impl,
    attrs = {
        "version": attrs.string(),
        "target": attrs.string(),
        "extraction": attrs.dep(providers = [ToolchainExtractionInfo]),
    },
)

# Create download function using shared utility
download_rust_distribution = create_download_distribution_function(
    family = "rust",
    checksums_dict = _RUST_S3_CHECKSUMS,
    distribution_rule = rust_distribution,
    toolchain_name = "Rust"
)

def _hermetic_rust_toolchain_impl(ctx: AnalysisContext) -> list[Provider]:
    """Create a hermetic Rust toolchain from a distribution."""

    dist = ctx.attrs.distribution[RustDistributionInfo]

    # No wrapper scripts needed - binaries work directly
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
