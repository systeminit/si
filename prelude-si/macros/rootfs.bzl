load(
    "@prelude-si//:rootfs.bzl",
    _rootfs = "rootfs",
)
load(
    "@prelude-si//:artifact.bzl",
    _artifact_promote = "artifact_promote",
    _artifact_publish = "artifact_publish",
)

def rootfs(
        name,
        pkg_name,
        source_url = "http://github.com/systeminit/si.git",
        author = "The System Initiative <dev@systeminit.com>",
        license = "Apache-2.0",
        visibility = ["PUBLIC"],
        publish_target = "publish-rootfs",
        promote_target = "promote-rootfs",
        artifact_destination = "s3://si-artifacts-prod",
        artifact_cname = "artifacts.systeminit.com",
        **kwargs):
    _rootfs(
        name = name,
        **kwargs,
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
        family = pkg_name,
        variant = "rootfs",
        destination = artifact_destination,
        cname = artifact_cname,
        visibility = visibility,
    )