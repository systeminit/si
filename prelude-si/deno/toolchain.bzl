DenoToolchainInfo = provider(fields = {
    "deno_bin": provider_field(typing.Any, default = None),
    "deno_cache": provider_field(typing.Any, default = None),
    "deno_compile": provider_field(typing.Any, default = None),
    "deno_format": provider_field(typing.Any, default = None),
    "deno_run": provider_field(typing.Any, default = None),
    "deno_test": provider_field(typing.Any, default = None),
    "deno_workspace": provider_field(typing.Any, default = None),
})

def deno_toolchain_impl(ctx) -> list[[DefaultInfo, DenoToolchainInfo]]:
    """
    A deno toolchain.
    """
    return [
        DefaultInfo(default_outputs = []),
        DenoToolchainInfo(
            deno_bin = ctx.attrs.deno_bin,
            deno_cache = ctx.attrs._deno_cache,
            deno_compile = ctx.attrs._deno_compile,
            deno_format = ctx.attrs._deno_format,
            deno_run = ctx.attrs._deno_run,
            deno_test = ctx.attrs._deno_test,
            deno_workspace = ctx.attrs._deno_workspace,
        ),
    ]

deno_toolchain = rule(
    impl = deno_toolchain_impl,
    attrs = {
       "deno_bin": attrs.string(),
       "_deno_cache": attrs.dep(
            default = "prelude-si//deno:deno_cache.py",
            providers = [DefaultInfo],
        ),
        "_deno_compile": attrs.dep(
            default = "prelude-si//deno:deno_compile.py",
            providers = [DefaultInfo],
        ),
       "_deno_format": attrs.dep(
            default = "prelude-si//deno:deno_format.py",
            providers = [DefaultInfo],
        ),
       "_deno_run": attrs.dep(
            default = "prelude-si//deno:deno_run.py",
            providers = [DefaultInfo],
        ),
       "_deno_test": attrs.dep(
            default = "prelude-si//deno:deno_test.py",
            providers = [DefaultInfo],
        ),
       "_deno_workspace": attrs.dep(
            default = "prelude-si//deno:deno_workspace.py",
            providers = [DefaultInfo],
        ),
    },
    is_toolchain_rule = True,
)
