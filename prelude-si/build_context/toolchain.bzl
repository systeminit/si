BuildContextToolchainInfo = provider(fields = {
    "build_context": typing.Any,
    "build_context_srcs_from_deps": typing.Any,
})

def build_context_toolchain_impl(ctx) -> list[[DefaultInfo, BuildContextToolchainInfo]]:
    """
    A BuildContext toolchain.
    """
    return [
        DefaultInfo(),
        BuildContextToolchainInfo(
            build_context = ctx.attrs._build_context,
            build_context_srcs_from_deps = ctx.attrs._build_context_srcs_from_deps,
        ),
    ]

build_context_toolchain = rule(
    impl = build_context_toolchain_impl,
    attrs = {
        "_build_context": attrs.dep(
            default = "prelude-si//build_context:build_context.py",
        ),
        "_build_context_srcs_from_deps": attrs.dep(
            default = "prelude-si//build_context:build_context_srcs_from_deps.bxl",
        ),
    },
    is_toolchain_rule = True,
)
