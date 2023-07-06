SiRustToolchainInfo = provider(
    fields = [
        "clippy_output",
        "crate_context",
        "rustfmt_check",
        "rustfmt_toml",
    ],
)

def si_rust_toolchain_impl(ctx) -> [[DefaultInfo.type, SiRustToolchainInfo.type]]:
    """
    An extended Rust toolchain.
    """
    if ctx.attrs.rustfmt_toml:
        rustfmt_toml = ctx.attrs.rustfmt_toml[DefaultInfo].default_outputs[0]
    else:
        rustfmt_toml = None

    return [
        DefaultInfo(),
        SiRustToolchainInfo(
            clippy_output = ctx.attrs._clippy_output,
            crate_context = ctx.attrs._crate_context,
            rustfmt_check = ctx.attrs._rustfmt_check,
            rustfmt_toml = rustfmt_toml,
        )
    ]

si_rust_toolchain = rule(
    impl = si_rust_toolchain_impl,
    attrs = {
        "rustfmt_toml": attrs.option(
            attrs.dep(providers = [DefaultInfo]),
            default = None,
        ),
        "_clippy_output": attrs.dep(
            default = "prelude-si//rust:clippy_output.py",
        ),
        "_crate_context": attrs.dep(
            default = "prelude-si//rust:crate_context.py",
        ),
        "_rustfmt_check": attrs.dep(
            default = "prelude-si//rust:rustfmt_check.py",
        ),
    },
    is_toolchain_rule = True,
)
