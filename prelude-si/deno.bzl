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

def deno_binary_impl(ctx: AnalysisContext) -> list[Provider]:
    out = ctx.actions.declare_output(paths.basename(ctx.attrs.out))

    deno_toolchain = ctx.attrs._deno_toolchain[DenoToolchainInfo]

    cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        deno_toolchain.build_deno_bin[DefaultInfo].default_outputs[0],
        "--input",
        ctx.attrs.main,
        "--output",
        out.as_output(),
    )

    if ctx.attrs.permissions:
        cmd.add("--permissions")
        cmd.add(ctx.attrs.permissions)

    if ctx.attrs.unstable_flags:
        cmd.add("--unstable-flags")
        cmd.add(ctx.attrs.unstable_flags)

    ctx.actions.run(cmd, category = "deno", identifier = "deno_binary")

    return [
        DefaultInfo(default_output = out),
        RunInfo(args = cmd_args(out)),
    ]

deno_binary = rule(
    impl = deno_binary_impl,
    attrs = {
        "main": attrs.source(
            doc = "The entry point TypeScript/JavaScript file",
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
