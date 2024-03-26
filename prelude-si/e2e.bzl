load(
    "//e2e:toolchain.bzl",
    "E2eToolchainInfo",
)

E2eTestInfo = provider(fields = {
    "name": provider_field(typing.Any, default = None),
})

def e2e_test_impl(ctx: AnalysisContext) -> list[[
    DefaultInfo,
    RunInfo,
    E2eTestInfo,
]]:
    if ctx.attrs.pkg_name:
        name = ctx.attrs.pkg_name
    else:
        name = ctx.attrs.name

    if ctx.attrs.web_endpoint:
        web_endpoint = ctx.attrs.web_endpoint
    else:
        web_endpoint = "http://localhost:8080"

    if ctx.attrs.e2e_tests:
        e2e_tests = ctx.attrs.e2e_tests
    else:
        e2e_tests = "cypress/e2e/**"

    test_report = ctx.actions.declare_output("{}".format(name))

    e2e_toolchain = ctx.attrs._e2e_toolchain[E2eToolchainInfo]
    cmd = cmd_args(
        "python3",
        e2e_toolchain.e2e_test[DefaultInfo].default_outputs,
        "--output",
        test_report.as_output(),
        "--tests",
        e2e_tests,
        "--web-endpoint",
        web_endpoint,
    )

    ctx.actions.run(cmd, category = "e2e_test")

    return [
        DefaultInfo(
        ),
        RunInfo(
            args = cmd
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
        "web_endpoint": attrs.option(
            attrs.string(),
            default = None,
            doc = """web endpoint to test against (default http://localhost:8080)""",
        ),
        "e2e_tests": attrs.option(
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
