load(
    "@prelude//python:toolchain.bzl",
    "PythonToolchainInfo",
)
load(
    "//build_context:toolchain.bzl",
    "BuildContextToolchainInfo",
)
load(
    "//git:toolchain.bzl",
    "GitToolchainInfo",
)
load(
    "//nix:toolchain.bzl",
    "NixToolchainInfo",
)
load(
    "//build_context.bzl",
    "BuildContext",
    _build_context = "build_context",
)
load(
    "//git.bzl",
    "GitInfo",
    _git_info = "git_info",
)

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

def nix_omnibus_pkg_impl(ctx: AnalysisContext) -> list[[
    DefaultInfo,
    NixOmnibusPkgInfo,
    GitInfo,
]]:
    if ctx.attrs.pkg_name:
        name = ctx.attrs.pkg_name
    else:
        name = ctx.attrs.name

    build_context = _build_context(ctx, [ctx.attrs.build_dep], ctx.attrs.srcs)
    git_info = _git_info(ctx)

    artifact = ctx.actions.declare_output("{}.tar.gz".format(name))
    build_metadata = ctx.actions.declare_output("build_metadata.json")
    pkg_metadata = ctx.actions.declare_output("pkg_metadata.json")

    nix_toolchain = ctx.attrs._nix_toolchain[NixToolchainInfo]

    cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        nix_toolchain.nix_omnibus_pkg_build[DefaultInfo].default_outputs,
        "--git-info-json",
        git_info.file,
        "--artifact-out-file",
        artifact.as_output(),
        "--build-metadata-out-file",
        build_metadata.as_output(),
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

    ctx.actions.run(cmd, category = "nix_omnibus_pkg_build")

    return [
        DefaultInfo(
            default_output = artifact,
        ),
        NixOmnibusPkgInfo(
            artifact = artifact,
            build_metadata = build_metadata,
            pkg_metadata = pkg_metadata,
        ),
        git_info,
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
            doc = """Image author to be used in image metadata.""",
        ),
        "source_url": attrs.string(
            doc = """Source code URL to be used in image metadata.""",
        ),
        "license": attrs.string(
            doc = """Image license string to be used in image metadata.""",
        ),
        "_python_toolchain": attrs.toolchain_dep(
            default = "toolchains//:python",
            providers = [PythonToolchainInfo],
        ),
        "_build_context_toolchain": attrs.toolchain_dep(
            default = "toolchains//:build_context",
            providers = [BuildContextToolchainInfo],
        ),
        "_git_toolchain": attrs.toolchain_dep(
            default = "toolchains//:git",
            providers = [GitToolchainInfo],
        ),
        "_nix_toolchain": attrs.toolchain_dep(
            default = "toolchains//:nix",
            providers = [NixToolchainInfo],
        ),
    },
)
