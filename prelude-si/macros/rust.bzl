load(
    "@prelude-si//:artifact.bzl",
    _VALID_LINUX_PLATFORM_TARGETS = "VALID_LINUX_PLATFORM_TARGETS",
    _artifact_promote = "artifact_promote",
    _artifact_publish = "artifact_publish",
    _validate_linux_platform_targets = "validate_linux_platform_targets",
)
load(
    "@prelude-si//:build_metadata.bzl",
    _rust_git_metadata_outdir = "rust_git_metadata_outdir",
)
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
    _rust_binary_artifact = "rust_binary_artifact",
    _rustfmt_check = "rustfmt_check",
)
load(
    "@prelude-si//:toml.bzl",
    _toml_format = "toml_format",
    _toml_format_check = "toml_format_check",
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
        edition = "2024",
        env = {},
        resources = [],
        test_unit_deps = [],
        test_unit_srcs = [],
        test_unit_env = {},
        test_unit_resources = {},
        extra_test_targets = [],
        toml_srcs = ["Cargo.toml"],
        git_metadata = True,
        visibility = ["PUBLIC"],
        **kwargs):
    base_env = {
        # Automatically set the bin name, as used by some crates that are only Cargo-aware
        "CARGO_BIN_NAME": name,
    }

    # If Git metadata is enabled, generate OUT_DIR with git metadata
    if git_metadata:
        git_metadata_target = "{}-git-metadata-outdir".format(name)
        _rust_git_metadata_outdir(
            name = git_metadata_target,
            git_metadata = "prelude-si//build_metadata:git",
            visibility = visibility,
        )

        # Automatically wire Git metadata OUT_DIR for version stamping
        base_env["OUT_DIR"] = "$(location :{})".format(git_metadata_target)

    native.rust_binary(
        name = name,
        edition = edition,
        srcs = srcs,
        deps = deps,
        crate_root = crate_root,
        resources = resources,
        env = base_env | env,
        visibility = visibility,
        **kwargs
    )

    if toml_srcs:
        _toml_format_check(
            name = "check-format-toml",
            srcs = toml_srcs,
            visibility = visibility,
        )

    _alias(
        name = "build",
        actual = ":{}".format(name),
    )

    if not rule_exists("test-unit"):
        native.rust_test(
            name = "test-unit",
            edition = edition,
            srcs = srcs + test_unit_srcs,
            deps = deps + test_unit_deps,
            crate_root = crate_root,
            resources = test_unit_resources,
            env = base_env | env | test_unit_env,
            visibility = visibility,
            **kwargs
        )

        _clippy_check(
            name = "check-lint-rust-unit",
            clippy_txt_dep = ":test-unit[clippy.txt]",
            visibility = visibility,
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

    check_lint_rust_targets = []
    if rule_exists("check-lint-rust-bin"):
        check_lint_rust_targets.append(":check-lint-rust-bin")
    if rule_exists("check-lint-rust-unit"):
        check_lint_rust_targets.append(":check-lint-rust-unit")

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
        tests = check_lint_rust_targets + extra_check_lint_targets,
        visibility = visibility,
    )

    if not rule_exists("check-lint"):
        _test_suite(
            name = "check-lint",
            tests = check_lint_rust_targets + extra_check_lint_targets,
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
        name = "fix-format-rust",
        crate = name,
        srcs = srcs,
        visibility = visibility,
    )

    if toml_srcs:
        _toml_format(
            name = "fix-format-toml",
            srcs = toml_srcs,
            visibility = visibility,
        )

    _cargo_clippy_fix(
        name = "fix-lint-rust",
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
        crate_root = "src/lib.rs",
        edition = "2024",
        env = {},
        resources = [],
        test_unit_deps = [],
        test_unit_srcs = [],
        test_unit_env = {},
        test_unit_resources = {},
        extra_test_targets = [],
        toml_srcs = ["Cargo.toml"],
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
        env = env,
        proc_macro = proc_macro,
        visibility = visibility,
        **kwargs
    )

    _alias(
        name = "build",
        actual = ":{}".format(name),
    )

    if toml_srcs:
        _toml_format_check(
            name = "check-format-toml",
            srcs = toml_srcs,
            visibility = visibility,
        )

    if not rule_exists("test-unit"):
        native.rust_test(
            name = "test-unit",
            edition = edition,
            srcs = srcs + test_unit_srcs,
            deps = deps + test_unit_deps,
            crate_root = crate_root,
            resources = test_unit_resources,
            env = env | test_unit_env,
            visibility = visibility,
            **kwargs
        )

        _clippy_check(
            name = "check-lint-rust-unit",
            clippy_txt_dep = ":test-unit[clippy.txt]",
            visibility = visibility,
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

    check_lint_rust_targets = []
    if rule_exists("check-lint-rust-lib"):
        check_lint_rust_targets.append(":check-lint-rust-lib")
    if rule_exists("check-lint-rust-unit"):
        check_lint_rust_targets.append(":check-lint-rust-unit")

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
        tests = check_lint_rust_targets + extra_check_lint_targets,
        visibility = visibility,
    )

    if not rule_exists("check-lint"):
        _test_suite(
            name = "check-lint",
            tests = check_lint_rust_targets + extra_check_lint_targets,
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
        name = "fix-format-rust",
        crate = name,
        srcs = srcs,
        visibility = visibility,
    )

    if toml_srcs:
        _toml_format(
            name = "fix-format-toml",
            srcs = toml_srcs,
            visibility = visibility,
        )

    _cargo_clippy_fix(
        name = "fix-lint-rust",
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
        edition = "2024",
        visibility = ["PUBLIC"],
        **kwargs):
    native.rust_test(
        name = name,
        edition = edition,
        visibility = visibility,
        **kwargs
    )

def rust_binary_artifact(
        name,
        binary,
        source_url = "http://github.com/systeminit/si.git",
        author = "The System Initiative <dev@systeminit.com>",
        license = "Apache-2.0",
        artifact_destination = "s3://si-artifacts-prod",
        artifact_cname = "artifacts.systeminit.com",
        platform_targets = _VALID_LINUX_PLATFORM_TARGETS,
        skip_all_publish = False,
        skip_all_promote = False,
        default_target = False,
        visibility = ["PUBLIC"]):
    """Create Rust binary artifact with publish/promote targets.

    Creates base targets:
    - :{name}-binary-artifact
    - :publish-{name}-binary-artifact
    - :promote-{name}-binary-artifact

    Args:
        name: Binary name
        binary: The rust_binary target
        source_url: Source code URL for metadata
        author: Author for metadata
        license: License string for metadata
        artifact_destination: S3 destination for artifacts
        artifact_cname: Canonical hostname for artifact URLs
        platform_targets: List of target platforms. Defaults to Linux native platforms.
        skip_all_publish: Skip publishing this artifact (default: False)
        skip_all_promote: Skip promoting this artifact (default: False)
        visibility: Target visibility
    """

    # Validate platform_targets
    _validate_linux_platform_targets(platform_targets, "rust_binary_artifact({})".format(name))

    # Base artifact target
    rust_binary_artifact_name = "{}-binary-artifact".format(name)
    _rust_binary_artifact(
        name = rust_binary_artifact_name,
        binary = binary,
        binary_name = name,
        family = name,
        author = author,
        source_url = source_url,
        license = license,
        platform_targets = platform_targets,
        visibility = visibility,
    )

    # Base publish target
    _artifact_publish(
        name = "publish-{}-binary-artifact".format(name),
        artifact = ":{}".format(rust_binary_artifact_name),
        destination = artifact_destination,
        cname = artifact_cname,
        platform_targets = platform_targets,
        skip_all = skip_all_publish,
        visibility = visibility,
    )

    # Base promote target
    _artifact_promote(
        name = "promote-{}-binary-artifact".format(name),
        family = name,
        variant = "binary",
        destination = artifact_destination,
        cname = artifact_cname,
        platform_targets = platform_targets,
        skip_all = skip_all_promote,
        visibility = visibility,
    )

    if default_target:
        _alias(
            name = name,
            actual = ":{}".format(rust_binary_artifact_name),
            visibility = visibility,
        )
