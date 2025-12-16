load("//rootfs:toolchain.bzl", "RootfsToolchainInfo")
load("//git:toolchain.bzl", "GitToolchainInfo")
load(
    "@prelude//python:toolchain.bzl",
    "PythonToolchainInfo",
)
load(
    "//artifact.bzl",
    "ArtifactInfo",
)
load(
    "//git.bzl",
    "GitInfo",
    _git_info = "git_info",
)

RootfsInfo = provider(fields = {
    "tar_archive": provider_field(typing.Any, default = None),  # [Artifact]
})

def rootfs_tarball_impl(ctx: AnalysisContext) -> list[[DefaultInfo, RunInfo, RootfsInfo, ArtifactInfo, GitInfo]]:

    if ctx.attrs.rootfs_name:
        tar_archive = ctx.actions.declare_output("{}.ext4".format(ctx.attrs.rootfs_name))
    else:
        tar_archive = ctx.actions.declare_output("{}.ext4".format(ctx.attrs.name))

    build_metadata = ctx.actions.declare_output("build_metadata.json")

    rootfs_toolchain = ctx.attrs._rootfs_toolchain[RootfsToolchainInfo]
    git_toolchain = ctx.attrs._git_toolchain[GitToolchainInfo]

    git_info = _git_info(ctx)
    cmd = cmd_args(
        "/bin/bash",
        rootfs_toolchain.rootfs_build[DefaultInfo].default_outputs,
        git_info.file,
        build_metadata.as_output(),
        tar_archive.as_output()
    )

    # Add the dependent binary(s) to the build process
    for dep in ctx.attrs.build_deps or []:
        cmd.add(dep)

    ctx.actions.run(cmd, category = "rootfs_tarball_build")

    return [
        DefaultInfo(
            default_output = tar_archive,
        ),
        RootfsInfo(
            tar_archive = tar_archive,
        ),
        ArtifactInfo(
            artifact = tar_archive,
            metadata = build_metadata,
            family = "rootfs",
            variant = "tar",
        ),
        git_info
    ]

rootfs_tarball = rule(
    impl = rootfs_tarball_impl,
    attrs = {
        "rootfs_name": attrs.option(
            attrs.string(),
            default = None,
            doc = """The rootfs output filename.""",
        ),
        "build_deps": attrs.list(
            attrs.source(),
            default = [],
            doc = """Buck2 targets that could be built into a rootfs.""",
        ),
        "_rootfs_toolchain": attrs.toolchain_dep(
            default = "toolchains//:rootfs",
            providers = [RootfsToolchainInfo],
        ),
        "_python_toolchain": attrs.toolchain_dep(
            default = "toolchains//:python",
            providers = [PythonToolchainInfo],
        ),
        "_git_toolchain": attrs.toolchain_dep(
            default = "toolchains//:git",
            providers = [GitToolchainInfo],
        ),
    },
)
