load(
    "@prelude-si//:python.bzl",
    _yapf_check = "yapf_check",
)

def yapf_check(
        name,
        visibility = ["PUBLIC"],
        **kwargs):
    _yapf_check(
        name = name,
        visibility = visibility,
        **kwargs
    )
