load(
    "@prelude-si//:artifact.bzl",
    _VALID_LINUX_PLATFORM_TARGETS = "VALID_LINUX_PLATFORM_TARGETS",
    _artifact_promote = "artifact_promote",
    _artifact_publish = "artifact_publish",
    _validate_linux_platform_targets = "validate_linux_platform_targets",
)
load(
    "@prelude-si//:docker.bzl",
    _container_image = "container_image",
)
load(
    "@prelude-si//macros:native.bzl",
    _alias = "alias",
)

def container_image(
        name,
        image_name = None,
        dockerfile = "Dockerfile",
        organization = "systeminit",
        source_url = "http://github.com/systeminit/si.git",
        author = "The System Initiative <dev@systeminit.com>",
        license = "Apache-2.0",
        artifact_destination = "oci://docker.io",
        artifact_cname = None,
        platform_targets = _VALID_LINUX_PLATFORM_TARGETS,
        skip_all_publish = False,
        skip_all_promote = False,
        default_target = False,
        visibility = ["PUBLIC"],
        **kwargs):
    """Create container image artifact with publish/promote targets.

    Creates targets:
    - :{name}-container-image
    - :publish-{name}-container-image
    - :promote-{name}-container-image

    Args:
        name: Container image name
        image_name: Override image name (defaults to name)
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

    # Default image_name to name
    if image_name == None:
        image_name = name

    # Validate platform_targets
    _validate_linux_platform_targets(platform_targets, "container_image({})".format(name))

    # Base container image target
    container_image_name = "{}-container-image".format(name)
    _container_image(
        name = container_image_name,
        image_name = image_name,
        dockerfile = dockerfile,
        organization = organization,
        source_url = source_url,
        author = author,
        license = license,
        platform_targets = platform_targets,
        visibility = visibility,
        **kwargs
    )

    # Base publish target
    _artifact_publish(
        name = "publish-{}-container-image".format(name),
        artifact = ":{}".format(container_image_name),
        destination = artifact_destination,
        cname = artifact_cname,
        platform_targets = platform_targets,
        skip_all = skip_all_publish,
        visibility = visibility,
    )

    # Base promote target
    _artifact_promote(
        name = "promote-{}-container-image".format(name),
        family = image_name,
        variant = "container",
        destination = artifact_destination,
        cname = artifact_cname,
        organization = organization,
        platform_targets = platform_targets,
        skip_all = skip_all_promote,
        visibility = visibility,
    )

    if default_target:
        _alias(
            name = name,
            actual = ":{}".format(container_image_name),
            visibility = visibility,
        )
