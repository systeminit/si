#load(
#    "//build_context:toolchain.bzl",
#    "BuildContextToolchainInfo",
#)
load(
    "//e2e:toolchain.bzl",
    "E2eToolchainInfo",
)
#load(
#    "//build_context.bzl",
#    "BuildContext",
#    _build_context = "build_context",
#)


E2eTestInfo = provider(fields = {
    "name": provider_field(typing.Any, default = None),
})

def e2e_test_impl(ctx: AnalysisContext) -> list[[
    DefaultInfo,
    E2eTestInfo,
]]:
    if ctx.attrs.pkg_name:
        name = ctx.attrs.pkg_name
    else:
        name = ctx.attrs.name

    test_report = ctx.actions.declare_output("{}".format(name))

    e2e_toolchain = ctx.attrs._e2e_toolchain[E2eToolchainInfo]
    #build_context = _build_context(ctx, [ctx.attrs.build_dep], ctx.attrs.srcs)
    
    cmd = cmd_args(
        "python3",
        e2e_toolchain.e2e_test[DefaultInfo].default_outputs,
        "--output",
        test_report.as_output(),
        #"--build-context-dir",
        #"johns-directory"
        #build_context.root,
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
        #"build_dep": attrs.dep(
        #    doc = """Buck2 target placeholder that might not be used.""",
        #),
        #"_build_context_toolchain": attrs.toolchain_dep(
        #    default = "toolchains//:build_context",
        #    providers = [BuildContextToolchainInfo],
        #),
        "_e2e_toolchain": attrs.toolchain_dep(
            default = "toolchains//:e2e",
            providers = [E2eToolchainInfo],
        ),
    },
)
