load(
    "@prelude-si//:cargo.bzl",
    _cargo_clippy = "cargo_clippy",
    _cargo_clippy_fix = "cargo_clippy_fix",
    _cargo_doc = "cargo_doc",
    _cargo_doc_check = "cargo_doc_check",
    _cargo_fmt = "cargo_fmt",
    _cargo_fmt_check = "cargo_fmt_check",
)
load(
    "@prelude-si//macros:native.bzl",
    _alias = "alias",
)

def rust_binary(
        name,
        srcs,
        deps,
        edition = "2021",
        resources = [],
        test_unit_deps = [],
        test_unit_srcs = [],
        test_unit_resources = {},
        visibility = ["PUBLIC"],
        **kwargs):

    native.rust_binary(
        name = name,
        edition = edition,
        srcs = srcs,
        deps = deps,
        resources = resources,
        visibility = visibility,
        **kwargs
    )

    _alias(
        name = "build",
        actual = ":{}".format(name),
    )

    native.rust_test(
        name = "test-unit",
        edition = edition,
        srcs = srcs + test_unit_srcs,
        deps = deps + test_unit_deps,
        resources = test_unit_resources,
        visibility = visibility,
        **kwargs
    )

    _cargo_doc_check(
        name = "check-doc",
        crate = name,
        srcs = srcs,
        visibility = visibility,
    )

    _cargo_fmt_check(
        name = "check-format",
        crate = name,
        srcs = srcs,
        visibility = visibility,
    )

    _cargo_clippy(
        name = "check-lint",
        crate = name,
        srcs = srcs,
        visibility = visibility,
    )

    _cargo_fmt(
        name = "fix-format",
        crate = name,
        srcs = srcs,
        visibility = visibility,
    )

    _cargo_clippy_fix(
        name = "fix-lint",
        crate = name,
        srcs = srcs,
        visibility = visibility,
    )

    _cargo_doc(
        name = "doc",
        crate = name,
        srcs = srcs,
        visibility = visibility,
    )

def rust_library(
        name,
        srcs,
        deps,
        edition = "2021",
        resources = [],
        test_unit_deps = [],
        test_unit_srcs = [],
        test_unit_resources = {},
        proc_macro = False,
        visibility = ["PUBLIC"],
        **kwargs):

    native.rust_library(
        name = name,
        edition = edition,
        srcs = srcs,
        deps = deps,
        resources = resources,
        proc_macro = proc_macro,
        visibility = visibility,
        **kwargs
    )

    _alias(
        name = "build",
        actual = ":{}".format(name),
    )

    native.rust_test(
        name = "test-unit",
        edition = edition,
        srcs = srcs + test_unit_srcs,
        deps = deps + test_unit_deps,
        resources = test_unit_resources,
        visibility = visibility,
        **kwargs
    )

    _cargo_doc_check(
        name = "check-doc",
        crate = name,
        srcs = srcs,
        visibility = visibility,
    )

    _cargo_fmt_check(
        name = "check-format",
        crate = name,
        srcs = srcs,
        visibility = visibility,
    )

    _cargo_clippy(
        name = "check-lint",
        crate = name,
        srcs = srcs,
        visibility = visibility,
    )

    _cargo_fmt(
        name = "fix-format",
        crate = name,
        srcs = srcs,
        visibility = visibility,
    )

    _cargo_clippy_fix(
        name = "fix-lint",
        crate = name,
        srcs = srcs,
        visibility = visibility,
    )

    _cargo_doc(
        name = "doc",
        crate = name,
        srcs = srcs,
        visibility = visibility,
    )
