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
        "--artifact-file",
        ctx.attrs.artifact[ArtifactInfo].artifact,
        "--metadata-file",
        ctx.attrs.artifact[ArtifactInfo].metadata,
        "--family",
        ctx.attrs.artifact[ArtifactInfo].family,
        "--variant",
        ctx.attrs.artifact[ArtifactInfo].variant,
        # This S3 destination is our public artifacts portfolio
        # we can parse s3:// variants off the front in the future.
        # i.e. we could pass gcs://bucket-name or docker://reg name
        # within the python
        "--destination",
        "s3://si-artifacts-prod"
    )

    ctx.actions.write(cli_args.as_output(), cmd)

    return [
        DefaultInfo(default_output = cli_args),
        RunInfo(args = cmd),
    ]

artifact_publish = rule(
    impl = artifact_publish_impl,
    attrs = {
        "artifact": attrs.dep(
            providers = [ArtifactInfo],
            doc = """The `artifact` to publish.""",
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
