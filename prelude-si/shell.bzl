load(
    "@prelude//python:toolchain.bzl",
    "PythonToolchainInfo",
)
load(
    "//shell:toolchain.bzl",
    "ShellToolchainInfo",
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
    re_executor = get_re_executor_from_props(ctx)

    # We implicitly make the target run from the project root if remote
    # excution options were specified
    run_from_project_root = "buck2_run_from_project_root" in (
        ctx.attrs.labels or []
    ) or re_executor != None

    return inject_test_run_info(
        ctx,
        ExternalRunnerTestInfo(
            type = "shellcheck",
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

shellcheck = rule(
    impl = shellcheck_impl,
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
        "_shell_toolchain": attrs.toolchain_dep(
            default = "toolchains//:shell",
            providers = [ShellToolchainInfo],
        ),
    } | re_test_common.test_args() | inject_test_env.args(),
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

shfmt_check = rule(
    impl = shfmt_check_impl,
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
        "_shell_toolchain": attrs.toolchain_dep(
            default = "toolchains//:shell",
            providers = [ShellToolchainInfo],
        ),
    } | re_test_common.test_args() | inject_test_env.args(),
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
