load("@prelude//python:toolchain.bzl", "PythonToolchainInfo")
load("//artifact:toolchain.bzl", "ArtifactToolchainInfo")

"""
Artifact publishing and promotion rules.

# Platform Targets

The platform_targets attribute declares which OS/architecture combinations should be built for an
artifact. Valid platform strings:

  - `darwin-aarch64`
  - `darwin-x86_64`
  - `linux-aarch64`
  - `linux-x86_64`
  - `windows-x86_64`

# Example usage in macros

```bzl
deno_binary_artifact(
    name = "my-tool",
    binary = ":my-tool",
    platform_targets = ["linux-x86_64", "darwin-x86_64"],  # Custom platforms
    skip_all_publish = False,  # Default: allow publishing
    skip_all_promote = True,   # Skip promotion to stable
)
```

# CI Integration

CI systems query platform_targets to determine which platforms to build:

```sh
buck2 uquery //bin/my-tool:publish-binary \
    --output-attribute platform_targets \
    --output-attribute skip_all \
    --json
```

The promote rule uses platform_targets internally to generate `--target` args.
"""

# Valid platform strings
#
# Please keep list sorted
VALID_PLATFORM_TARGETS = [
    "darwin-aarch64",
    "darwin-x86_64",
    "linux-aarch64",
    "linux-x86_64",
    "windows-x86_64",
]

# Valid Rust platform target strings.
#
# TODO(fnichol): remove when cross-compilation is supported, then use list above
#
# Please keep list sorted
VALID_LINUX_PLATFORM_TARGETS = [
    "linux-aarch64",
    "linux-x86_64",
]

def validate_platform_targets(platform_targets: list[str], err_prefix: str) -> None:
    """Validate platform_targets list contains only valid platform strings.

    Args:
        platform_targets: List of platform strings to validate
        err_prefix: Description of where this validation is happening (for error messages)
    """
    _validate_platform_targets_from_list(platform_targets, VALID_PLATFORM_TARGETS, err_prefix)

# TODO(fnichol): remove when cross-compilation is supported, then use list above
def validate_linux_platform_targets(platform_targets: list[str], err_prefix: str) -> None:
    """Validate platform_targets list contains only valid Rust platform strings.

    Args:
        platform_targets: List of platform strings to validate
        err_prefix: Description of where this validation is happening (for error messages)
    """
    _validate_platform_targets_from_list(platform_targets, VALID_LINUX_PLATFORM_TARGETS, err_prefix)

def _validate_platform_targets_from_list(
        platform_targets: list[str],
        valid_platform_targets: list[str],
        err_prefix: str) -> None:
    """Validate platform_targets list contains only valid platform strings.

    Args:
        platform_targets: List of platform strings to validate
        valid_platform_targets: List of all valid platform strings
        err_prefix: Description of where this validation is happening (for error messages)
    """
    if not platform_targets:
        fail("{}: platform_targets cannot be empty".format(err_prefix))

    for target in platform_targets:
        if target not in valid_platform_targets:
            fail(
                "{}: invalid platform '{}'. Supported platforms: {}".format(
                    err_prefix,
                    target,
                    ", ".join(valid_platform_targets),
                ),
            )

ArtifactInfo = provider(fields = {
    "artifact": provider_field(typing.Any, default = None),
    "metadata": provider_field(typing.Any, default = None),
    "family": str,
    "variant": str,
})

def artifact_publish_impl(ctx: AnalysisContext) -> list[[DefaultInfo, RunInfo]]:
    cli_args = ctx.actions.declare_output("args.txt")

    artifact_toolchain = ctx.attrs._artifact_toolchain[ArtifactToolchainInfo]

    cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        artifact_toolchain.publish[DefaultInfo].default_outputs,
        "--destination",
        ctx.attrs.destination,
        "--artifact-file",
        ctx.attrs.artifact[ArtifactInfo].artifact,
        "--metadata-file",
        ctx.attrs.artifact[ArtifactInfo].metadata,
    )
    if ctx.attrs.cname:
        cmd.add("--cname")
        cmd.add(ctx.attrs.cname)

    ctx.actions.write(cli_args.as_output(), cmd)

    return [
        DefaultInfo(default_output = cli_args),
        RunInfo(args = cmd),
    ]

artifact_publish = rule(
    impl = artifact_publish_impl,
    attrs = {
        "destination": attrs.string(
            doc = """Destination [examples: {}].""".format(", ".join([
                "s3://my-bucket",
                "gcs://bucket-name",
                "docker://docker.io",
            ])),
        ),
        "artifact": attrs.dep(
            providers = [ArtifactInfo],
            doc = """The `artifact` to publish.""",
        ),
        "cname": attrs.option(
            attrs.string(),
            default = None,
            doc = """Hostname used when calculating canonical URLs.""",
        ),
        "platform_targets": attrs.list(
            attrs.string(),
            default = [],
            doc = """List of target platforms (e.g., ["linux-x86_64", "darwin-x86_64"]).
            Used by CI to determine which platforms to build.""",
        ),
        "skip_all": attrs.bool(
            default = False,
            doc = """Skip publishing this artifact entirely.
            Useful for temporarily disabling artifact publication.""",
        ),
        "_python_toolchain": attrs.toolchain_dep(
            default = "toolchains//:python",
            providers = [PythonToolchainInfo],
        ),
        "_artifact_toolchain": attrs.toolchain_dep(
            default = "toolchains//:artifact",
            providers = [ArtifactToolchainInfo],
        ),
    },
)

def artifact_promote_impl(ctx: AnalysisContext) -> list[[DefaultInfo, RunInfo]]:
    cli_args = ctx.actions.declare_output("args.txt")

    artifact_toolchain = ctx.attrs._artifact_toolchain[ArtifactToolchainInfo]

    cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        artifact_toolchain.promote[DefaultInfo].default_outputs,
        "--destination",
        ctx.attrs.destination,
        "--channel",
        ctx.attrs.channel,
        "--family",
        ctx.attrs.family,
        "--variant",
        ctx.attrs.variant,
    )
    if ctx.attrs.cname:
        cmd.add("--cname")
        cmd.add(ctx.attrs.cname)

    for target in ctx.attrs.platform_targets:
        cmd.add("--target")
        cmd.add(target)

    ctx.actions.write(cli_args.as_output(), cmd)

    return [
        DefaultInfo(default_output = cli_args),
        RunInfo(args = cmd),
    ]

artifact_promote = rule(
    impl = artifact_promote_impl,
    attrs = {
        "destination": attrs.string(
            doc = """Destination [examples: {}].""".format(", ".join([
                "s3://my-bucket",
                "gcs://bucket-name",
                "docker://docker.io",
            ])),
        ),
        "channel": attrs.string(
            doc = """Release channel.""",
            default = "stable",
        ),
        "family": attrs.string(
            doc = """Artifact family.""",
        ),
        "variant": attrs.string(
            doc = """Artifact variant.""",
        ),
        "cname": attrs.option(
            attrs.string(),
            default = None,
            doc = """Hostname used when calculating canonical URLs.""",
        ),
        "platform_targets": attrs.list(
            attrs.string(),
            default = [],
            doc = """List of target platforms (e.g., ["linux-x86_64", "darwin-x86_64"]).
            Used to generate --target args for promote.py.""",
        ),
        "skip_all": attrs.bool(
            default = False,
            doc = """Skip promoting this artifact entirely.
            Useful for temporarily disabling artifact promotion.""",
        ),
        "_python_toolchain": attrs.toolchain_dep(
            default = "toolchains//:python",
            providers = [PythonToolchainInfo],
        ),
        "_artifact_toolchain": attrs.toolchain_dep(
            default = "toolchains//:artifact",
            providers = [ArtifactToolchainInfo],
        ),
    },
)

def artifact_download_impl(ctx: AnalysisContext) -> list[Provider]:
    """Download an artifact from S3 without extraction."""

    # Construct S3 URL
    filename = "{}-{}-{}-{}.tar.gz".format(
        ctx.attrs.family,
        ctx.attrs.version,
        ctx.attrs.os,
        ctx.attrs.arch,
    )
    url = "{}/{}/{}/{}/{}/{}".format(
        ctx.attrs.destination.rstrip("/"),
        ctx.attrs.family,
        ctx.attrs.version,
        ctx.attrs.os,
        ctx.attrs.arch,
        filename,
    )

    # Download file
    output_file = ctx.actions.declare_output("{}-{}-{}-{}.tar.gz".format(
        ctx.attrs.family,
        ctx.attrs.version,
        ctx.attrs.os,
        ctx.attrs.arch,
    ))

    ctx.actions.download_file(output_file.as_output(), url, sha256 = ctx.attrs.sha256)

    return [DefaultInfo(default_output = output_file)]

artifact_download = rule(
    impl = artifact_download_impl,
    attrs = {
        "destination": attrs.string(
            doc = """Destination base URL (e.g., https://artifacts.systeminit.com/toolchains).""",
            default = "https://artifacts.systeminit.com/toolchains",
        ),
        "family": attrs.string(
            doc = """Artifact family (rust, python, clang, deno).""",
        ),
        "version": attrs.string(
            doc = """Artifact version.""",
        ),
        "os": attrs.string(
            doc = """Operating system (linux, darwin).""",
        ),
        "arch": attrs.string(
            doc = """Architecture (x86_64, aarch64).""",
        ),
        "sha256": attrs.string(
            doc = """SHA256 checksum.""",
        ),
    },
)
