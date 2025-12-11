load(
    "@prelude//decls/re_test_common.bzl",
    "re_test_common",
)
load(
    "@prelude//python:toolchain.bzl",
    "PythonToolchainInfo",
)
load(
    "@prelude//test/inject_test_run_info.bzl",
    "inject_test_run_info",
)
load(
    "@prelude//tests:re_utils.bzl",
    "get_re_executors_from_props",
)
load(
    "@prelude-si//:artifact.bzl",
    "ArtifactInfo",
)
load(
    "@prelude-si//:test.bzl",
    "inject_test_env",
)
load(
    "//artifact:toolchain.bzl",
    "ArtifactToolchainInfo",
)
load(
    "//git:toolchain.bzl",
    "GitToolchainInfo",
)
load(
    "//git.bzl",
    _git_info = "git_info",
)
load(
    "//rust:toolchain.bzl",
    "SiRustToolchainInfo",
)

def _get_host_platform():
    """Get host platform arch and os for artifact metadata.

    Note: This detects the build host platform. For cross-compilation,
    Deno artifacts use toolchain-provided platform info instead.

    Returns: tuple of (arch, os) as strings matching artifact format
    """
    arch = host_info().arch
    os = host_info().os

    if arch.is_x86_64:
        arch_str = "x86_64"
    elif arch.is_aarch64:
        arch_str = "aarch64"
    else:
        fail("Unsupported host architecture for Rust artifacts")

    if os.is_linux:
        os_str = "linux"
    elif os.is_macos:
        os_str = "macos"
    else:
        fail("Unsupported host OS for Rust artifacts")

    return (arch_str, os_str)

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

def _rust_binary_artifact_impl(ctx):
    binary = ctx.attrs.binary[DefaultInfo].default_outputs[0]
    git_info = _git_info(ctx)
    build_metadata = ctx.actions.declare_output("build_metadata.json")

    si_rust_toolchain = ctx.attrs._si_rust_toolchain[SiRustToolchainInfo]
    artifact_toolchain = ctx.attrs._artifact_toolchain[ArtifactToolchainInfo]

    # Get host platform for artifact metadata
    host_arch, host_os = _get_host_platform()

    # Generate metadata using generic script with explicit platform
    cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        artifact_toolchain.generate_binary_metadata[DefaultInfo].default_outputs,
        "--binary",
        binary,
        "--git-info-json",
        git_info.file,
        "--build-metadata-out-file",
        build_metadata.as_output(),
        "--name",
        ctx.attrs.binary_name,
        "--arch",
        host_arch,
        "--os",
        host_os,
        "--author",
        ctx.attrs.author,
        "--source-url",
        ctx.attrs.source_url,
        "--license",
        ctx.attrs.license,
    )

    ctx.actions.run(cmd, category = "rust_metadata")

    # Create archive using generic script with usr-local-bin layout
    tarred_binary = ctx.actions.declare_output("{}.tar.gz".format(ctx.attrs.binary_name))

    archive_cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        artifact_toolchain.create_binary_archive[DefaultInfo].default_outputs,
        "--binary",
        binary,
        "--metadata",
        build_metadata,
        "--output",
        tarred_binary.as_output(),
        "--os",
        host_os,
        "--usr-local-bin",
        "--binary-name",
        ctx.attrs.binary_name,
    )

    ctx.actions.run(archive_cmd, category = "binary_archive")

    return [
        DefaultInfo(default_output = tarred_binary),
        ArtifactInfo(
            artifact = tarred_binary,
            metadata = build_metadata,
            family = ctx.attrs.family,
            variant = ctx.attrs.variant,
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
        "variant": attrs.string(
            doc = "Artifact variant.",
        ),
        "author": attrs.string(
            doc = """Image author to be used in package metadata.""",
        ),
        "source_url": attrs.string(
            doc = """Source code URL to be used in package metadata.""",
        ),
        "license": attrs.string(
            doc = """License string to be used in package metadata.""",
        ),
        "platform_targets": attrs.list(
            attrs.string(),
            default = [],
            doc = """List of target platforms this artifact supports.
            Used by CI to determine which platforms to build.""",
        ),
        "_git_toolchain": attrs.toolchain_dep(
            default = "toolchains//:git",
            providers = [GitToolchainInfo],
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
