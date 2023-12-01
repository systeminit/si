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
        **kwargs,
    )

def nix_omnibus_pkg(
        name,
        source_url = "http://github.com/systeminit/si.git",
        author = "The System Initiative <dev@systeminit.com>",
        license = "Apache-2.0",
        visibility = ["PUBLIC"],
        **kwargs):
    _nix_omnibus_pkg(
        name = name,
        source_url = source_url,
        author = author,
        license = license,
        visibility = visibility,
        **kwargs,
    )
