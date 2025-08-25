"""
Toolchain extraction rules for unified tar.gz archives.

Extracts toolchain archives downloaded via artifact_download into usable toolchains.
All toolchains follow the unified structure: toolchain/{bin,lib,share}
"""

load("@prelude-si//:artifact.bzl", "artifact_download")

ToolchainExtractionInfo = provider(
    fields = {
        "family": provider_field(str),
        "version": provider_field(str), 
        "os": provider_field(str),
        "arch": provider_field(str),
        "toolchain_dir": provider_field(typing.Any),
        "bin_dir": provider_field(typing.Any),
        "lib_dir": provider_field(typing.Any),
    },
)

def _toolchain_extract_impl(ctx: AnalysisContext) -> list[Provider]:
    """Extract a unified toolchain tar.gz archive."""
    
    archive = ctx.attrs.archive[DefaultInfo].default_outputs[0]
    
    # Extract archive - unified structure means simple extraction
    extracted_dir = ctx.actions.declare_output("extracted", dir = True)
    # Create script to extract with proper error handling
    extract_script = ctx.actions.declare_output("extract.sh")
    ctx.actions.write(
        extract_script,
        [
            "#!/bin/bash",
            "set -e",
            "OUTPUT_DIR=\"$1\"",
            "ARCHIVE=\"$2\"", 
            "mkdir -p \"$OUTPUT_DIR\"",
            "tar xzf \"$ARCHIVE\" -C \"$OUTPUT_DIR\"",
        ],
        is_executable = True,
    )
    
    ctx.actions.run([
        extract_script, extracted_dir.as_output(), archive
    ], category = "extract_toolchain")
    
    # The archive contains a single 'toolchain/' directory at the root  
    toolchain_dir = cmd_args(extracted_dir, "/toolchain", delimiter="")
    bin_dir = cmd_args(extracted_dir, "/toolchain/bin", delimiter="")
    lib_dir = cmd_args(extracted_dir, "/toolchain/lib", delimiter="")
    
    return [
        DefaultInfo(default_output = extracted_dir),
        ToolchainExtractionInfo(
            family = ctx.attrs.family,
            version = ctx.attrs.version,
            os = ctx.attrs.os,
            arch = ctx.attrs.arch,
            toolchain_dir = toolchain_dir,  # Points to extracted/toolchain
            bin_dir = bin_dir, 
            lib_dir = lib_dir,
        ),
    ]

toolchain_extract = rule(
    impl = _toolchain_extract_impl,
    attrs = {
        "archive": attrs.dep(providers = [DefaultInfo], doc = "Downloaded toolchain archive"),
        "family": attrs.string(doc = "Toolchain family (rust, python, clang, deno)"),
        "version": attrs.string(doc = "Toolchain version"),
        "os": attrs.string(doc = "Operating system (linux, darwin)"),
        "arch": attrs.string(doc = "Architecture (x86_64, aarch64)"),
    },
)

def _host_os_arch():
    """Get host OS and arch in standardized format."""
    arch = host_info().arch
    os = host_info().os
    
    if arch.is_x86_64:
        arch_str = "x86_64"
    elif arch.is_aarch64:
        arch_str = "aarch64"
    else:
        fail("Unsupported host architecture")
        
    if os.is_linux:
        os_str = "linux"
    elif os.is_macos:
        os_str = "darwin"
    else:
        fail("Unsupported host OS")
        
    return os_str, arch_str

def download_and_extract_toolchain(
    name: str,
    family: str,
    version: str, 
    sha256: str,
    os: [None, str] = None,
    arch: [None, str] = None,
    visibility: [None, list] = None):
    """Download and extract a toolchain from S3.
    
    Creates two targets:
    - {name}-download: Downloads the tar.gz file
    - {name}: Extracts the toolchain
    
    Args:
        name: Target name for the extracted toolchain
        family: Toolchain family (rust, python, clang, deno)
        version: Version string
        sha256: SHA256 checksum
        os: OS (defaults to host)
        arch: Architecture (defaults to host)
        visibility: Target visibility
    """
    if os == None or arch == None:
        host_os, host_arch = _host_os_arch()
        if os == None:
            os = host_os
        if arch == None:
            arch = host_arch
    
    download_name = name + "-download"
    
    # Download the artifact
    artifact_download(
        name = download_name,
        family = family,
        version = version,
        os = os,
        arch = arch,
        sha256 = sha256,
        visibility = [":" + name],  # Only visible to extraction target
    )
    
    # Extract the toolchain  
    toolchain_extract(
        name = name,
        archive = ":" + download_name,
        family = family,
        version = version,
        os = os,
        arch = arch,
        visibility = visibility,
    )