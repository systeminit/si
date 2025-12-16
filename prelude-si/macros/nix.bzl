load(
    "@prelude-si//:artifact.bzl",
    _VALID_LINUX_PLATFORM_TARGETS = "VALID_LINUX_PLATFORM_TARGETS",
    _artifact_promote = "artifact_promote",
    _artifact_publish = "artifact_publish",
    _validate_linux_platform_targets = "validate_linux_platform_targets",
)
load(
    "@prelude-si//:nix.bzl",
    _nix_binary = "nix_binary",
    _nix_flake_lock = "nix_flake_lock",
    _nix_omnibus_pkg = "nix_omnibus_pkg",
)
load(
    "@prelude-si//macros:native.bzl",
    _alias = "alias",
)

def nix_flake_lock(
        name,
        src = None,
        nix_flake = ":flake.nix",
        visibility = ["PUBLIC"],
        **kwargs):
    _nix_flake_lock(
        name = name,
        src = src or name,
        nix_flake = nix_flake,
        visibility = visibility,
        **kwargs
    )

def nix_binary(
        name,
        binary_name,
        source_url = "http://github.com/systeminit/si.git",
        author = "The System Initiative <dev@systeminit.com>",
        license = "Apache-2.0",
        visibility = ["PUBLIC"],
        publish_target = "publish-binary",
        promote_target = "promote-binary",
        artifact_destination = "s3://si-artifacts-prod",
        artifact_cname = "artifacts.systeminit.com",
        **kwargs):
    _nix_binary(
        name = name,
        binary_name = binary_name,
        source_url = source_url,
        author = author,
        license = license,
        visibility = visibility,
        **kwargs
    )

    _artifact_publish(
        name = publish_target,
        artifact = ":{}".format(name),
        destination = artifact_destination,
        cname = artifact_cname,
        visibility = visibility,
    )

    _artifact_promote(
        name = promote_target,
        family = binary_name,
        variant = "binary",
        destination = artifact_destination,
        cname = artifact_cname,
        visibility = visibility,
    )

def nix_omnibus_pkg(
        name,
        pkg_name = None,
        source_url = "http://github.com/systeminit/si.git",
        author = "The System Initiative <dev@systeminit.com>",
        license = "Apache-2.0",
        artifact_destination = "s3://si-artifacts-prod",
        artifact_cname = "artifacts.systeminit.com",
        platform_targets = _VALID_LINUX_PLATFORM_TARGETS,
        skip_all_publish = False,
        skip_all_promote = False,
        default_target = False,
        visibility = ["PUBLIC"],
        **kwargs):
    """Create Nix omnibus package artifact with publish/promote targets.

    Creates targets:
    - :{name}-nix-omnibus-pkg
    - :publish-{name}-nix-omnibus-pkg
    - :promote-{name}-nix-omnibus-pkg

    Args:
        name: Package name
        pkg_name: Nix package name
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

    # Default pkg_name
    if pkg_name == None:
        pkg_name = name

    # Validate platform_targets
    _validate_linux_platform_targets(platform_targets, "nix_omnibus_pkg({})".format(name))

    # Base artifact target
    nix_omnibus_pkg_name = "{}-nix-omnibus-pkg".format(name)
    _nix_omnibus_pkg(
        name = nix_omnibus_pkg_name,
        pkg_name = pkg_name,
        source_url = source_url,
        author = author,
        license = license,
        visibility = visibility,
        **kwargs
    )

    # Base publish target
    _artifact_publish(
        name = "publish-{}-nix-omnibus-pkg".format(name),
        artifact = ":{}".format(nix_omnibus_pkg_name),
        destination = artifact_destination,
        cname = artifact_cname,
        platform_targets = platform_targets,
        skip_all = skip_all_publish,
        visibility = visibility,
    )

    # Base promote target
    _artifact_promote(
        name = "promote-{}-omnibus".format(name),
        family = pkg_name,
        variant = "omnibus",
        destination = artifact_destination,
        cname = artifact_cname,
        platform_targets = platform_targets,
        skip_all = skip_all_promote,
        visibility = visibility,
    )

    if default_target:
        _alias(
            name = name,
            actual = ":{}".format(nix_omnibus_pkg_name),
            visibility = visibility,
        )
