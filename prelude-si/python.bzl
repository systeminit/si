load(
    "@prelude//python:toolchain.bzl",
    "PythonToolchainInfo",
)
load(
    "//build_context:toolchain.bzl",
    "BuildContextToolchainInfo",
)
load(
    "//python:toolchain.bzl",
    "SiPythonToolchainInfo",
)
load(
    "@prelude//decls/re_test_common.bzl",
    "re_test_common",
)
load(
    "@prelude//test/inject_test_run_info.bzl",
    "inject_test_run_info",
)
load(
    "@prelude//tests:re_utils.bzl",
    "get_re_executor_from_props",
)
load(
    "@prelude-si//:test.bzl",
    "inject_test_env",
)
load(
    "@prelude//:paths.bzl",
    "paths",
)
load(
    "//build_context.bzl",
    "BuildContext",
    _build_context = "build_context",
)

def yapf_check_impl(ctx: AnalysisContext) -> list[[
    DefaultInfo,
    RunInfo,
    ExternalRunnerTestInfo,
]]:
    srcs = {}
    for src in ctx.attrs.srcs:
        # An empty string triggers build_context.py to map `src` into the
        # `dirname(src)` directory in the build context
        srcs[src] = ""
    build_context = _build_context(ctx, [], srcs)

    si_python_toolchain = ctx.attrs._si_python_toolchain[SiPythonToolchainInfo]

    run_cmd_args = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        si_python_toolchain.yapf_check[DefaultInfo].default_outputs,
    )
    run_cmd_args.add(build_context.root)

    args_file = ctx.actions.write("args.txt", run_cmd_args)

    # Setup a RE executor based on the `remote_execution` param.
    re_executor = get_re_executor_from_props(ctx)

    # We implicitly make the target run from the project root if remote
    # excution options were specified
    run_from_project_root = "buck2_run_from_project_root" in (
        ctx.attrs.labels or []
    ) or re_executor != None

    return inject_test_run_info(
        ctx,
        ExternalRunnerTestInfo(
            type = "shfmt",
            command = [run_cmd_args],
            env = ctx.attrs.env,
            labels = ctx.attrs.labels,
            contacts = ctx.attrs.contacts,
            default_executor = re_executor,
            run_from_project_root = run_from_project_root,
            use_project_relative_paths = run_from_project_root,
        ),
    ) + [
        DefaultInfo(default_output = args_file),
    ]

yapf_check = rule(
    impl = yapf_check_impl,
    attrs = {
        "srcs": attrs.list(
            attrs.source(),
            default = [],
            doc = """The set of source files to consider.""",
        ),
        "_python_toolchain": attrs.toolchain_dep(
            default = "toolchains//:python",
            providers = [PythonToolchainInfo],
        ),
        "_build_context_toolchain": attrs.toolchain_dep(
            default = "toolchains//:build_context",
            providers = [BuildContextToolchainInfo],
        ),
        "_si_python_toolchain": attrs.toolchain_dep(
            default = "toolchains//:si_python",
            providers = [SiPythonToolchainInfo],
        ),
    } | re_test_common.test_args() | inject_test_env.args(),
)
