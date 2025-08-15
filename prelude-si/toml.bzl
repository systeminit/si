load(
    "@prelude//:paths.bzl",
    "paths",
)
load(
    "@prelude//decls/re_test_common.bzl",
    "re_test_common",
)
load(
    "@prelude//python:toolchain.bzl",
    "PythonToolchainInfo",
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
load(
    "//build_context:toolchain.bzl",
    "BuildContextToolchainInfo",
)
load(
    "//toml:toolchain.bzl",
    "TomlToolchainInfo",
)

def toml_format_check_impl(ctx: AnalysisContext) -> list[[
    DefaultInfo,
    RunInfo,
    ExternalRunnerTestInfo,
]]:
    toml_toolchain = ctx.attrs._toml_toolchain[TomlToolchainInfo]
    sources_ctx = sources_context(ctx)

    run_cmd_args = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        toml_toolchain.toml_format[DefaultInfo].default_outputs,
        "--check",
        "--root-dir",
        sources_ctx.srcs_tree,
    )
    
    # Add taplo and cargo paths
    if toml_toolchain.taplo_path:
        run_cmd_args.add("--taplo-path")
        run_cmd_args.add(toml_toolchain.taplo_path)
    if toml_toolchain.cargo_path:
        run_cmd_args.add("--cargo-path")
        run_cmd_args.add(toml_toolchain.cargo_path)
    if toml_toolchain.cargo_sort_path:
        run_cmd_args.add("--cargo-sort-path")
        run_cmd_args.add(toml_toolchain.cargo_sort_path)
    for src in ctx.attrs.srcs:
        run_cmd_args.add(cmd_args(src).relative_to(ctx.label.cell_root))

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
            type = "toml_format_check",
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

toml_format_check = rule(
    impl = toml_format_check_impl,
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
        "_toml_toolchain": attrs.toolchain_dep(
            default = "toolchains//:toml",
            providers = [TomlToolchainInfo],
        ),
    } | re_test_common.test_args() | inject_test_env.args(),
)

def toml_format_impl(ctx: AnalysisContext) -> list[[DefaultInfo, RunInfo]]:
    toml_toolchain = ctx.attrs._toml_toolchain[TomlToolchainInfo]

    run_cmd_args = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        toml_toolchain.toml_format[DefaultInfo].default_outputs,
        "--root-dir",
        ctx.label.cell_root,
    )
    
    # Add taplo and cargo paths
    if toml_toolchain.taplo_path:
        run_cmd_args.add("--taplo-path")
        run_cmd_args.add(toml_toolchain.taplo_path)
    if toml_toolchain.cargo_path:
        run_cmd_args.add("--cargo-path")
        run_cmd_args.add(toml_toolchain.cargo_path)
    if toml_toolchain.cargo_sort_path:
        run_cmd_args.add("--cargo-sort-path")
        run_cmd_args.add(toml_toolchain.cargo_sort_path)
    for src in ctx.attrs.srcs:
        run_cmd_args.add(cmd_args(src).relative_to(ctx.label.cell_root))

    args_file = ctx.actions.write("toml-format-args.txt", run_cmd_args)

    return [
        DefaultInfo(
            default_output = args_file,
        ),
        RunInfo(
            args = run_cmd_args,
        ),
    ]

toml_format = rule(
    impl = toml_format_impl,
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
        "_toml_toolchain": attrs.toolchain_dep(
            default = "toolchains//:toml",
            providers = [TomlToolchainInfo],
        ),
    },
)

SourcesContext = record(
    srcs_tree = field(Artifact),
)

def sources_context(ctx: AnalysisContext) -> SourcesContext:
    srcs_tree = ctx.actions.declare_output("__src")

    toml_toolchain = ctx.attrs._toml_toolchain[TomlToolchainInfo]

    cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        toml_toolchain.build_context[DefaultInfo].default_outputs,
    )
    for src in ctx.attrs.srcs:
        cmd.add("--src")
        cmd.add(src)
    if toml_toolchain.cargo_sort_config:
        cmd.add("--root-src")
        cmd.add(toml_toolchain.cargo_sort_config)
    if toml_toolchain.taplo_config:
        cmd.add("--root-src")
        cmd.add(toml_toolchain.taplo_config)
    cmd.add(srcs_tree.as_output())

    ctx.actions.run(cmd, category = "build_context")

    return SourcesContext(
        srcs_tree = srcs_tree,
    )
