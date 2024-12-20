load(
    "@prelude-si//:deno.bzl",
    _deno_binary = "deno_binary",
    _deno_format = "deno_format",
)

def deno_binary(
        visibility = ["PUBLIC"],
        **kwargs):
    _deno_binary(
        visibility = visibility,
        **kwargs
    )

def deno_format(
        visibility = ["PUBLIC"],
        **kwargs):
    _deno_format(
        visibility = visibility,
        **kwargs
    )
