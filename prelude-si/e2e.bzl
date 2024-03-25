
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
    "//e2e:toolchain.bzl",
    "E2eToolchainInfo",
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
load(
    "//artifact.bzl",
    "ArtifactInfo",
)

E2eTestInfo = provider(fields = {
    "name": provider_field(typing.Any, default = None),
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

def e2e_test_impl(ctx: AnalysisContext) -> list[[
    DefaultInfo,
    E2eTestInfo,
]]:
    if ctx.attrs.pkg_name:
        name = ctx.attrs.pkg_name
    else:
        name = ctx.attrs.name

    test_report = ctx.actions.declare_output("{}.html".format(name))

    e2e_toolchain = ctx.attrs._e2e_toolchain[E2eToolchainInfo]

    cmd = cmd_args(
        "/bin/bash",
        e2e_toolchain.e2e_test[DefaultInfo].default_outputs,
        test_report.as_output(),
    )

    ctx.actions.run(cmd, category = "e2e_test")

    return [
        DefaultInfo(
            default_output = test_report,
        ),
        E2eTestInfo(
            name = test_report
        ),
    ]

e2e_test = rule(
    impl = e2e_test_impl,
    attrs = {
        "pkg_name": attrs.option(
            attrs.string(),
            default = None,
            doc = """package name (default: 'attrs.name').""",
        ),
        "_e2e_toolchain": attrs.toolchain_dep(
            default = "toolchains//:e2e",
            providers = [E2eToolchainInfo],
        ),
    },
)
