load("@prelude//python:toolchain.bzl", "PythonToolchainInfo")
load("//build_metadata:toolchain.bzl", "BuildMetadataToolchainInfo")

def git_metadata_impl(ctx: AnalysisContext) -> list[Provider]:
    """Generates git metadata JSON file by running git commands locally."""
    metadata_file = ctx.actions.declare_output("git_metadata.json")

    build_metadata_toolchain = ctx.attrs._build_metadata_toolchain[BuildMetadataToolchainInfo]

    cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        build_metadata_toolchain.generate_git_metadata[DefaultInfo].default_outputs,
        metadata_file.as_output(),
    )

    ctx.actions.run(
        cmd,
        category = "git_metadata",
        local_only = True,  # NOTE: Must run locally to access .git directory
    )

    return [
        DefaultInfo(default_output = metadata_file),
    ]

git_metadata = rule(
    impl = git_metadata_impl,
    attrs = {
        "_build_metadata_toolchain": attrs.toolchain_dep(
            default = "toolchains//:build_metadata",
            providers = [BuildMetadataToolchainInfo],
        ),
        "_python_toolchain": attrs.toolchain_dep(
            default = "toolchains//:python",
            providers = [PythonToolchainInfo],
        ),
    },
)

def rust_git_metadata_outdir_impl(ctx: AnalysisContext) -> list[Provider]:
    """Generates git_metadata.rs in an OUT_DIR-compatible directory."""

    # Generate git_metadata.rs into the OUT_DIR directory
    out_dir = ctx.actions.declare_output("OUT_DIR", dir = True)
    output_file = cmd_args(out_dir.as_output(), "/git_metadata.rs", delimiter = "")

    metadata_file = ctx.attrs.git_metadata[DefaultInfo].default_outputs[0]

    build_metadata_toolchain = ctx.attrs._build_metadata_toolchain[BuildMetadataToolchainInfo]

    cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        build_metadata_toolchain.generate_git_metadata_rust[DefaultInfo].default_outputs,
        metadata_file,
        output_file,
    )

    ctx.actions.run(cmd, category = "generate_rust_git_metadata")

    return [DefaultInfo(default_output = out_dir)]

rust_git_metadata_outdir = rule(
    impl = rust_git_metadata_outdir_impl,
    attrs = {
        "git_metadata": attrs.dep(),
        "_build_metadata_toolchain": attrs.toolchain_dep(
            default = "toolchains//:build_metadata",
            providers = [BuildMetadataToolchainInfo],
        ),
        "_python_toolchain": attrs.toolchain_dep(
            default = "toolchains//:python",
            providers = [PythonToolchainInfo],
        ),
    },
)

def deno_git_metadata_typescript_impl(ctx: AnalysisContext) -> list[Provider]:
    """Generates git_metadata.ts from git metadata JSON."""

    # Output file with fixed name for predictable imports
    output_file = ctx.actions.declare_output("git_metadata.ts")

    metadata_file = ctx.attrs.git_metadata[DefaultInfo].default_outputs[0]

    build_metadata_toolchain = ctx.attrs._build_metadata_toolchain[BuildMetadataToolchainInfo]

    cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        build_metadata_toolchain.generate_git_metadata_typescript[DefaultInfo].default_outputs,
        metadata_file,
        output_file.as_output(),
    )

    ctx.actions.run(cmd, category = "generate_typescript_git_metadata")

    return [DefaultInfo(default_output = output_file)]

deno_git_metadata_typescript = rule(
    impl = deno_git_metadata_typescript_impl,
    attrs = {
        "git_metadata": attrs.dep(),
        "_build_metadata_toolchain": attrs.toolchain_dep(
            default = "toolchains//:build_metadata",
            providers = [BuildMetadataToolchainInfo],
        ),
        "_python_toolchain": attrs.toolchain_dep(
            default = "toolchains//:python",
            providers = [PythonToolchainInfo],
        ),
    },
)
