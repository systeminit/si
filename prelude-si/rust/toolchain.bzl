RustClippyToolchainInfo = provider(
    fields = [
        "clippy_output",
    ],
)

def rust_clippy_toolchain_impl(ctx) -> [[DefaultInfo.type, RustClippyToolchainInfo.type]]:
    """
    A toolchain for Clippy.
    """
    return [
        DefaultInfo(),
        RustClippyToolchainInfo(
            clippy_output = ctx.attrs._clippy_output,
        )
    ]

rust_clippy_toolchain = rule(
    impl = rust_clippy_toolchain_impl,
    attrs = {
        "_clippy_output": attrs.dep(
            default = "prelude-si//rust:clippy_output.py",
        ),
    },
    is_toolchain_rule = True,
)
