load(
    "@prelude-si//:build_metadata.bzl",
    _deno_git_metadata_typescript = "deno_git_metadata_typescript",
)
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
        deno_json = "deno.json",
        deno_lock = "deno.lock",
        git_metadata = True,
        visibility = ["PUBLIC"],
        **kwargs):
    # If Git metadata is enabled, generate and add to extra_srcs
    if git_metadata:
        git_metadata_target = "{}-git-metadata-typescript".format(name)
        _deno_git_metadata_typescript(
            name = git_metadata_target,
            git_metadata = "prelude-si//build_metadata:git",
            visibility = visibility,
        )

        # Add generated file to extra_srcs dict with src/ destination
        extra_srcs = kwargs.get("extra_srcs", {})
        extra_srcs["src/git_metadata.ts"] = ":{}-git-metadata-typescript".format(name)
        kwargs["extra_srcs"] = extra_srcs

    _deno_binary(
        name = name,
        main = main,
        deno_json = deno_json,
        deno_lock = deno_lock,
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
