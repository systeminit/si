def nix_flake_lock_impl(ctx: "context") -> [DefaultInfo.type]:
    out = ctx.actions.declare_output("flake.lock")

    output = ctx.actions.copy_file(out, ctx.attrs.src)

    return [DefaultInfo(default_output = out)]

nix_flake_lock = rule(
    impl = nix_flake_lock_impl,
    attrs = {
        "src": attrs.source(
            doc = """flake.lock source.""",
        ),
        "nix_flake": attrs.dep(
            default = "//:flake.nix",
            doc = """Nix flake dependency.""",
        ),
    },
)
