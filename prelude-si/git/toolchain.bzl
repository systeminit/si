GitToolchainInfo = provider(fields = {
    "git_info": typing.Any,
})

def git_toolchain_impl(ctx) -> list[[DefaultInfo, GitToolchainInfo]]:
    """
    A Git toolchain.
    """
    return [
        DefaultInfo(),
        GitToolchainInfo(
            git_info = ctx.attrs._git_info,
        ),
    ]

git_toolchain = rule(
    impl = git_toolchain_impl,
    attrs = {
        "_git_info": attrs.dep(
            default = "prelude-si//git:git_info.py",
        ),
    },
    is_toolchain_rule = True,
)
