MiseToolchainInfo = provider(fields = {
    "mise_install": typing.Any,
})

def mise_toolchain_impl(ctx) -> list[[DefaultInfo, MiseToolchainInfo]]:
    """
    A Mise toolchain.
    """
    return [
        DefaultInfo(),
        MiseToolchainInfo(
            mise_install = ctx.attrs._mise_install,
        ),
    ]

mise_toolchain = rule(
    impl = mise_toolchain_impl,
    attrs = {
        "_mise_install": attrs.dep(
            default = "prelude-si//mise:mise_install.py",
        ),
    },
    is_toolchain_rule = True,
)
