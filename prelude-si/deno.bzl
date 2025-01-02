load(
    "//deno:toolchain.bzl",
    "DenoToolchainInfo",
)
load(
    "@prelude//:paths.bzl",
    "paths",
)
load(
    "@prelude//python:toolchain.bzl",
    "PythonToolchainInfo",
)

def deno_compile_impl(ctx: AnalysisContext) -> list[Provider]:
    out = ctx.actions.declare_output(paths.basename(ctx.attrs.out))

    deno_toolchain = ctx.attrs._deno_toolchain[DenoToolchainInfo]

    cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        deno_toolchain.deno_compile[DefaultInfo].default_outputs[0],
        "--input",
        ctx.attrs.main,
        "--output",
        out.as_output(),
    )

    cmd.hidden(ctx.attrs.srcs)

    if ctx.attrs.permissions:
        cmd.add("--permissions")
        cmd.add(ctx.attrs.permissions)

    if ctx.attrs.unstable_flags:
        cmd.add("--unstable-flags")
        cmd.add(ctx.attrs.unstable_flags)

    ctx.actions.run(cmd, category = "deno", identifier = "deno_compile")

    return [
        DefaultInfo(default_output = out),
        RunInfo(args = cmd_args(out)),
    ]

deno_compile = rule(
    impl = deno_compile_impl,
    attrs = {
        "main": attrs.source(
            doc = "The entry point TypeScript/JavaScript file",
        ),
        "srcs": attrs.list(
            attrs.source(),
            default = [],
            doc = "All source files that are part of the compilation",
        ),
        "out": attrs.string(
            doc = "The name of the output binary",
        ),
        "permissions": attrs.list(
            attrs.string(),
            default = [],
            doc = "List of Deno permissions to grant (e.g., read, write, net)",
        ),
        "unstable_flags": attrs.list(
            attrs.string(),
            default = [],
            doc = "List of unstable flags to enable",
        ),
        "_python_toolchain": attrs.toolchain_dep(
            default = "toolchains//:python",
            providers = [PythonToolchainInfo],
        ),
        "_deno_toolchain": attrs.toolchain_dep(
            default = "toolchains//:deno",
            providers = [DenoToolchainInfo],
        ),
    },
)

def deno_format_impl(ctx: AnalysisContext) -> list[Provider]:
    """Implementation of the deno_format rule."""
    deno_toolchain = ctx.attrs._deno_toolchain[DenoToolchainInfo]

    cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        deno_toolchain.deno_format[DefaultInfo].default_outputs[0],
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
        RunInfo(args = cmd),
    ]

deno_format = rule(
    impl = deno_format_impl,
    attrs = {
        "srcs": attrs.list(
            attrs.source(),
            default = [],
            doc = "The source files to format",
        ),
        "check": attrs.bool(
            default = False,
            doc = "Check if files are formatted without making changes",
        ),
        "ignore": attrs.list(
            attrs.string(),
            default = [],
            doc = "List of files or directories to ignore",
        ),
        "_python_toolchain": attrs.toolchain_dep(
            default = "toolchains//:python",
            providers = [PythonToolchainInfo],
        ),
        "_deno_toolchain": attrs.toolchain_dep(
            default = "toolchains//:deno",
            providers = [DenoToolchainInfo],
        ),
    },
)

def deno_test_impl(ctx: AnalysisContext) -> list[Provider]:
    """Implementation of the deno_test rule."""
    deno_toolchain = ctx.attrs._deno_toolchain[DenoToolchainInfo]

    cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        deno_toolchain.deno_test[DefaultInfo].default_outputs[0],
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

    return [
        DefaultInfo(),
        RunInfo(args = cmd),
    ]

deno_test = rule(
    impl = deno_test_impl,
    attrs = {
        "srcs": attrs.list(
            attrs.source(),
            default = [],
            doc = "The test files to run",
        ),
        "filter": attrs.option(
            attrs.string(),
            default = None,
            doc = "Run tests with this string or pattern in the test name",
        ),
        "ignore": attrs.list(
            attrs.string(),
            default = [],
            doc = "List of files or directories to ignore",
        ),
        "parallel": attrs.bool(
            default = True,
            doc = "Run tests in parallel",
        ),
        "watch": attrs.bool(
            default = False,
            doc = "Watch for file changes and restart tests",
        ),
        "_python_toolchain": attrs.toolchain_dep(
            default = "toolchains//:python",
            providers = [PythonToolchainInfo],
        ),
        "_deno_toolchain": attrs.toolchain_dep(
            default = "toolchains//:deno",
            providers = [DenoToolchainInfo],
        ),
    },
)
