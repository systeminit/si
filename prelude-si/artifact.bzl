load("@prelude//python:toolchain.bzl", "PythonToolchainInfo")
load("//artifact:toolchain.bzl", "ArtifactToolchainInfo")

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
