ArtifactToolchainInfo = provider(fields = {
    "promote": typing.Any,
    "publish": typing.Any,
    "generate_build_metadata": typing.Any,
    "create_binary_archive": typing.Any,
})

def artifact_toolchain_impl(ctx) -> list[[DefaultInfo, ArtifactToolchainInfo]]:
    """
    A artifact toolchain to manage a compiled or built artifact through it's lifecycle
    This toolchain will empower targets to publish, deprecate, etc.
    """
    return [
        DefaultInfo(),
        ArtifactToolchainInfo(
            promote = ctx.attrs._promote,
            publish = ctx.attrs._publish,
            generate_build_metadata = ctx.attrs._generate_build_metadata,
            create_binary_archive = ctx.attrs._create_binary_archive,
        ),
    ]

artifact_toolchain = rule(
    impl = artifact_toolchain_impl,
    attrs = {
        "_promote": attrs.dep(
            default = "prelude-si//artifact:promote.py",
        ),
        "_publish": attrs.dep(
            default = "prelude-si//artifact:publish.py",
        ),
        "_generate_build_metadata": attrs.dep(
            default = "prelude-si//artifact:generate_build_metadata.py",
        ),
        "_create_binary_archive": attrs.dep(
            default = "prelude-si//artifact:create_binary_archive.py",
        ),
    },
    is_toolchain_rule = True,
)
