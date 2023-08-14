load(
    "@prelude-si//:shell.bzl",
    _shellcheck = "shellcheck",
    _shfmt_check = "shfmt_check",
)

def shellcheck(
        name,
        visibility = ["PUBLIC"],
        **kwargs):

    _shellcheck(
        name = name,
        visibility = visibility,
        **kwargs
    )

def shfmt_check(
        name,
        visibility = ["PUBLIC"],
        **kwargs):

    _shfmt_check(
        name = name,
        visibility = visibility,
        **kwargs
    )
