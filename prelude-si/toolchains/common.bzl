"""Common utilities for toolchain distribution rules.

Provides shared functions to eliminate duplication across *_distribution.bzl files.
"""

load("@prelude-si//toolchains:extraction.bzl", "download_and_extract_toolchain", "ToolchainExtractionInfo")

def get_toolchain_checksum(checksums_dict, version: str, os: str, arch: str, toolchain_name: str):
    """Generic checksum lookup for any toolchain type."""
    if version not in checksums_dict:
        fail("Unknown {} version '{}'. Available versions: {}".format(
            toolchain_name,
            version,
            ", ".join(checksums_dict.keys()),
        ))

    version_dict = checksums_dict[version]
    if os not in version_dict:
        fail("Unsupported OS '{}' for {} {}. Supported: {}".format(
            os, toolchain_name, version, ", ".join(version_dict.keys())))
        
    os_dict = version_dict[os]
    if arch not in os_dict:
        fail("Unsupported arch '{}' for {} {} on {}. Supported: {}".format(
            arch, toolchain_name, version, os, ", ".join(os_dict.keys())))

    return os_dict[arch]

def host_target() -> str:
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

def target_to_os_arch(target: str):
    """Convert target triple to standardized os/arch."""
    if "linux" in target:
        os = "linux"
    elif "darwin" in target or "apple" in target:
        os = "darwin"
    else:
        fail("Unsupported target OS: " + target)
        
    if "x86_64" in target:
        arch = "x86_64"
    elif "aarch64" in target:
        arch = "aarch64"
    else:
        fail("Unsupported target arch: " + target)
        
    return os, arch

def create_distribution_provider(additional_fields = {}):
    """Create a distribution provider with common fields plus tool-specific ones."""
    common_fields = {
        "version": provider_field(typing.Any, default = None),
        "target": provider_field(typing.Any, default = None),
    }
    common_fields.update(additional_fields)
    return provider(fields = common_fields)

def create_generic_distribution_rule(provider_type, field_mapper_fn):
    """Create a generic distribution rule implementation.
    
    Args:
        provider_type: The provider type to return (e.g., RustDistributionInfo)
        field_mapper_fn: Function that takes (ctx, extraction) and returns dict of provider fields
    """
    def _distribution_impl(ctx: AnalysisContext) -> list[Provider]:
        extraction = ctx.attrs.extraction[ToolchainExtractionInfo]
        
        # Get toolchain-specific fields from the mapper function
        provider_fields = field_mapper_fn(ctx, extraction)
        
        return [
            DefaultInfo(),
            provider_type(**provider_fields),
        ]
    
    return rule(
        impl = _distribution_impl,
        attrs = {
            "version": attrs.string(),
            "target": attrs.string(), 
            "extraction": attrs.dep(providers = [ToolchainExtractionInfo]),
        },
    )

def create_download_distribution_function(
        family: str,
        checksums_dict,
        distribution_rule,
        toolchain_name: str):
    """Factory function to create download functions for toolchains."""
    
    def download_distribution(
            name: str,
            version: str,
            target: [None, str] = None,
            visibility: [None, list] = None):
        """Download a toolchain distribution from S3.
        
        Args:
            name: Name of the target
            version: Toolchain version
            target: Target triple (defaults to host)
            visibility: Target visibility
        """
        if target == None:
            target = host_target()

        os, arch = target_to_os_arch(target)
        sha256 = get_toolchain_checksum(checksums_dict, version, os, arch, toolchain_name)
        extraction_name = name + "-extraction"

        # Download and extract from S3
        download_and_extract_toolchain(
            name = extraction_name,
            family = family,
            version = version,
            sha256 = sha256,
            os = os,
            arch = arch,
            visibility = [":" + name],
        )

        # Create distribution from extraction
        distribution_rule(
            name = name,
            version = version,
            target = target,
            extraction = ":" + extraction_name,
            visibility = visibility,
        )
    
    return download_distribution