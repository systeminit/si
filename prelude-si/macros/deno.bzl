load(
    "@prelude-si//:deno.bzl",
    _deno_compile = "deno_compile",
    _deno_format = "deno_format",
    _deno_test = "deno_test",
)

def deno_compile(
        visibility = ["PUBLIC"],
        **kwargs):
    _deno_compile(
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

def deno_test(
        visibility = ["PUBLIC"],
        **kwargs):
    _deno_test(
        visibility = visibility,
        **kwargs
    )
