SiRustToolchainInfo = provider(
    fields = {
        "clippy_output": typing.Any,
        "crate_context": typing.Any,
        "rust_metadata": typing.Any,
        "rustfmt_check": typing.Any,
        "rustfmt_path": typing.Any,
        "rustfmt_toml": provider_field(typing.Any, default = None),
    },
)

def si_rust_toolchain_impl(ctx) -> list[[DefaultInfo, SiRustToolchainInfo]]:
    """
    An extended Rust toolchain.
    """
    if ctx.attrs.rustfmt_toml:
        rustfmt_toml = ctx.attrs.rustfmt_toml[DefaultInfo].default_outputs[0]
    else:
        rustfmt_toml = None

    # Build rustfmt path from rust distribution
    if ctx.attrs.rust_dist:
        rust_dist_dir = ctx.attrs.rust_dist[DefaultInfo].default_outputs[0]
        rustfmt_path = cmd_args(rust_dist_dir, "/bin/rustfmt", delimiter="")
    else:
        rustfmt_path = cmd_args("rustfmt")

    return [
        DefaultInfo(),
        SiRustToolchainInfo(
            clippy_output = ctx.attrs._clippy_output,
            crate_context = ctx.attrs._crate_context,
            rust_metadata = ctx.attrs._rust_metadata,
            rustfmt_check = ctx.attrs._rustfmt_check,
            rustfmt_path = rustfmt_path,
            rustfmt_toml = rustfmt_toml,
        ),
    ]

si_rust_toolchain = rule(
    impl = si_rust_toolchain_impl,
    attrs = {
        "rustfmt_toml": attrs.option(
            attrs.dep(providers = [DefaultInfo]),
            default = None,
        ),
        "rust_dist": attrs.option(
            attrs.exec_dep(providers = [DefaultInfo]),
            default = None,
        ),
        "_clippy_output": attrs.dep(
            default = "prelude-si//rust:clippy_output.py",
        ),
        "_crate_context": attrs.dep(
            default = "prelude-si//rust:crate_context.py",
        ),
        "_rust_metadata": attrs.dep(
            default = "prelude-si//rust:generate_rust_metadata.py",
        ),
        "_rustfmt_check": attrs.dep(
            default = "prelude-si//rust:rustfmt_check.py",
        ),
    },
    is_toolchain_rule = True,
)
