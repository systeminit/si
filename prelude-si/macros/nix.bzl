load(
    "@prelude-si//:artifact.bzl",
    _artifact_promote = "artifact_promote",
    _artifact_publish = "artifact_publish",
)
load(
    "@prelude-si//:nix.bzl",
    _nix_flake_lock = "nix_flake_lock",
    _nix_omnibus_pkg = "nix_omnibus_pkg",
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

def nix_omnibus_pkg(
        name,
        pkg_name,
        source_url = "http://github.com/systeminit/si.git",
        author = "The System Initiative <dev@systeminit.com>",
        license = "Apache-2.0",
        visibility = ["PUBLIC"],
        publish_target = "publish-omnibus",
        promote_target = "promote-omnibus",
        artifact_destination = "s3://si-artifacts-prod",
        artifact_cname = "artifacts.systeminit.com",
        **kwargs):
    _nix_omnibus_pkg(
        name = name,
        pkg_name = pkg_name,
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
        family = pkg_name,
        variant = "omnibus",
        destination = artifact_destination,
        cname = artifact_cname,
        visibility = visibility,
    )
