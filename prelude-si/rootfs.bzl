load("//rootfs:toolchain.bzl", "RootfsToolchainInfo")
load("//git:toolchain.bzl", "GitToolchainInfo")

RootfsInfo = provider(fields = {
    "tar_archive": provider_field(typing.Any, default = None),  # [Artifact]
})

def rootfs_impl(ctx: AnalysisContext) -> list[[DefaultInfo, RunInfo, RootfsInfo]]:

    rootfs_info = build_rootfs(ctx)

    return [
        DefaultInfo(
            default_output = rootfs_info.tar_archive,
        ),
    ]

rootfs = rule(
    impl = rootfs_impl,
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
        "_git_toolchain": attrs.toolchain_dep(
            default = "toolchains//:git",
            providers = [GitToolchainInfo],
        ),
    },
)

def build_rootfs(ctx: AnalysisContext) -> RootfsInfo:

    tar_archive = ctx.actions.declare_output("{}.tar".format("johns-rootfs"))

    rootfs_toolchain = ctx.attrs._rootfs_toolchain[RootfsToolchainInfo]
    git_toolchain = ctx.attrs._git_toolchain[GitToolchainInfo]

    cmd = cmd_args(
        "/bin/bash",
        rootfs_toolchain.rootfs_build[DefaultInfo].default_outputs,
        git_toolchain.git_info[DefaultInfo].default_outputs,
        tar_archive.as_output()
    )

    # Add the dependent binary(s) to the build process
    for dep in ctx.attrs.build_deps or []:
        cmd.add(dep)

    ctx.actions.run(cmd, category = "rootfs_build")

    return RootfsInfo(
        tar_archive = tar_archive,
    )
