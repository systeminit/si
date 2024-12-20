DenoToolchainInfo = provider(fields = {
    "build_deno_bin": provider_field(typing.Any, default = None),
    "format_deno": provider_field(typing.Any, default = None),
})

def deno_toolchain_impl(ctx) -> list[[DefaultInfo, DenoToolchainInfo]]:
    """
    A deno toolchain.
    """
    return [
        DefaultInfo(default_outputs = []),
        DenoToolchainInfo(
            build_deno_bin = ctx.attrs._build_deno_bin,
            format_deno = ctx.attrs._format_deno,
        ),
    ]

deno_toolchain = rule(
    impl = deno_toolchain_impl,
    attrs = {
        "_build_deno_bin": attrs.dep(
            default = "prelude-si//deno:build_deno_bin.py",
            providers = [DefaultInfo],
        ),
       "_format_deno": attrs.dep(
            default = "prelude-si//deno:deno_format.py",
            providers = [DefaultInfo],
        ),
    },
    is_toolchain_rule = True,
)
