load(
    "@prelude//python:toolchain.bzl",
    "PythonToolchainInfo",
)
load(
    "//shell:toolchain.bzl",
    "ShellToolchainInfo",
)
load(
    "@prelude//decls/common.bzl",
    "buck",
)
load(
    "@prelude//test/inject_test_run_info.bzl",
    "inject_test_run_info",
)
load(
    "@prelude//tests:re_utils.bzl",
    "get_re_executor_from_props",
)

def shellcheck_impl(ctx: AnalysisContext) -> list[[
    DefaultInfo,
    RunInfo,
    ExternalRunnerTestInfo,
]]:
    shell_toolchain = ctx.attrs._shell_toolchain[ShellToolchainInfo]
    sources_ctx = sources_context(ctx)

    run_cmd_args = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        shell_toolchain.shellcheck[DefaultInfo].default_outputs,
    )
    run_cmd_args.add(sources_ctx.srcs_tree)

    args_file = ctx.actions.write("args.txt", run_cmd_args)

    # Setup a RE executor based on the `remote_execution` param.
    re_executor = get_re_executor_from_props(ctx.attrs.remote_execution)

    return inject_test_run_info(
        ctx,
        ExternalRunnerTestInfo(
            type = "shellcheck",
            command = [run_cmd_args],
            env = ctx.attrs.env,
            labels = ctx.attrs.labels,
            contacts = ctx.attrs.contacts,
            default_executor = re_executor,
            # We implicitly make this test via the project root, instead of
            # the cell root (e.g. fbcode root).
            run_from_project_root = re_executor != None,
            use_project_relative_paths = re_executor != None,
        ),
    ) + [
        DefaultInfo(default_output = args_file),
    ]

shellcheck = rule(
    impl = shellcheck_impl,
    attrs = {
        "srcs": attrs.list(
            attrs.source(),
            default = [],
            doc = """The set of source files to consider."""
        ),
        "env": attrs.dict(
            key = attrs.string(),
            value = attrs.arg(),
            sorted = False,
            default = {},
            doc = """Set environment variables for this rule's invocation of shfmt. The environment
            variable values may include macros which are expanded.""",
        ),
        "labels": attrs.list(
            attrs.string(),
            default = [],
        ),
        "contacts": attrs.list(
            attrs.string(),
            default = [],
        ),
        "remote_execution": buck.re_opts_for_tests_arg(),
        "_inject_test_env": attrs.default_only(
            attrs.dep(default = "prelude//test/tools:inject_test_env"),
        ),
        "_python_toolchain": attrs.toolchain_dep(
            default = "toolchains//:python",
            providers = [PythonToolchainInfo],
        ),
        "_shell_toolchain": attrs.toolchain_dep(
            default = "toolchains//:shell",
            providers = [ShellToolchainInfo],
        ),
    },
)

def shfmt_check_impl(ctx: AnalysisContext) -> list[[
    DefaultInfo,
    RunInfo,
    ExternalRunnerTestInfo,
]]:
    shell_toolchain = ctx.attrs._shell_toolchain[ShellToolchainInfo]
    sources_ctx = sources_context(ctx)

    run_cmd_args = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        shell_toolchain.shfmt_check[DefaultInfo].default_outputs,
    )
    run_cmd_args.add(sources_ctx.srcs_tree)

    args_file = ctx.actions.write("args.txt", run_cmd_args)

    # Setup a RE executor based on the `remote_execution` param.
    re_executor = get_re_executor_from_props(ctx.attrs.remote_execution)

    return inject_test_run_info(
        ctx,
        ExternalRunnerTestInfo(
            type = "shfmt",
            command = [run_cmd_args],
            env = ctx.attrs.env,
            labels = ctx.attrs.labels,
            contacts = ctx.attrs.contacts,
            default_executor = re_executor,
            # We implicitly make this test via the project root, instead of
            # the cell root (e.g. fbcode root).
            run_from_project_root = re_executor != None,
            use_project_relative_paths = re_executor != None,
        ),
    ) + [
        DefaultInfo(default_output = args_file),
    ]

shfmt_check = rule(
    impl = shfmt_check_impl,
    attrs = {
        "srcs": attrs.list(
            attrs.source(),
            default = [],
            doc = """The set of source files to consider."""
        ),
        "env": attrs.dict(
            key = attrs.string(),
            value = attrs.arg(),
            sorted = False,
            default = {},
            doc = """Set environment variables for this rule's invocation of shfmt. The environment
            variable values may include macros which are expanded.""",
        ),
        "labels": attrs.list(
            attrs.string(),
            default = [],
        ),
        "contacts": attrs.list(
            attrs.string(),
            default = [],
        ),
        "remote_execution": buck.re_opts_for_tests_arg(),
        "_inject_test_env": attrs.default_only(
            attrs.dep(default = "prelude//test/tools:inject_test_env"),
        ),
        "_python_toolchain": attrs.toolchain_dep(
            default = "toolchains//:python",
            providers = [PythonToolchainInfo],
        ),
        "_shell_toolchain": attrs.toolchain_dep(
            default = "toolchains//:shell",
            providers = [ShellToolchainInfo],
        ),
    },
)

SourcesContext = record(
    srcs_tree = field(Artifact),
)

def sources_context(ctx: AnalysisContext) -> SourcesContext:
    srcs_tree = ctx.actions.declare_output("__src")

    shell_toolchain = ctx.attrs._shell_toolchain[ShellToolchainInfo]

    cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        shell_toolchain.build_context[DefaultInfo].default_outputs,
    )
    for src in ctx.attrs.srcs:
        cmd.add("--src")
        cmd.add(src)
    if shell_toolchain.editorconfig:
        cmd.add("--editorconfig")
        cmd.add(shell_toolchain.editorconfig)
    cmd.add(srcs_tree.as_output())

    ctx.actions.run(cmd, category = "build_context")

    return SourcesContext(
        srcs_tree = srcs_tree,
    )
