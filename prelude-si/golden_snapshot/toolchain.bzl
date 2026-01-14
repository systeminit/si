GoldenSnapshotToolchainInfo = provider(fields = {
    "golden_snapshot_build": typing.Any,
})

def golden_snapshot_toolchain_impl(ctx) -> list[[DefaultInfo, GoldenSnapshotToolchainInfo]]:
    """
    A golden snapshot toolchain for building Firecracker VM snapshots.
    """
    return [
        DefaultInfo(),
        GoldenSnapshotToolchainInfo(
            golden_snapshot_build = ctx.attrs._golden_snapshot_build,
        ),
    ]

golden_snapshot_toolchain = rule(
    impl = golden_snapshot_toolchain_impl,
    attrs = {
        "_golden_snapshot_build": attrs.dep(
            default = "prelude-si//golden_snapshot:golden_snapshot_build.sh",
        ),
    },
    is_toolchain_rule = True,
)
