load(
    "@prelude//python:toolchain.bzl",
    "PythonToolchainInfo",
)
load(
    "//rust:toolchain.bzl",
    "SiRustToolchainInfo",
)
load(
    "//git:toolchain.bzl",
    "GitToolchainInfo",
)
load(
    "@prelude//decls/re_test_common.bzl",
    "re_test_common",
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
    "@prelude-si//:test.bzl",
    "inject_test_env",
)
load(
    "//git.bzl",
    _git_info = "git_info",
)
load(
    "@prelude-si//:artifact.bzl",
    "ArtifactInfo"
)

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

    # Generate metadata first
    cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        si_rust_toolchain.rust_metadata[DefaultInfo].default_outputs,
        "--binary",
        binary,
        "--git-info-json",
        git_info.file,
        "--build-metadata-out-file",
        build_metadata.as_output(),
        "--name",
        ctx.attrs.binary_name,
        "--author",
        ctx.attrs.author,
        "--source-url",
        ctx.attrs.source_url,
        "--license",
        ctx.attrs.license,
    )

    ctx.actions.run(cmd, category = "rust_metadata")

    # Create a tar.gz archive containing the binary and metadata
    tarred_binary = ctx.actions.declare_output("{}.tar.gz".format(ctx.attrs.binary_name))
    
    tar_cmd = cmd_args(
        "tar",
        "-czf",
        tarred_binary.as_output(),
        "--transform",
        "s|.*{}|usr/local/bin/{}|".format(ctx.attrs.binary_name, ctx.attrs.binary_name),
        "--transform",
        "s|.*build_metadata.json|usr/local/share/{}/metadata.json|".format(ctx.attrs.binary_name),
        binary,
        build_metadata,
    )
    
    ctx.actions.run(tar_cmd, category = "tar_binary")

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
            doc = "The built rust_binary target."
        ),
        "binary_name": attrs.string(
            doc = "Artifact name."
        ),
        "family": attrs.string(
            doc = "Artifact family name."
        ),
        "variant": attrs.string(
            doc = "Artifact variant."
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
    },
)
