load("@prelude//python:toolchain.bzl", "PythonToolchainInfo")
load("//mise:toolchain.bzl", "MiseToolchainInfo")

MiseInfo = provider(fields = {
    "mise_tools_dir": provider_field(typing.Any, default = None),  # [Artifact]
    "mise_bootstrap": provider_field(typing.Any, default = None),  # [Artifact]
    "shims_dir": provider_field(typing.Any, default = None),  # [Artifact]
    "rust_version": provider_field(typing.Any, default = None),  # [str | None]
})

def mise_install_impl(ctx: AnalysisContext) -> list[[DefaultInfo, MiseInfo]]:
    mise_tools_dir = ctx.actions.declare_output("mise-tools", dir = True)
    mise_bootstrap = ctx.actions.declare_output("bin/mise")

    # Extract rust version from packages
    rust_version = None
    for package in ctx.attrs.packages:
        if package.startswith("rust@"):
            rust_version = package.split("@", 1)[1]
            break

    mise_toolchain = ctx.attrs._mise_toolchain[MiseToolchainInfo]

    # Create the mise install command
    cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        mise_toolchain.mise_install[DefaultInfo].default_outputs,
        "--mise-tools-dir",
        mise_tools_dir.as_output(),
        "--mise-bootstrap",
        mise_bootstrap.as_output(),
    )

    # Add packages to install
    for package in ctx.attrs.packages:
        cmd.add("--package", package)

    ctx.actions.run(cmd, category = "mise_install")

    return [
        DefaultInfo(
            default_outputs = [mise_tools_dir, mise_bootstrap],
            sub_targets = {
                "mise-tools": [DefaultInfo(default_outputs = [mise_tools_dir])],
                "bootstrap": [DefaultInfo(default_outputs = [mise_bootstrap])],
            },
        ),
        MiseInfo(
            mise_tools_dir = mise_tools_dir,
            mise_bootstrap = mise_bootstrap,
            shims_dir = mise_tools_dir,  # shims are within mise_tools_dir/shims
            rust_version = rust_version,
        ),
    ]

mise_install = rule(
    impl = mise_install_impl,
    attrs = {
        "packages": attrs.list(
            attrs.string(),
            default = [],
            doc = """List of packages to install (e.g., 'python@3.12', 'rust@1.82.0').""",
        ),
        "_python_toolchain": attrs.toolchain_dep(
            default = "toolchains//:python",
            providers = [PythonToolchainInfo],
        ),
        "_mise_toolchain": attrs.toolchain_dep(
            default = "toolchains//:mise",
            providers = [MiseToolchainInfo],
        ),
    },
)
