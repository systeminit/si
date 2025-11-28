BuildMetadataToolchainInfo = provider(fields = {
    "generate_git_metadata": typing.Any,
    "generate_git_metadata_rust": typing.Any,
    "generate_git_metadata_typescript": typing.Any,
})

def build_metadata_toolchain_impl(ctx) -> list[[DefaultInfo, BuildMetadataToolchainInfo]]:
    """
    A build metadata toolchain.
    """
    return [
        DefaultInfo(),
        BuildMetadataToolchainInfo(
            generate_git_metadata = ctx.attrs._generate_git_metadata,
            generate_git_metadata_rust = ctx.attrs._generate_git_metadata_rust,
            generate_git_metadata_typescript = ctx.attrs._generate_git_metadata_typescript,
        ),
    ]

build_metadata_toolchain = rule(
    impl = build_metadata_toolchain_impl,
    attrs = {
        "_generate_git_metadata": attrs.dep(
            default = "prelude-si//build_metadata:generate_git_metadata.py",
        ),
        "_generate_git_metadata_rust": attrs.dep(
            default = "prelude-si//build_metadata:generate_git_metadata_rust.py",
        ),
        "_generate_git_metadata_typescript": attrs.dep(
            default = "prelude-si//build_metadata:generate_git_metadata_typescript.py",
        ),
    },
    is_toolchain_rule = True,
)
