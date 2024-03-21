load("//e2e:toolchain.bzl", "E2eToolchainInfo")

E2eInfo = provider(fields = {
    "report": provider_field(typing.Any, default = None),  # Potentially an artifact we can distribute alongside (?)
})

def e2e_impl(ctx: AnalysisContext) -> list[[DefaultInfo, RunInfo, E2eInfo]]:

    e2e_info = e2e_test(ctx)

    return [
        DefaultInfo(
            default_output = e2e_info.report,
        ),
    ]

e2e = rule(
    impl = e2e_impl,
    attrs = {
        "e2e_name": attrs.option(
            attrs.string(),
            default = None,
            doc = """The e2e output report filename.""",
        ),
        "_e2e_toolchain": attrs.toolchain_dep(
            default = "toolchains//:e2e",
            providers = [E2eToolchainInfo],
        ),
    },
)

def e2e_test(ctx: AnalysisContext) -> E2eInfo:

    e2eInfo = ctx.actions.declare_output("{}.html".format("example-e2e-result"))

    e2e_toolchain = ctx.attrs._e2e_toolchain[E2eToolchainInfo]

    cmd = cmd_args(
        "/bin/bash",
        e2e_toolchain.e2e_build[DefaultInfo].default_outputs,
        e2eInfo.as_output()
    )

    ctx.actions.run(cmd, category = "e2e_test")

    return E2eInfo(
        report = e2eInfo,
    )