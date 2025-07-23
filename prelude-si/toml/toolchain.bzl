TomlToolchainInfo = provider(
    fields = {
        "build_context": typing.Any,
        "cargo_sort_config": provider_field(typing.Any, default = None),
        "taplo_config": provider_field(typing.Any, default = None),
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

    return [
        DefaultInfo(),
        TomlToolchainInfo(
            build_context = ctx.attrs._build_context,
            cargo_sort_config = cargo_sort_config,
            taplo_config = taplo_config,
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
        "_toml_format": attrs.dep(
            default = "prelude-si//toml:toml_format.py",
        ),
    },
    is_toolchain_rule = True,
)
