load(
    "@prelude-si//:toml.bzl",
    _toml_format = "toml_format",
    _toml_format_check = "toml_format_check",
)

def toml_format(
        name,
        visibility = ["PUBLIC"],
        **kwargs):
    _toml_format(
        name = name,
        visibility = visibility,
        **kwargs
    )

def toml_format_check(
        name,
        visibility = ["PUBLIC"],
        **kwargs):
    _toml_format_check(
        name = name,
        visibility = visibility,
        **kwargs
    )
