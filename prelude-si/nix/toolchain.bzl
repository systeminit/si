NixToolchainInfo = provider(fields = {
    "nix_omnibus_pkg_build": typing.Any,
})

def nix_toolchain_impl(ctx) -> list[[DefaultInfo, NixToolchainInfo]]:
    """
    A Nix toolchain.
    """
    return [
        DefaultInfo(),
        NixToolchainInfo(
            nix_omnibus_pkg_build = ctx.attrs._nix_omnibus_pkg_build,
        ),
    ]

nix_toolchain = rule(
    impl = nix_toolchain_impl,
    attrs = {
        "_nix_omnibus_pkg_build": attrs.dep(
            default = "prelude-si//nix:nix_omnibus_pkg_build.py",
        ),
    },
    is_toolchain_rule = True,
)
