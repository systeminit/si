load(
    "@prelude-si//:cargo.bzl",
    _cargo_clippy_fix = "cargo_clippy_fix",
    _cargo_doc = "cargo_doc",
    _cargo_doc_check = "cargo_doc_check",
    _cargo_fmt = "cargo_fmt",
)
load(
    "@prelude-si//:rust.bzl",
    _clippy_check = "clippy_check",
    _rustfmt_check = "rustfmt_check",
)
load(
    "@prelude-si//macros:native.bzl",
    _alias = "alias",
    _test_suite = "test_suite",
)

def rust_binary(
        name,
        srcs,
        deps,
        crate_root = "src/main.rs",
        edition = "2021",
        resources = [],
        test_unit_deps = [],
        test_unit_srcs = [],
        test_unit_resources = {},
        extra_test_targets = [],
        visibility = ["PUBLIC"],
        **kwargs):

    native.rust_binary(
        name = name,
        edition = edition,
        srcs = srcs,
        deps = deps,
        crate_root = crate_root,
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
        crate_root = crate_root,
        resources = test_unit_resources,
        visibility = visibility,
        **kwargs
    )

    _test_suite(
        name = "test",
        tests = [":test-unit"] + extra_test_targets,
        visibility = visibility,
    )

    _cargo_doc_check(
        name = "check-doc",
        crate = name,
        srcs = srcs,
        visibility = visibility,
    )

    _rustfmt_check(
        name = "check-format-rust",
        srcs = srcs,
        crate_root = crate_root,
        visibility = visibility,
    )

    if not rule_exists("check-format"):
        _test_suite(
            name = "check-format",
            tests = [":check-format-rust"],
            visibility = visibility,
        )

    _clippy_check(
        name = "check-lint-rust-bin",
        clippy_txt_dep = ":{}[clippy.txt]".format(name),
        visibility = visibility,
    )

    _clippy_check(
        name = "check-lint-rust-unit",
        clippy_txt_dep = ":{}[clippy.txt]".format("test-unit"),
        visibility = visibility,
    )

    extra_check_lint_targets = []
    for extra_test_target in extra_test_targets:
        check_name = "check-lint-rust-{}".format(extra_test_target.replace("test-", ""))
        _clippy_check(
            name = check_name,
            clippy_txt_dep = "{}[clippy.txt]".format(extra_test_target),
            visibility = visibility,
        )
        extra_check_lint_targets.append(":{}".format(check_name))

    _test_suite(
        name = "check-lint-rust",
        tests = [
            ":check-lint-rust-bin",
            ":check-lint-rust-unit",
        ] + extra_check_lint_targets,
        visibility = visibility,
    )

    if not rule_exists("check-lint"):
        _test_suite(
            name = "check-lint",
            tests = [
                ":check-lint-rust-bin",
                ":check-lint-rust-unit",
            ] + extra_check_lint_targets,
            visibility = visibility,
        )

    _test_suite(
        name = "check",
        tests = [
            ":check-doc",
            ":check-format",
            ":check-lint",
        ],
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


def rust_library_integration_test(
        name,
        srcs,
        deps,
        crate_root = "src/lib.rs",
        edition = "2021",
        resources = [],
        test_unit_deps = [],
        test_unit_srcs = [],
        test_unit_resources = {},
        extra_test_targets = [],
        proc_macro = False,
        visibility = ["PUBLIC"],
        **kwargs):

    native.rust_library(
        name = name,
        edition = edition,
        srcs = srcs,
        deps = deps,
        crate_root = crate_root,
        resources = resources,
        proc_macro = proc_macro,
        visibility = visibility,
        **kwargs
    )

    _alias(
        name = "integration_test",
        actual = ":{}".format(name),
    )

def rust_library(
        name,
        srcs,
        deps,
        crate_root = "src/lib.rs",
        edition = "2021",
        resources = [],
        test_unit_deps = [],
        test_unit_srcs = [],
        test_unit_resources = {},
        extra_test_targets = [],
        proc_macro = False,
        visibility = ["PUBLIC"],
        **kwargs):

    native.rust_library(
        name = name,
        edition = edition,
        srcs = srcs,
        deps = deps,
        crate_root = crate_root,
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
        crate_root = crate_root,
        resources = test_unit_resources,
        visibility = visibility,
        **kwargs
    )

    _test_suite(
        name = "test",
        tests = [":test-unit"] + extra_test_targets,
        visibility = visibility,
    )

    _cargo_doc_check(
        name = "check-doc",
        crate = name,
        srcs = srcs,
        visibility = visibility,
    )

    _rustfmt_check(
        name = "check-format-rust",
        srcs = srcs,
        crate_root = crate_root,
        visibility = visibility,
    )

    if not rule_exists("check-format"):
        _test_suite(
            name = "check-format",
            tests = [":check-format-rust"],
            visibility = visibility,
        )

    _clippy_check(
        name = "check-lint-rust-lib",
        clippy_txt_dep = ":{}[clippy.txt]".format(name),
        visibility = visibility,
    )

    _clippy_check(
        name = "check-lint-rust-unit",
        clippy_txt_dep = ":{}[clippy.txt]".format("test-unit"),
        visibility = visibility,
    )

    extra_check_lint_targets = []
    for extra_test_target in extra_test_targets:
        check_name = "check-lint-rust-{}".format(extra_test_target.replace(":", "").replace("test-", ""))
        _clippy_check(
            name = check_name,
            clippy_txt_dep = "{}[clippy.txt]".format(extra_test_target),
            visibility = visibility,
        )
        extra_check_lint_targets.append(":{}".format(check_name))

    _test_suite(
        name = "check-lint-rust",
        tests = [
            ":check-lint-rust-lib",
            ":check-lint-rust-unit",
        ] + extra_check_lint_targets,
        visibility = visibility,
    )

    if not rule_exists("check-lint"):
        _test_suite(
            name = "check-lint",
            tests = [
                ":check-lint-rust-lib",
                ":check-lint-rust-unit",
            ] + extra_check_lint_targets,
            visibility = visibility,
        )

    _test_suite(
        name = "check",
        tests = [
            ":check-doc",
            ":check-format",
            ":check-lint",
        ],
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

def rust_test(
        name,
        edition = "2021",
        visibility = ["PUBLIC"],
        **kwargs):

    native.rust_test(
        name = name,
        edition = edition,
        visibility = visibility,
        **kwargs
    )
