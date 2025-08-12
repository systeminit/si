TomlToolchainInfo = provider(
    fields = {
        "build_context": typing.Any,
        "cargo_sort_config": provider_field(typing.Any, default = None),
        "taplo_config": provider_field(typing.Any, default = None),
        "taplo_path": provider_field(typing.Any, default = None),
        "cargo_path": provider_field(typing.Any, default = None),
        "cargo_sort_path": provider_field(typing.Any, default = None),
        "toml_format": typing.Any,
    },
)

def toml_toolchain_impl(ctx) -> list[[DefaultInfo, TomlToolchainInfo]]:
    """
    A TOML toolchain.
    """

    if ctx.attrs.cargo_sort_config:
        cargo_sort_config = ctx.attrs.cargo_sort_config[DefaultInfo].default_outputs[0]
    else:
        cargo_sort_config = None

    if ctx.attrs.taplo_config:
        taplo_config = ctx.attrs.taplo_config[DefaultInfo].default_outputs[0]
    else:
        taplo_config = None

    # Use system taplo binary
    taplo_path = cmd_args("taplo")

    # Get cargo binary path (we'll use the nightly rust distribution)
    if ctx.attrs.cargo_dist:
        # Access the cargo binary from the RustDistributionInfo provider
        cargo_path = cmd_args(ctx.attrs.cargo_dist[DefaultInfo].default_outputs[0], "/bin/cargo", delimiter="")
    else:
        cargo_path = cmd_args("cargo")

    # Use system cargo-sort binary
    cargo_sort_path = cmd_args("cargo-sort")

    return [
        DefaultInfo(),
        TomlToolchainInfo(
            build_context = ctx.attrs._build_context,
            cargo_sort_config = cargo_sort_config,
            taplo_config = taplo_config,
            taplo_path = taplo_path,
            cargo_path = cargo_path,
            cargo_sort_path = cargo_sort_path,
            toml_format = ctx.attrs._toml_format,
        ),
    ]

toml_toolchain = rule(
    impl = toml_toolchain_impl,
    attrs = {
        "_build_context": attrs.dep(
            default = "prelude-si//toml:build_context.py",
        ),
        "cargo_sort_config": attrs.option(
            attrs.dep(providers = [DefaultInfo]),
            default = None,
        ),
        "taplo_config": attrs.option(
            attrs.dep(providers = [DefaultInfo]),
            default = None,
        ),
        "cargo_dist": attrs.option(
            attrs.exec_dep(providers = [DefaultInfo]),
            default = None,
        ),
        "_toml_format": attrs.dep(
            default = "prelude-si//toml:toml_format.py",
        ),
    },
    is_toolchain_rule = True,
)
