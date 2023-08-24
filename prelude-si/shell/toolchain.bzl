ShellToolchainInfo = provider(
    fields = [
        "build_context",
        "editorconfig",
        "shellcheck",
        "shfmt_check",
    ],
)

def shell_toolchain_impl(ctx) -> list[[DefaultInfo, ShellToolchainInfo]]:
    """
    A shell toolchain.
    """
    if ctx.attrs.editorconfig:
        editorconfig = ctx.attrs.editorconfig[DefaultInfo].default_outputs[0]
    else:
        editorconfig = None

    return [
        DefaultInfo(),
        ShellToolchainInfo(
            build_context = ctx.attrs._build_context,
            editorconfig = editorconfig,
            shellcheck = ctx.attrs._shellcheck,
            shfmt_check = ctx.attrs._shfmt_check,
        )
    ]

shell_toolchain = rule(
    impl = shell_toolchain_impl,
    attrs = {
        "editorconfig": attrs.option(
            attrs.dep(providers = [DefaultInfo]),
            default = None,
        ),
        "_build_context": attrs.dep(
            default = "prelude-si//shell:build_context.py",
        ),
        "_shellcheck": attrs.dep(
            default = "prelude-si//shell:shellcheck.py",
        ),
        "_shfmt_check": attrs.dep(
            default = "prelude-si//shell:shfmt_check.py",
        ),
    },
    is_toolchain_rule = True,
)
