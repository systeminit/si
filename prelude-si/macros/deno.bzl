load(
    "@prelude-si//:artifact.bzl",
    _VALID_PLATFORM_TARGETS = "VALID_PLATFORM_TARGETS",
    _artifact_promote = "artifact_promote",
    _artifact_publish = "artifact_publish",
    _validate_platform_targets = "validate_platform_targets",
)
load(
    "@prelude-si//:build_metadata.bzl",
    _deno_git_metadata_typescript = "deno_git_metadata_typescript",
)
load(
    "@prelude-si//:deno.bzl",
    _deno_binary = "deno_binary",
    _deno_binary_artifact = "deno_binary_artifact",
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
        platform_targets = _VALID_PLATFORM_TARGETS,
        visibility = ["PUBLIC"],
        **kwargs):
    # Validate all platform targets
    _validate_platform_targets(platform_targets, "deno_binary_artifact({})".format(name))

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

    for target in platform_targets:
        # Convenience alias for cross-compilation
        _alias(
            name = "{}-{}".format(name, target),
            actual = ":{}".format(name),
            default_target_platform = "prelude-si//platforms:{}".format(target),
            visibility = visibility,
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

def deno_binary_artifact(
        name,
        binary,
        source_url = "http://github.com/systeminit/si.git",
        author = "The System Initiative <dev@systeminit.com>",
        license = "Apache-2.0",
        artifact_destination = "s3://si-artifacts-prod",
        artifact_cname = "artifacts.systeminit.com",
        platform_targets = _VALID_PLATFORM_TARGETS,
        skip_all_publish = False,
        skip_all_promote = False,
        visibility = ["PUBLIC"]):
    """Create Deno binary artifact with publish/promote targets for all platforms.

    Creates base targets:
    - :{name}-binary-artifact
    - :publish-{name}-binary-artifact
    - :promote-{name}-binary-artifact

    Plus platform-specific aliases for each:
    - :{name}-binary-artifact-{platform}
    - :publish-{name}-binary-artifact-{platform}

    Args:
        name: Binary name
        binary: The deno_binary target
        source_url: Source code URL for metadata
        author: Author for metadata
        license: License string for metadata
        artifact_destination: S3 destination for artifacts
        artifact_cname: Canonical hostname for artifact URLs
        platform_targets: List of target platforms. Defaults to all supported Deno platforms.
        skip_all_publish: Skip publishing this artifact (default: False)
        skip_all_promote: Skip promoting this artifact (default: False)
        visibility: Target visibility
    """

    # Validate all platform targets
    _validate_platform_targets(platform_targets, "deno_binary_artifact({})".format(name))

    # Base artifact target
    _deno_binary_artifact(
        name = "{}-binary-artifact".format(name),
        binary = binary,
        binary_name = name,
        family = name,
        variant = "binary",
        author = author,
        source_url = source_url,
        license = license,
        platform_targets = platform_targets,
        visibility = visibility,
    )

    # Base publish target
    _artifact_publish(
        name = "publish-{}-binary-artifact".format(name),
        artifact = ":{}-binary-artifact".format(name),
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

    for platform in platform_targets:
        # Artifact alias
        _alias(
            name = "{}-binary-artifact-{}".format(name, platform),
            actual = ":{}-binary-artifact".format(name),
            default_target_platform = "prelude-si//platforms:{}".format(platform),
            visibility = visibility,
        )

        # Publish alias
        _alias(
            name = "publish-{}-binary-artifact-{}".format(name, platform),
            actual = ":publish-{}-binary-artifact".format(name),
            default_target_platform = "prelude-si//platforms:{}".format(platform),
            visibility = visibility,
        )
