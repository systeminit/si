load("@prelude//python:toolchain.bzl", "PythonToolchainInfo")
load("//artifact:toolchain.bzl", "ArtifactToolchainInfo")
load("//artifact.bzl", "ArtifactInfo")
load("//platform.bzl", "get_host_platform")
load("//rootfs:toolchain.bzl", "RootfsToolchainInfo")

def rootfs_tarball_impl(ctx: AnalysisContext) -> list[[
    DefaultInfo,
    ArtifactInfo,
]]:
    artifact_toolchain = ctx.attrs._artifact_toolchain[ArtifactToolchainInfo]
    rootfs_toolchain = ctx.attrs._rootfs_toolchain[RootfsToolchainInfo]

    name = ctx.attrs.rootfs_name or ctx.atrs.name

    artifact = ctx.actions.declare_output("{}.rootfs.ext4".format(name))

    git_metadata_file = ctx.attrs.git_metadata[DefaultInfo].default_outputs[0]

    # Get host platform information
    host_os, host_arch = get_host_platform()

    # Get target platform information from host platform (i.e. not yet cross-compilation aware)
    target_os = host_os
    target_arch = host_arch

    variant = "rootfs"

    # Build artifact
    build_metadata_old = ctx.actions.declare_output("build_metadata-old.json")
    build_cmd = cmd_args(
        "/bin/bash",
        rootfs_toolchain.rootfs_build[DefaultInfo].default_outputs,
        git_metadata_file,
        target_arch,
        build_metadata_old.as_output(),
        artifact.as_output(),
    )
    for dep in ctx.attrs.build_deps or []:
        build_cmd.add(dep)
    ctx.actions.run(build_cmd, category = "build_rootfs_tarball")

    # Generate build metadata
    build_metadata = ctx.actions.declare_output("build_metadata.json")
    metadata_cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        artifact_toolchain.generate_build_metadata[DefaultInfo].default_outputs,
        "--artifact-file",
        artifact,
        "--git-info-json",
        git_metadata_file,
        "--build-metadata-out-file",
        build_metadata.as_output(),
        "--name",
        name,
        "--variant",
        variant,
        "--arch",
        target_arch,
        "--os",
        target_os,
        "--author",
        ctx.attrs.author,
        "--source-url",
        ctx.attrs.source_url,
        "--license",
        ctx.attrs.license,
    )
    ctx.actions.run(metadata_cmd, category = "build_artifact_metadata")

    return [
        DefaultInfo(
            default_output = artifact,
            sub_targets = {
                "metadata": [DefaultInfo(default_output = build_metadata)],
            },
        ),
        ArtifactInfo(
            artifact = artifact,
            metadata = build_metadata,
            family = name,
            variant = variant,
        ),
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
        "author": attrs.string(
            doc = """Image author to be used in artifact metadata.""",
        ),
        "source_url": attrs.string(
            doc = """Source code URL to be used in artifact metadata.""",
        ),
        "license": attrs.string(
            doc = """Image license string to be used in artifact metadata.""",
        ),
        "git_metadata": attrs.dep(
            default = "prelude-si//build_metadata:git",
            doc = """Git metadata target providing repository information.""",
        ),
        "_rootfs_toolchain": attrs.toolchain_dep(
            default = "toolchains//:rootfs",
            providers = [RootfsToolchainInfo],
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
