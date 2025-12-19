load("@prelude//python:toolchain.bzl", "PythonToolchainInfo")
load("//artifact:toolchain.bzl", "ArtifactToolchainInfo")
load("//artifact.bzl", "ArtifactInfo")
load("//build_context:toolchain.bzl", "BuildContextToolchainInfo")
load("//build_context.bzl", "BuildContext", _build_context = "build_context")
load("//nix:toolchain.bzl", "NixToolchainInfo")
load("//platform.bzl", "get_host_platform")

NixBinaryInfo = provider(fields = {
    "artifact": provider_field(typing.Any, default = None),  # [Artifact]
    "build_metadata": provider_field(typing.Any, default = None),  # [Artifact]
    "binary_metadata": provider_field(typing.Any, default = None),  # [Artifact]
})

NixOmnibusPkgInfo = provider(fields = {
    "artifact": provider_field(typing.Any, default = None),  # [Artifact]
    "build_metadata": provider_field(typing.Any, default = None),  # [Artifact]
    "pkg_metadata": provider_field(typing.Any, default = None),  # [Artifact]
})

def nix_flake_lock_impl(ctx: AnalysisContext) -> list[DefaultInfo]:
    out = ctx.actions.declare_output("flake.lock")

    output = ctx.actions.copy_file(out, ctx.attrs.src)

    return [DefaultInfo(default_output = out)]

nix_flake_lock = rule(
    impl = nix_flake_lock_impl,
    attrs = {
        "src": attrs.source(
            doc = """flake.lock source.""",
        ),
        "nix_flake": attrs.dep(
            default = "//:flake.nix",
            doc = """Nix flake dependency.""",
        ),
    },
)

def nix_binary_impl(ctx: AnalysisContext) -> list[[
    DefaultInfo,
    NixBinaryInfo,
    ArtifactInfo,
]]:
    if ctx.attrs.binary_name:
        name = ctx.attrs.binary_name
    else:
        name = ctx.attrs.name

    build_context = _build_context(ctx, [ctx.attrs.build_dep], ctx.attrs.srcs)

    git_metadata_file = ctx.attrs.git_metadata[DefaultInfo].default_outputs[0]

    artifact = ctx.actions.declare_output(name)
    build_metadata = ctx.actions.declare_output("build_metadata.json")
    binary_metadata = ctx.actions.declare_output("binary_metadata.json")

    nix_toolchain = ctx.attrs._nix_toolchain[NixToolchainInfo]

    cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        nix_toolchain.nix_binary_build[DefaultInfo].default_outputs,
        "--git-info-json",
        git_metadata_file,
        "--artifact-out-file",
        artifact.as_output(),
        "--build-metadata-out-file",
        build_metadata.as_output(),
        "--binary-metadata-out-file",
        binary_metadata.as_output(),
        "--build-context-dir",
        build_context.root,
        "--name",
        name,
        "--author",
        ctx.attrs.author,
        "--source-url",
        ctx.attrs.source_url,
        "--license",
        ctx.attrs.license,
    )

    ctx.actions.run(cmd, category = "nix_binary_build")

    return [
        DefaultInfo(
            default_output = artifact,
        ),
        NixBinaryInfo(
            artifact = artifact,
            build_metadata = build_metadata,
            binary_metadata = binary_metadata,
        ),
        ArtifactInfo(
            artifact = artifact,
            metadata = build_metadata,
            family = name,
            variant = "binary",
        ),
    ]

nix_binary = rule(
    impl = nix_binary_impl,
    attrs = {
        "binary_name": attrs.option(
            attrs.string(),
            default = None,
            doc = """binary name (default: 'attrs.name').""",
        ),
        "build_dep": attrs.dep(
            doc = """Buck2 target that will be built in a package.""",
        ),
        "srcs": attrs.dict(
            attrs.source(allow_directory = True),
            attrs.string(),
            default = {},
            doc = """Mapping of sources files to the relative directory in a build context..""",
        ),
        "author": attrs.string(
            doc = """Image author to be used in binary metadata.""",
        ),
        "source_url": attrs.string(
            doc = """Source code URL to be used in binary metadata.""",
        ),
        "license": attrs.string(
            doc = """License string to be used in binary metadata.""",
        ),
        "git_metadata": attrs.dep(
            default = "prelude-si//build_metadata:git",
            doc = """Git metadata target providing repository information.""",
        ),
        "_python_toolchain": attrs.toolchain_dep(
            default = "toolchains//:python",
            providers = [PythonToolchainInfo],
        ),
        "_build_context_toolchain": attrs.toolchain_dep(
            default = "toolchains//:build_context",
            providers = [BuildContextToolchainInfo],
        ),
        "_nix_toolchain": attrs.toolchain_dep(
            default = "toolchains//:nix",
            providers = [NixToolchainInfo],
        ),
    },
)

def nix_omnibus_pkg_impl(ctx: AnalysisContext) -> list[[
    DefaultInfo,
    ArtifactInfo,
    NixOmnibusPkgInfo,
]]:
    artifact_toolchain = ctx.attrs._artifact_toolchain[ArtifactToolchainInfo]
    nix_toolchain = ctx.attrs._nix_toolchain[NixToolchainInfo]

    if ctx.attrs.pkg_name:
        name = ctx.attrs.pkg_name
    else:
        name = ctx.attrs.name

    build_context = _build_context(ctx, [ctx.attrs.build_dep], ctx.attrs.srcs)

    git_metadata_file = ctx.attrs.git_metadata[DefaultInfo].default_outputs[0]

    # Get host platform information
    host_os, host_arch = get_host_platform()

    # Get target platform information from host platform (i.e. not yet cross-compilation aware)
    target_os = host_os
    target_arch = host_arch

    variant = "omnibus"

    # Build artifact
    artifact = ctx.actions.declare_output("{}.tar.gz".format(name))
    build_metadata_old = ctx.actions.declare_output("build_metadata-old.json")
    pkg_metadata = ctx.actions.declare_output("pkg_metadata.json")
    build_cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        nix_toolchain.nix_omnibus_pkg_build[DefaultInfo].default_outputs,
        "--git-info-json",
        git_metadata_file,
        "--artifact-out-file",
        artifact.as_output(),
        "--build-metadata-out-file",
        build_metadata_old.as_output(),
        "--pkg-metadata-out-file",
        pkg_metadata.as_output(),
        "--build-context-dir",
        build_context.root,
        "--name",
        name,
        "--author",
        ctx.attrs.author,
        "--source-url",
        ctx.attrs.source_url,
        "--license",
        ctx.attrs.license,
    )
    ctx.actions.run(build_cmd, category = "build_nix_omnibus_pkg")

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
        NixOmnibusPkgInfo(
            artifact = artifact,
            build_metadata = build_metadata,
            pkg_metadata = pkg_metadata,
        ),
    ]

nix_omnibus_pkg = rule(
    impl = nix_omnibus_pkg_impl,
    attrs = {
        "pkg_name": attrs.option(
            attrs.string(),
            default = None,
            doc = """package name (default: 'attrs.name').""",
        ),
        "build_dep": attrs.dep(
            doc = """Buck2 target that will be built in a package.""",
        ),
        "srcs": attrs.dict(
            attrs.source(allow_directory = True),
            attrs.string(),
            default = {},
            doc = """Mapping of sources files to the relative directory in a build context..""",
        ),
        "author": attrs.string(
            doc = """Image author to be used in artifact metadata.""",
        ),
        "source_url": attrs.string(
            doc = """Source code URL to be used in artifact metadata.""",
        ),
        "license": attrs.string(
            doc = """License string to be used in artifact metadata.""",
        ),
        "git_metadata": attrs.dep(
            default = "prelude-si//build_metadata:git",
            doc = """Git metadata target providing repository information.""",
        ),
        "_python_toolchain": attrs.toolchain_dep(
            default = "toolchains//:python",
            providers = [PythonToolchainInfo],
        ),
        "_build_context_toolchain": attrs.toolchain_dep(
            default = "toolchains//:build_context",
            providers = [BuildContextToolchainInfo],
        ),
        "_nix_toolchain": attrs.toolchain_dep(
            default = "toolchains//:nix",
            providers = [NixToolchainInfo],
        ),
        "_artifact_toolchain": attrs.toolchain_dep(
            default = "toolchains//:artifact",
            providers = [ArtifactToolchainInfo],
        ),
    },
)
