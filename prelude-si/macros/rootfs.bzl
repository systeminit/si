load(
    "@prelude-si//:rootfs.bzl",
    _rootfs = "rootfs",
    #_rootfs_promote = "rootfs_promote",
    _build_rootfs = "build_rootfs",
)

def rootfs(
        name,
        **kwargs):
    _rootfs(
        name = name,
        **kwargs,
    )
