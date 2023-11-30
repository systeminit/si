SiPythonToolchainInfo = provider(
    fields = {
        "yapf_check": typing.Any,
    },
)

def si_python_toolchain_impl(ctx) -> list[[DefaultInfo, SiPythonToolchainInfo]]:
    """
    A extended Python toolchain.
    """

    return [
        DefaultInfo(),
        SiPythonToolchainInfo(
            yapf_check = ctx.attrs._yapf_check,
        ),
    ]

si_python_toolchain = rule(
    impl = si_python_toolchain_impl,
    attrs = {
        "_yapf_check": attrs.dep(
            default = "prelude-si//python:yapf_check.py",
        ),
    },
    is_toolchain_rule = True,
)
