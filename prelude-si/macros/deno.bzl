load(
    "@prelude-si//:deno.bzl",
    _deno_binary = "deno_binary",
)

def deno_binary(
        visibility = ["PUBLIC"],
        **kwargs):
    _deno_binary(
        visibility = visibility,
        **kwargs
    )
