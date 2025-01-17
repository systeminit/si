DenoToolchainInfo = provider(fields = {
    "deno_compile": provider_field(typing.Any, default = None),
    "deno_format": provider_field(typing.Any, default = None),
    "deno_test": provider_field(typing.Any, default = None),
})

def deno_toolchain_impl(ctx) -> list[[DefaultInfo, DenoToolchainInfo]]:
    """
    A deno toolchain.
    """
    return [
        DefaultInfo(default_outputs = []),
        DenoToolchainInfo(
            deno_compile = ctx.attrs._deno_compile,
            deno_format = ctx.attrs._deno_format,
            deno_test = ctx.attrs._deno_test,
        ),
    ]

deno_toolchain = rule(
    impl = deno_toolchain_impl,
    attrs = {
        "_deno_compile": attrs.dep(
            default = "prelude-si//deno:deno_compile.py",
            providers = [DefaultInfo],
        ),
       "_deno_format": attrs.dep(
            default = "prelude-si//deno:deno_format.py",
            providers = [DefaultInfo],
        ),
       "_deno_test": attrs.dep(
            default = "prelude-si//deno:deno_test.py",
            providers = [DefaultInfo],
        ),
    },
    is_toolchain_rule = True,
)
