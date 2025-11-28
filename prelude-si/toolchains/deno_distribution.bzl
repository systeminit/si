"""Hermetic Deno toolchain distribution rules using S3 artifacts.

Downloads pre-packaged Deno toolchains from si-artifacts-prod.
Much simpler than the original - no complex extraction needed.
"""

load(
    "@prelude-si//toolchains:common.bzl",
    "create_distribution_provider",
    "create_download_distribution_function",
    "get_toolchain_checksum",
    "host_target",
    "target_to_os_arch",
)
load(
    "@prelude-si//toolchains:extraction.bzl",
    "ToolchainExtractionInfo",
)

# Deno version checksums for our S3 artifacts
_DENO_S3_CHECKSUMS = {
    "2.2.12": {
        "linux": {
            "x86_64": "084d318f605f0d302ea2503b00082cfd97559f3a24286eb2970971344918a8cc",
            "aarch64": "2f1450598f09eff5a7457cb78a0bc32e8e61bd1b505f6f1893a2f5cbc845e4d1",
        },
        "darwin": {
            "x86_64": "e5d4e8a4c53b7bc9812ea980d5c4a43c514811c3e92a6668d8771460c801934f",
            "aarch64": "bb7d95749f71863d89e03031cbc2f163af7259825d9feb95c6de3ac4cf3e8f60",
        },
    },
}

# Create provider using shared utility
DenoDistributionInfo = create_distribution_provider({
    "deno": provider_field(typing.Any, default = None),
})

def _deno_field_mapper(ctx: AnalysisContext, extraction):
    """Map extraction to Deno-specific provider fields."""
    return {
        "version": ctx.attrs.version,
        "target": ctx.attrs.target,
        "deno": cmd_args(extraction.bin_dir, "/deno", delimiter = ""),
    }

def _deno_distribution_impl(ctx: AnalysisContext) -> list[Provider]:
    """Create Deno distribution from extracted S3 toolchain."""
    extraction = ctx.attrs.extraction[ToolchainExtractionInfo]

    return [
        DefaultInfo(),
        DenoDistributionInfo(**_deno_field_mapper(ctx, extraction)),
    ]

deno_distribution = rule(
    impl = _deno_distribution_impl,
    attrs = {
        "version": attrs.string(),
        "target": attrs.string(),
        "extraction": attrs.dep(providers = [ToolchainExtractionInfo]),
    },
)

# Create download function using shared utility
download_deno_distribution = create_download_distribution_function(
    family = "deno",
    checksums_dict = _DENO_S3_CHECKSUMS,
    distribution_rule = deno_distribution,
    toolchain_name = "Deno",
)
