RootfsToolchainInfo = provider(fields = {
    "rootfs_build": typing.Any,
    #"rootfs_promote": typing.Any,
})

def rootfs_toolchain_impl(ctx) -> list[[DefaultInfo, RootfsToolchainInfo]]:
    """
    A rootfs toolchain.
    """
    return [
        DefaultInfo(),
        RootfsToolchainInfo(
            rootfs_build = ctx.attrs._rootfs_build,
            #rootfs_promote = ctx.attrs._rootfs_promote,
        ),
    ]

rootfs_toolchain = rule(
    impl = rootfs_toolchain_impl,
    attrs = {
        "_rootfs_build": attrs.dep(
            default = "prelude-si//rootfs:rootfs_build.sh",
        ),
        #"_rootfs_promote": attrs.dep(
        #    default = "prelude-si//rootfs:rootfs_promote.sh",
        #),
    },
    is_toolchain_rule = True,
) 