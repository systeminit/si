load(
    "@prelude-si//:nix.bzl",
    _nix_flake_lock = "nix_flake_lock",
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
