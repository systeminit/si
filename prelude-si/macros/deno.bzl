load(
    "@prelude-si//:deno.bzl",
    _deno_binary = "deno_binary",
    _deno_format = "deno_format",
    _deno_test = "deno_test",
)
load(
    "@prelude-si//macros:native.bzl",
    _alias = "alias",
    _test_suite = "test_suite",
)

def deno_binary(
        name,
        main = "main.ts",
        visibility = ["PUBLIC"],
        **kwargs):
    _deno_binary(
        name = name,
        main = main,
        out = kwargs.get("out", name),
        visibility = visibility,
        **kwargs
    )

    if not rule_exists("build"):
        _alias(
            name = "build",
            actual = ":{}".format(name),
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
