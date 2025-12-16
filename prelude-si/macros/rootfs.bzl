load(
    "@prelude-si//:artifact.bzl",
    _VALID_LINUX_PLATFORM_TARGETS = "VALID_LINUX_PLATFORM_TARGETS",
    _artifact_promote = "artifact_promote",
    _artifact_publish = "artifact_publish",
    _validate_linux_platform_targets = "validate_linux_platform_targets",
)
load(
    "@prelude-si//:rootfs.bzl",
    _rootfs_tarball = "rootfs_tarball",
)
load(
    "@prelude-si//macros:native.bzl",
    _alias = "alias",
)

def rootfs_tarball(
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
    """Create rootfs tarball artifact with publish/promote targets.

    Creates targets:
    - :{name}-rootfs-tarball
    - :publish-{name}-rootfs-tarball
    - :promote-{name}-rootfs-tarball

    Args:
        name: Rootfs name
        pkg_name: Package name (defaults to "{name}-rootfs")
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
        pkg_name = "{}-rootfs".format(name)

    # Validate platform_targets
    _validate_linux_platform_targets(platform_targets, "rootfs_tarball({})".format(name))

    # Base artifact target
    rootfs_tarball_name = "{}-rootfs-tarball".format(name)
    _rootfs_tarball(
        name = rootfs_tarball_name,
        rootfs_name = pkg_name,
        visibility = visibility,
        **kwargs
    )

    # Base publish target
    _artifact_publish(
        name = "publish-{}-rootfs-tarball".format(name),
        artifact = ":{}".format(rootfs_tarball_name),
        destination = artifact_destination,
        cname = artifact_cname,
        platform_targets = platform_targets,
        skip_all = skip_all_publish,
        visibility = visibility,
    )

    # Base promote target
    _artifact_promote(
        name = "promote-{}-rootfs-tarball".format(name),
        family = name,
        variant = "rootfs",
        destination = artifact_destination,
        cname = artifact_cname,
        platform_targets = platform_targets,
        skip_all = skip_all_promote,
        visibility = visibility,
    )

    if default_target:
        _alias(
            name = name,
            actual = ":{}".format(rootfs_tarball_name),
            visibility = visibility,
        )
