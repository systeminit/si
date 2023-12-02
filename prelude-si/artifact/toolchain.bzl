ArtifactToolchainInfo = provider(fields = {
    "publish": typing.Any,
})

def artifact_toolchain_impl(ctx) -> list[[DefaultInfo, ArtifactToolchainInfo]]:
    """
    A artifact toolchain to manage a compiled or built artifact through it's lifecycle
    This toolchain will empower targets to publish, deprecate, etc. 
    """
    return [
        DefaultInfo(),
        ArtifactToolchainInfo(
            publish = ctx.attrs._publish,
        ),
    ]

artifact_toolchain = rule(
    impl = artifact_toolchain_impl,
    attrs = {
        "_publish": attrs.dep(
            default = "prelude-si//artifact:publish.py",
        ),
    },
    is_toolchain_rule = True,
) 
