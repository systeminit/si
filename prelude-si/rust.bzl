load(
    "@prelude//python:toolchain.bzl",
    "PythonToolchainInfo",
)
load(
    "//rust:toolchain.bzl",
    "SiRustToolchainInfo",
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
    "get_re_executors_from_props",
)
load(
    "@prelude-si//:test.bzl",
    "inject_test_env",
)

def clippy_check_impl(ctx: AnalysisContext) -> list[[
    DefaultInfo,
    RunInfo,
    ExternalRunnerTestInfo,
]]:
    clippy_txt = ctx.attrs.clippy_txt_dep[DefaultInfo].default_outputs

    si_rust_toolchain = ctx.attrs._si_rust_toolchain[SiRustToolchainInfo]

    run_cmd_args = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        si_rust_toolchain.clippy_output[DefaultInfo].default_outputs,
        clippy_txt,
    )

    args_file = ctx.actions.write("args.txt", run_cmd_args)

    # Setup a RE executor based on the `remote_execution` param.
    re_executor, executor_overrides = get_re_executors_from_props(ctx)

    # We implicitly make the target run from the project root if remote
    # excution options were specified
    run_from_project_root = "buck2_run_from_project_root" in (
        ctx.attrs.labels or []
    ) or re_executor != None

    return inject_test_run_info(
        ctx,
        ExternalRunnerTestInfo(
            type = "clippy",
            command = [run_cmd_args],
            env = ctx.attrs.env,
            labels = ctx.attrs.labels,
            contacts = ctx.attrs.contacts,
            default_executor = re_executor,
            executor_overrides = executor_overrides,
            run_from_project_root = run_from_project_root,
            use_project_relative_paths = run_from_project_root,
        ),
    ) + [
        DefaultInfo(default_output = args_file),
    ]

clippy_check = rule(
    impl = clippy_check_impl,
    attrs = {
        "clippy_txt_dep": attrs.dep(
            doc = """Clippy sub target dep from a Rust library or binary""",
        ),
        "_python_toolchain": attrs.toolchain_dep(
            default = "toolchains//:python",
            providers = [PythonToolchainInfo],
        ),
        "_si_rust_toolchain": attrs.toolchain_dep(
            default = "toolchains//:si_rust",
            providers = [SiRustToolchainInfo],
        ),
    } | re_test_common.test_args() | inject_test_env.args(),
)

def rustfmt_check_impl(ctx: AnalysisContext) -> list[[
    DefaultInfo,
    RunInfo,
    ExternalRunnerTestInfo,
]]:
    si_rust_toolchain = ctx.attrs._si_rust_toolchain[SiRustToolchainInfo]
    crate_ctx = crate_context(ctx)

    run_cmd_args = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        si_rust_toolchain.rustfmt_check[DefaultInfo].default_outputs,
    )
    if si_rust_toolchain.rustfmt_toml:
        run_cmd_args.add("--config-path")
        run_cmd_args.add(si_rust_toolchain.rustfmt_toml)
    run_cmd_args.add(cmd_args(
        [crate_ctx.srcs_tree, ctx.label.package, ctx.attrs.crate_root],
        delimiter = "/",
    ))

    args_file = ctx.actions.write("args.txt", run_cmd_args)

    # Setup a RE executor based on the `remote_execution` param.
    re_executor, executor_overrides = get_re_executors_from_props(ctx)

    # We implicitly make the target run from the project root if remote
    # excution options were specified
    run_from_project_root = "buck2_run_from_project_root" in (
        ctx.attrs.labels or []
    ) or re_executor != None

    return inject_test_run_info(
        ctx,
        ExternalRunnerTestInfo(
            type = "rustfmt",
            command = [run_cmd_args],
            env = ctx.attrs.env,
            labels = ctx.attrs.labels,
            contacts = ctx.attrs.contacts,
            default_executor = re_executor,
            executor_overrides = executor_overrides,
            run_from_project_root = run_from_project_root,
            use_project_relative_paths = run_from_project_root,
        ),
    ) + [
        DefaultInfo(default_output = args_file),
    ]

rustfmt_check = rule(
    impl = rustfmt_check_impl,
    attrs = {
        "srcs": attrs.list(
            attrs.source(),
            default = [],
            doc = """The set of Rust source files in the crate.""",
        ),
        "crate_root": attrs.string(
            doc = """Top level source file for the crate.""",
        ),
        "_python_toolchain": attrs.toolchain_dep(
            default = "toolchains//:python",
            providers = [PythonToolchainInfo],
        ),
        "_si_rust_toolchain": attrs.toolchain_dep(
            default = "toolchains//:si_rust",
            providers = [SiRustToolchainInfo],
        ),
    } | re_test_common.test_args() | inject_test_env.args(),
)

CrateContext = record(
    srcs_tree = field(Artifact),
)

def crate_context(ctx: AnalysisContext) -> CrateContext:
    srcs_tree = ctx.actions.declare_output("__src")

    si_rust_toolchain = ctx.attrs._si_rust_toolchain[SiRustToolchainInfo]

    cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        si_rust_toolchain.crate_context[DefaultInfo].default_outputs,
    )
    for src in ctx.attrs.srcs:
        cmd.add("--src")
        cmd.add(src)
    cmd.add(srcs_tree.as_output())

    ctx.actions.run(cmd, category = "crate_context")

    return CrateContext(
        srcs_tree = srcs_tree,
    )
