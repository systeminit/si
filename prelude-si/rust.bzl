load("@prelude//decls/re_test_common.bzl", "re_test_common")
load("@prelude//python:toolchain.bzl", "PythonToolchainInfo")
load("@prelude//test/inject_test_run_info.bzl", "inject_test_run_info")
load("@prelude//tests:re_utils.bzl", "get_re_executors_from_props")
load("@prelude-si//:artifact.bzl", "ArtifactInfo")
load("@prelude-si//:test.bzl", "inject_test_env")
load("//artifact:toolchain.bzl", "ArtifactToolchainInfo")
load("//platform.bzl", "get_host_platform")
load("//rust:toolchain.bzl", "SiRustToolchainInfo")

def clippy_check_impl(ctx: AnalysisContext) -> list[[
    DefaultInfo,
    RunInfo,
    ExternalRunnerTestInfo,
]]:
    clippy_txt = ctx.attrs.clippy_txt_dep[DefaultInfo].default_outputs

    si_rust_toolchain = ctx.attrs._si_rust_toolchain[SiRustToolchainInfo]

    run_cmd_args = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        si_rust_toolchain.clippy_output[DefaultInfo].default_outputs,
        clippy_txt,
    )

    args_file = ctx.actions.write("args.txt", run_cmd_args)

    # Setup a RE executor based on the `remote_execution` param.
    re_executor, executor_overrides = get_re_executors_from_props(ctx)

    # We implicitly make the target run from the project root if remote
    # excution options were specified
    run_from_project_root = "buck2_run_from_project_root" in (
        ctx.attrs.labels or []
    ) or re_executor != None

    return inject_test_run_info(
        ctx,
        ExternalRunnerTestInfo(
            type = "clippy",
            command = [run_cmd_args],
            env = ctx.attrs.env,
            labels = ctx.attrs.labels,
            contacts = ctx.attrs.contacts,
            default_executor = re_executor,
            executor_overrides = executor_overrides,
            run_from_project_root = run_from_project_root,
            use_project_relative_paths = run_from_project_root,
        ),
    ) + [
        DefaultInfo(default_output = args_file),
    ]

clippy_check = rule(
    impl = clippy_check_impl,
    attrs = {
        "clippy_txt_dep": attrs.dep(
            doc = """Clippy sub target dep from a Rust library or binary""",
        ),
        "_python_toolchain": attrs.toolchain_dep(
            default = "toolchains//:python",
            providers = [PythonToolchainInfo],
        ),
        "_si_rust_toolchain": attrs.toolchain_dep(
            default = "toolchains//:si_rust",
            providers = [SiRustToolchainInfo],
        ),
    } | re_test_common.test_args() | inject_test_env.args(),
)

def rustfmt_check_impl(ctx: AnalysisContext) -> list[[
    DefaultInfo,
    RunInfo,
    ExternalRunnerTestInfo,
]]:
    si_rust_toolchain = ctx.attrs._si_rust_toolchain[SiRustToolchainInfo]
    crate_ctx = crate_context(ctx)

    run_cmd_args = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        si_rust_toolchain.rustfmt_check[DefaultInfo].default_outputs,
    )
    if si_rust_toolchain.rustfmt_toml:
        run_cmd_args.add("--config-path")
        run_cmd_args.add(si_rust_toolchain.rustfmt_toml)

    # Add rustfmt binary path from the distribution
    run_cmd_args.add("--rustfmt-path")
    run_cmd_args.add(si_rust_toolchain.rustfmt_path)

    run_cmd_args.add(cmd_args(
        [crate_ctx.srcs_tree, ctx.label.package, ctx.attrs.crate_root],
        delimiter = "/",
    ))

    args_file = ctx.actions.write("args.txt", run_cmd_args)

    # Setup a RE executor based on the `remote_execution` param.
    re_executor, executor_overrides = get_re_executors_from_props(ctx)

    # We implicitly make the target run from the project root if remote
    # excution options were specified
    run_from_project_root = "buck2_run_from_project_root" in (
        ctx.attrs.labels or []
    ) or re_executor != None

    return inject_test_run_info(
        ctx,
        ExternalRunnerTestInfo(
            type = "rustfmt",
            command = [run_cmd_args],
            env = ctx.attrs.env,
            labels = ctx.attrs.labels,
            contacts = ctx.attrs.contacts,
            default_executor = re_executor,
            executor_overrides = executor_overrides,
            run_from_project_root = run_from_project_root,
            use_project_relative_paths = run_from_project_root,
        ),
    ) + [
        DefaultInfo(default_output = args_file),
    ]

rustfmt_check = rule(
    impl = rustfmt_check_impl,
    attrs = {
        "srcs": attrs.list(
            attrs.source(),
            default = [],
            doc = """The set of Rust source files in the crate.""",
        ),
        "crate_root": attrs.string(
            doc = """Top level source file for the crate.""",
        ),
        "_python_toolchain": attrs.toolchain_dep(
            default = "toolchains//:python",
            providers = [PythonToolchainInfo],
        ),
        "_si_rust_toolchain": attrs.toolchain_dep(
            default = "toolchains//:si_rust",
            providers = [SiRustToolchainInfo],
        ),
    } | re_test_common.test_args() | inject_test_env.args(),
)

CrateContext = record(
    srcs_tree = field(Artifact),
)

def crate_context(ctx: AnalysisContext) -> CrateContext:
    srcs_tree = ctx.actions.declare_output("__src")

    si_rust_toolchain = ctx.attrs._si_rust_toolchain[SiRustToolchainInfo]

    cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        si_rust_toolchain.crate_context[DefaultInfo].default_outputs,
    )
    for src in ctx.attrs.srcs:
        cmd.add("--src")
        cmd.add(src)
    cmd.add(srcs_tree.as_output())

    ctx.actions.run(cmd, category = "crate_context")

    return CrateContext(
        srcs_tree = srcs_tree,
    )

def _rust_binary_artifact_impl(ctx) -> list[[
    DefaultInfo,
    ArtifactInfo,
]]:
    artifact_toolchain = ctx.attrs._artifact_toolchain[ArtifactToolchainInfo]
    si_rust_toolchain = ctx.attrs._si_rust_toolchain[SiRustToolchainInfo]

    binary = ctx.attrs.binary[DefaultInfo].default_outputs[0]

    git_metadata_file = ctx.attrs.git_metadata[DefaultInfo].default_outputs[0]

    # Get host platform information
    host_os, host_arch = get_host_platform()

    # Get target platform information from host platform (i.e. not yet cross-compilation aware)
    target_os = host_os
    target_arch = host_arch

    variant = "binary"

    # Build artitfact
    pkg_metadata = ctx.actions.declare_output("pkg_metadata.json")
    archive_ext = ".zip" if target_os == "windows" else ".tar.gz"
    archive = ctx.actions.declare_output("{}{}".format(ctx.attrs.binary_name, archive_ext))
    archive_cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        artifact_toolchain.create_binary_archive[DefaultInfo].default_outputs,
        "--name",
        ctx.attrs.binary_name,
        "--binary",
        binary,
        "--git-info-json",
        git_metadata_file,
        "--artifact-out-file",
        archive.as_output(),
        "--pkg-metadata-out-file",
        pkg_metadata.as_output(),
        "--os",
        target_os,
        "--arch",
        target_arch,
        "--author",
        ctx.attrs.author,
        "--source-url",
        ctx.attrs.source_url,
        "--license",
        ctx.attrs.license,
        "--usr-local-bin",
        "--binary-name",
        ctx.attrs.binary_name,
    )
    ctx.actions.run(archive_cmd, category = "build_binary_artifact")

    # Generate build metadata
    build_metadata = ctx.actions.declare_output("build_metadata.json")
    metadata_cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        artifact_toolchain.generate_build_metadata[DefaultInfo].default_outputs,
        "--artifact-file",
        archive,
        "--git-info-json",
        git_metadata_file,
        "--build-metadata-out-file",
        build_metadata.as_output(),
        "--name",
        ctx.attrs.binary_name,
        "--variant",
        variant,
        "--arch",
        target_arch,
        "--os",
        target_os,
        "--author",
        ctx.attrs.author,
        "--source-url",
        ctx.attrs.source_url,
        "--license",
        ctx.attrs.license,
    )
    ctx.actions.run(metadata_cmd, category = "build_artifact_metadata")

    return [
        DefaultInfo(
            default_output = archive,
            sub_targets = {
                "metadata": [DefaultInfo(default_output = build_metadata)],
            },
        ),
        ArtifactInfo(
            artifact = archive,
            metadata = build_metadata,
            family = ctx.attrs.family,
            variant = variant,
        ),
    ]

rust_binary_artifact = rule(
    impl = _rust_binary_artifact_impl,
    attrs = {
        "binary": attrs.dep(
            doc = "The built rust_binary target.",
        ),
        "binary_name": attrs.string(
            doc = "Artifact name.",
        ),
        "family": attrs.string(
            doc = "Artifact family name.",
        ),
        "author": attrs.string(
            doc = "Author to be used in artifact metadata.",
        ),
        "source_url": attrs.string(
            doc = "Source code URL to be used in artifact metadata.",
        ),
        "license": attrs.string(
            doc = "License string to be used in artifact metadata.",
        ),
        "platform_targets": attrs.list(
            attrs.string(),
            default = [],
            doc = """List of target platforms this artifact supports.
            Used by CI to determine which platforms to build.""",
        ),
        "git_metadata": attrs.dep(
            default = "prelude-si//build_metadata:git",
            doc = """Git metadata target providing repository information.""",
        ),
        "_python_toolchain": attrs.toolchain_dep(
            default = "toolchains//:python",
            providers = [PythonToolchainInfo],
        ),
        "_si_rust_toolchain": attrs.toolchain_dep(
            default = "toolchains//:si_rust",
            providers = [SiRustToolchainInfo],
        ),
        "_artifact_toolchain": attrs.toolchain_dep(
            default = "toolchains//:artifact",
            providers = [ArtifactToolchainInfo],
        ),
    },
)
