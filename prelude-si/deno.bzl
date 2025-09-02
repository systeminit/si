load(
    "//deno:toolchain.bzl",
    "DenoToolchainInfo",
    "DenoWorkspaceInfo",
)
load(
    "@prelude//python:toolchain.bzl",
    "PythonToolchainInfo",
)
load("@prelude//utils:cmd_script.bzl", "cmd_script")

def deno_compile_impl(ctx: AnalysisContext) -> list[Provider]:
    out = ctx.actions.declare_output(ctx.attrs.out)
    deno_toolchain = ctx.attrs._deno_toolchain[DenoToolchainInfo]

    cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        deno_toolchain.deno_compile[DefaultInfo].default_outputs[0],
        "--deno-binary",
        deno_toolchain.deno_binary,
        "--input",
        ctx.attrs.main,
        "--extra-srcs",
        ctx.attrs.extra_srcs,
        "--output",
        out.as_output(),
        hidden=ctx.attrs.srcs,
    )

    if ctx.attrs.deno_cache:
        deno_cache_provider = ctx.attrs.deno_cache[DenoWorkspaceInfo]
        cmd.add("--deno-dir", deno_cache_provider.cache_dir)
        cmd.add("--workspace-dir", deno_cache_provider.workspace_dir)

    if ctx.attrs.permissions:
        cmd.add("--permissions", *ctx.attrs.permissions)

    if ctx.attrs.unstable_flags:
        cmd.add("--unstable-flags", *ctx.attrs.unstable_flags)

    # Add includes for JavaScript files from extra_srcs (generated files)
    for src in ctx.attrs.extra_srcs:
        if src.basename.endswith(".js"):
            cmd.add("--includes", src)

    ctx.actions.run(cmd, category="deno", identifier="deno_compile")

    return [
        DefaultInfo(default_output=out),
        RunInfo(args=cmd_args(out)),
    ]


deno_compile = rule(
    impl=deno_compile_impl,
    attrs={
        "main":
        attrs.source(doc="The entry point TypeScript/JavaScript file"),
        "srcs":
        attrs.list(attrs.source(),
                   default=[],
                   doc="All source files that are part of the compilation"),
        "out":
        attrs.string(doc="The name of the output binary"),
        "extra_srcs": attrs.list(
            attrs.source(),
            default = [],
            doc = "Sources from other targets",
        ),
        "deno_cache":
        attrs.option(attrs.dep(providers=[DenoWorkspaceInfo]), default=None),
        "permissions":
        attrs.list(
            attrs.string(),
            default=[],
            doc="List of Deno permissions to grant (e.g., read, write, net)"),
        "unstable_flags":
        attrs.list(attrs.string(),
                   default=[],
                   doc="List of unstable flags to enable"),
        "_python_toolchain":
        attrs.toolchain_dep(default="toolchains//:python",
                            providers=[PythonToolchainInfo]),
        "_deno_toolchain":
        attrs.toolchain_dep(default="toolchains//:deno",
                            providers=[DenoToolchainInfo]),
    },
)


def deno_format_impl(ctx: AnalysisContext) -> list[Provider]:
    """Implementation of the deno_format rule."""
    deno_toolchain = ctx.attrs._deno_toolchain[DenoToolchainInfo]

    cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        deno_toolchain.deno_format[DefaultInfo].default_outputs[0],
        "--deno-binary",
        deno_toolchain.deno_binary,
    )

    for src in ctx.attrs.srcs:
        cmd.add("--input", src)

    if ctx.attrs.check:
        cmd.add("--check")

    if ctx.attrs.ignore:
        for ignore_path in ctx.attrs.ignore:
            cmd.add("--ignore=" + ignore_path)

    return [
        DefaultInfo(),
        RunInfo(args=cmd),
    ]


deno_format = rule(
    impl=deno_format_impl,
    attrs={
        "srcs":
        attrs.list(
            attrs.source(),
            default=[],
            doc="The source files to format",
        ),
        "check":
        attrs.bool(
            default=False,
            doc="Check if files are formatted without making changes",
        ),
        "ignore":
        attrs.list(
            attrs.string(),
            default=[],
            doc="List of files or directories to ignore",
        ),
        "_python_toolchain":
        attrs.toolchain_dep(
            default="toolchains//:python",
            providers=[PythonToolchainInfo],
        ),
        "_deno_toolchain":
        attrs.toolchain_dep(
            default="toolchains//:deno",
            providers=[DenoToolchainInfo],
        ),
    },
)


def deno_run_impl(ctx: AnalysisContext) -> list[Provider]:
    out = ctx.actions.declare_output(ctx.attrs.out)
    deno_toolchain = ctx.attrs._deno_toolchain[DenoToolchainInfo]

    cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        deno_toolchain.deno_run[DefaultInfo].default_outputs[0],
        "--deno-binary",
        deno_toolchain.deno_binary,
        "--input",
        ctx.attrs.main,
        "--output",
        out.as_output(),
        hidden=ctx.attrs.srcs)

    if ctx.attrs.deno_cache:
        deno_cache_provider = ctx.attrs.deno_cache[DenoWorkspaceInfo]
        cmd.add("--deno-dir", deno_cache_provider.cache_dir)
        cmd.add("--workspace-dir", deno_cache_provider.workspace_dir)

    if ctx.attrs.permissions:
        cmd.add("--permissions", *ctx.attrs.permissions)

    if ctx.attrs.unstable_flags:
        cmd.add("--unstable-flags", *ctx.attrs.unstable_flags)

    ctx.actions.run(cmd, category="deno", identifier="deno_run")

    return [
        DefaultInfo(default_output=out),
    ]


deno_run = rule(
    impl=deno_run_impl,
    attrs={
        "main":
        attrs.source(doc="The entry point TypeScript/JavaScript file"),
        "srcs":
        attrs.list(attrs.source(),
                   default=[],
                   doc="All source files that are part of the compilation"),
        "out":
        attrs.string(doc="The name of the output binary"),
        "deno_cache":
        attrs.option(attrs.dep(providers=[DenoWorkspaceInfo]), default=None),
        "permissions":
        attrs.list(
            attrs.string(),
            default=[],
            doc="List of Deno permissions to grant (e.g., read, write, net)"),
        "unstable_flags":
        attrs.list(attrs.string(),
                   default=[],
                   doc="List of unstable flags to enable"),
        "_python_toolchain":
        attrs.toolchain_dep(default="toolchains//:python",
                            providers=[PythonToolchainInfo]),
        "_deno_toolchain":
        attrs.toolchain_dep(default="toolchains//:deno",
                            providers=[DenoToolchainInfo]),
    },
)


def deno_test_impl(ctx: AnalysisContext) -> list[Provider]:
    """Implementation of the deno_test rule."""
    deno_toolchain = ctx.attrs._deno_toolchain[DenoToolchainInfo]

    shim = cmd_script(
        ctx=ctx,
        name="deno_shim",
        cmd=cmd_args(deno_toolchain.deno_binary),
    )

    cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        deno_toolchain.deno_test[DefaultInfo].default_outputs[0],
        "--deno-binary",
        deno_toolchain.deno_binary,
    )

    for src in ctx.attrs.srcs:
        cmd.add("--input", src)

    if ctx.attrs.filter:
        cmd.add("--filter", ctx.attrs.filter)

    if ctx.attrs.ignore:
        for ignore_path in ctx.attrs.ignore:
            cmd.add("--ignore", ignore_path)

    if ctx.attrs.parallel:
        cmd.add("--parallel")

    if ctx.attrs.watch:
        cmd.add("--watch")

    if ctx.attrs.permissions:
        cmd.add("--permissions")
        cmd.add(ctx.attrs.permissions)

    if ctx.attrs.unstable_flags:
        cmd.add("--unstable-flags")
        cmd.add(ctx.attrs.unstable_flags)

    return [
        DefaultInfo(),
        RunInfo(args=cmd),
    ]


deno_test = rule(
    impl=deno_test_impl,
    attrs={
        "srcs":
        attrs.list(
            attrs.source(),
            default=[],
            doc="The test files to run",
        ),
        "filter":
        attrs.option(
            attrs.string(),
            default=None,
            doc="Run tests with this string or pattern in the test name",
        ),
        "ignore":
        attrs.list(
            attrs.string(),
            default=[],
            doc="List of files or directories to ignore",
        ),
        "parallel":
        attrs.bool(
            default=True,
            doc="Run tests in parallel",
        ),
        "permissions":
        attrs.list(
            attrs.string(),
            default=[],
            doc="List of Deno permissions to grant (e.g., read, write, net)",
        ),
        "unstable_flags":
        attrs.list(
            attrs.string(),
            default=[],
            doc="List of unstable flags to enable",
        ),
        "watch":
        attrs.bool(
            default=False,
            doc="Watch for file changes and restart tests",
        ),
        "_python_toolchain":
        attrs.toolchain_dep(
            default="toolchains//:python",
            providers=[PythonToolchainInfo],
        ),
        "_deno_toolchain":
        attrs.toolchain_dep(
            default="toolchains//:deno",
            providers=[DenoToolchainInfo],
        ),
    },
)


def _deno_workspace_impl(ctx: AnalysisContext) -> list[Provider]:
    deno_toolchain = ctx.attrs._deno_toolchain[DenoToolchainInfo]

    workspace_out = ctx.actions.declare_output(ctx.attrs.workspace_out_name,
                                               dir=True)
    cache_out = ctx.actions.declare_output(ctx.attrs.cache_out_name, dir=True)

    cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        deno_toolchain.deno_workspace[DefaultInfo].default_outputs[0],
    )

    cmd.add("--root-config",
            ctx.attrs.root_config[DefaultInfo].default_outputs[0])
    cmd.add("--workspace-dir", workspace_out.as_output())
    cmd.add("--deno-binary", deno_toolchain.deno_binary)
    cmd.add("--deno-dir", cache_out.as_output())

    for src in ctx.attrs.srcs:
        cmd.add("--src", src)

    ctx.actions.run(
        cmd,
        category="deno_workspace",
    )

    return [
        DefaultInfo(default_output=workspace_out),
        DenoWorkspaceInfo(
            workspace_dir=workspace_out,
            cache_dir=cache_out,
        ),
    ]


deno_workspace = rule(
    impl=_deno_workspace_impl,
    attrs={
        "root_config":
        attrs.dep(doc="Root deno.json configuration file", ),
        "srcs":
        attrs.list(
            attrs.source(),
            doc="All source files and deno.json files for the workspace.",
        ),
        "workspace_out_name":
        attrs.string(
            default="workspace",
            doc="The name of the output directory for the workspace structure.",
        ),
        "cache_out_name":
        attrs.string(
            default="deno_cache",
            doc="The name of the output directory for the Deno cache.",
        ),
        "_python_toolchain":
        attrs.toolchain_dep(
            default="toolchains//:python",
            providers=[PythonToolchainInfo],
        ),
        "_deno_toolchain":
        attrs.toolchain_dep(
            default="toolchains//:deno",
            providers=[DenoToolchainInfo],
        ),
    },
)
