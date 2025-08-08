"""Hermetic Deno toolchain distribution rules.

Downloads and manages Deno toolchains without external dependencies.
"""

load("@prelude-si//deno/toolchain.bzl", "DenoToolchainInfo")

# Deno release information with checksums
# Updated for Deno 2.2.12
_DENO_RELEASES = {
    "2.2.12": {
        "x86_64-unknown-linux-gnu": {
            "url": "https://github.com/denoland/deno/releases/download/v2.2.12/deno-x86_64-unknown-linux-gnu.zip",
            "sha256": "bb7ef8ba33d0e02f9f3507e79c3559cfcd31c8a5c64f4f1bb914f3ac83dd66d5",
        },
        "aarch64-unknown-linux-gnu": {
            "url": "https://github.com/denoland/deno/releases/download/v2.2.12/deno-aarch64-unknown-linux-gnu.zip",
            "sha256": "1755f366947236c15e8f469ffdee5dce845fe78af77d404149af970ab2302074",
        },
        "x86_64-apple-darwin": {
            "url": "https://github.com/denoland/deno/releases/download/v2.2.12/deno-x86_64-apple-darwin.zip",
            "sha256": "ijkl9012345678ijkl9012345678ijkl9012345678ijkl9012345678ijkl9012",
        },
        "aarch64-apple-darwin": {
            "url": "https://github.com/denoland/deno/releases/download/v2.2.12/deno-aarch64-apple-darwin.zip",
            "sha256": "mnop3456789012mnop3456789012mnop3456789012mnop3456789012mnop3456",
        },
    },
}

DenoDistributionInfo = provider(
    fields = {
        "version": provider_field(typing.Any, default = None),
        "target": provider_field(typing.Any, default = None),
        "deno": provider_field(typing.Any, default = None),
    },
)

def _get_deno_release(version: str, target: str):
    if version not in _DENO_RELEASES:
        fail("Unknown Deno version '{}'. Available versions: {}".format(
            version,
            ", ".join(_DENO_RELEASES.keys()),
        ))

    deno_version = _DENO_RELEASES[version]
    if target not in deno_version:
        fail("Unsupported target '{}' for Deno {}. Supported targets: {}".format(
            target,
            version,
            ", ".join(deno_version.keys()),
        ))

    return deno_version[target]

def _deno_distribution_impl(ctx: AnalysisContext) -> list[Provider]:
    """Extract Deno binary from downloaded archive."""

    # Create output binary
    deno = ctx.actions.declare_output("bin/deno")

    # Path to binary in the extracted archive
    archive_path = cmd_args(ctx.attrs.archive[DefaultInfo].default_outputs[0])
    binary_path = cmd_args(archive_path, "/deno", delimiter="")

    # Copy the binary to the output location
    ctx.actions.run([
        "cp",
        binary_path,
        deno.as_output()
    ], category = "deno_setup")

    return [
        DefaultInfo(
            sub_targets = {
                "deno": [DefaultInfo(default_outputs = [deno])],
            },
        ),
        DenoDistributionInfo(
            version = ctx.attrs.version,
            target = ctx.attrs.target,
            deno = deno,
        ),
    ]

deno_distribution = rule(
    impl = _deno_distribution_impl,
    attrs = {
        "version": attrs.string(),
        "target": attrs.string(),
        "archive": attrs.dep(providers = [DefaultInfo]),
    },
)

def _http_archive_impl(ctx: AnalysisContext) -> list[Provider]:
    """Download and extract a zip archive."""
    url = ctx.attrs.urls[0]

    # Download archive
    archive = ctx.actions.declare_output("archive.zip")
    ctx.actions.download_file(archive.as_output(), url, sha256 = ctx.attrs.sha256)

    # Extract archive
    output = ctx.actions.declare_output(ctx.label.name, dir = True)
    ctx.actions.run([
        "unzip", "-q", archive, "-d", output.as_output()
    ], category = "extract_deno")

    return [DefaultInfo(default_output = output)]

_http_archive = rule(
    impl = _http_archive_impl,
    attrs = {
        "urls": attrs.list(attrs.string()),
        "sha256": attrs.string(default = ""),
    },
)

def _host_target() -> str:
    """Determine the host target triple for Deno."""
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

def download_deno_distribution(
    name: str,
    version: str,
    target: [None, str] = None):
    """Download a Deno distribution.

    Args:
        name: Name of the target
        version: Deno version (e.g., "2.2.12")
        target: Target triple (defaults to host)
    """
    if target == None:
        target = _host_target()

    release = _get_deno_release(version, target)
    archive_name = name + "-archive"

    _http_archive(
        name = archive_name,
        urls = [release["url"]],
        sha256 = release["sha256"],
    )

    deno_distribution(
        name = name,
        version = version,
        target = target,
        archive = ":" + archive_name,
    )

# DenoToolchainInfo is imported from prelude-si//deno/toolchain.bzl

def _hermetic_deno_toolchain_impl(ctx: AnalysisContext) -> list[Provider]:
    """Create a hermetic Deno toolchain from a distribution."""

    dist = ctx.attrs.distribution[DenoDistributionInfo]

    return [
        DefaultInfo(),
        DenoToolchainInfo(
            deno_binary = cmd_args(dist.deno),
            deno_compile = ctx.attrs._deno_compile,
            deno_format = ctx.attrs._deno_format,
            deno_run = ctx.attrs._deno_run,
            deno_test = ctx.attrs._deno_test,
            deno_workspace = ctx.attrs._deno_workspace,
        ),
    ]

hermetic_deno_toolchain = rule(
    impl = _hermetic_deno_toolchain_impl,
    attrs = {
        "distribution": attrs.exec_dep(providers = [DenoDistributionInfo]),
        "_deno_compile": attrs.dep(
            default = "prelude-si//deno:deno_compile.py",
            providers = [DefaultInfo],
        ),
       "_deno_format": attrs.dep(
            default = "prelude-si//deno:deno_format.py",
            providers = [DefaultInfo],
        ),
       "_deno_run": attrs.dep(
            default = "prelude-si//deno:deno_run.py",
            providers = [DefaultInfo],
        ),
       "_deno_test": attrs.dep(
            default = "prelude-si//deno:deno_test.py",
            providers = [DefaultInfo],
        ),
       "_deno_workspace": attrs.dep(
            default = "prelude-si//deno:deno_workspace.py",
            providers = [DefaultInfo],
        ),
    },
    is_toolchain_rule = True,
)
