load(
    "@prelude//python:toolchain.bzl",
    "PythonToolchainInfo",
)
load(
    "@prelude//utils:cmd_script.bzl",
    "cmd_script",
)
load(
    "@prelude-si//:artifact.bzl",
    "ArtifactInfo",
)
load(
    "//artifact:toolchain.bzl",
    "ArtifactToolchainInfo",
)
load(
    "//deno:toolchain.bzl",
    "DenoToolchainInfo",
    "DenoWorkspaceInfo",
)
load(
    "//git:toolchain.bzl",
    "GitToolchainInfo",
)
load(
    "//git.bzl",
    _git_info = "git_info",
)

DenoTargetRuntimeInfo = provider(fields = {
    "target": provider_field(typing.Any, default = None),
    "runtime_dir": provider_field(typing.Any, default = None),
})

def deno_binary_impl(ctx: AnalysisContext) -> list[Provider]:
    deno_toolchain = ctx.attrs._deno_toolchain[DenoToolchainInfo]

    # Read target string from toolchain (selected by platform constraints)
    target_string = deno_toolchain.target_string if deno_toolchain.target_string else None

    # Determine binary extension based on target platform
    binary_name = ctx.attrs.out
    if target_string and target_string.startswith("windows-"):
        if not binary_name.endswith(".exe"):
            binary_name = binary_name + ".exe"

    out = ctx.actions.declare_output(binary_name)

    # Build hidden dependencies list
    hidden_deps = list(ctx.attrs.srcs)

    # Add target runtime as hidden dependency (triggers download if not cached)
    if target_string:
        runtime_dep = ctx.attrs._target_runtimes.get(target_string)
        if runtime_dep:
            runtime_info = runtime_dep[DenoTargetRuntimeInfo]
            hidden_deps.append(runtime_info.runtime_dir)

    cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        deno_toolchain.deno_binary[DefaultInfo].default_outputs[0],
        "--deno-exe",
        deno_toolchain.deno_exe,
        "--input",
        ctx.attrs.main,
        "--output",
        out.as_output(),
        hidden = hidden_deps,
    )

    # Add explicit source mappings (following pnpm pattern)
    # Buck2's {} placeholder expands to the artifact path, we prepend the destination
    for src in ctx.attrs.srcs:
        cmd.add("--src")
        cmd.add(cmd_args(src, format = "{}"))

    # Add deno.json if provided
    if ctx.attrs.deno_json:
        cmd.add("--deno-json", ctx.attrs.deno_json)
        hidden_deps.append(ctx.attrs.deno_json)

    # Add deno.lock if provided
    if ctx.attrs.deno_lock:
        cmd.add("--deno-lock", ctx.attrs.deno_lock)
        hidden_deps.append(ctx.attrs.deno_lock)

    # Add extra_srcs as destination=source pairs
    for (dest, src) in ctx.attrs.extra_srcs.items():
        cmd.add("--extra-src")
        cmd.add(cmd_args(src, format = dest + "={}"))
        hidden_deps.append(src)

    if ctx.attrs.deno_cache:
        deno_cache_provider = ctx.attrs.deno_cache[DenoWorkspaceInfo]
        cmd.add("--deno-dir", deno_cache_provider.cache_dir)
        cmd.add("--workspace-dir", deno_cache_provider.workspace_dir)

    # Handle cross-compilation with target runtime
    if target_string:
        cmd.add("--target", target_string)

    if ctx.attrs.permissions:
        cmd.add("--permissions", *ctx.attrs.permissions)

    if ctx.attrs.unstable_flags:
        cmd.add("--unstable-flags", *ctx.attrs.unstable_flags)

    # Add includes for JavaScript files from extra_srcs (generated files)
    for src in ctx.attrs.extra_srcs.values():
        if src.basename.endswith(".js"):
            cmd.add("--includes", src)

    ctx.actions.run(cmd, category = "deno", identifier = "deno_binary")

    return [
        DefaultInfo(default_output = out),
        RunInfo(args = cmd_args(out)),
    ]

deno_binary = rule(
    impl = deno_binary_impl,
    attrs = {
        "main": attrs.source(doc = "The entry point TypeScript/JavaScript file"),
        "srcs": attrs.list(
            attrs.source(),
            default = [],
            doc = "All source files that are part of the compilation",
        ),
        "deno_json": attrs.option(
            attrs.source(),
            default = None,
            doc = "The deno.json configuration file",
        ),
        "deno_lock": attrs.option(
            attrs.source(),
            default = None,
            doc = "The deno.lock file for reproducible builds",
        ),
        "out": attrs.string(doc = "The name of the output binary"),
        "extra_srcs": attrs.dict(
            key = attrs.string(),
            value = attrs.source(),
            default = {},
            doc = "Mapping of destination paths (relative to input file's parent) to source artifacts",
        ),
        "deno_cache": attrs.option(attrs.dep(providers = [DenoWorkspaceInfo]), default = None),
        "permissions": attrs.list(
            attrs.string(),
            default = [],
            doc = "List of Deno permissions to grant (e.g., read, write, net)",
        ),
        "unstable_flags": attrs.list(
            attrs.string(),
            default = [],
            doc = "List of unstable flags to enable",
        ),
        "_python_toolchain": attrs.toolchain_dep(
            default = "toolchains//:python",
            providers = [PythonToolchainInfo],
        ),
        "_deno_toolchain": attrs.toolchain_dep(
            default = "toolchains//:deno",
            providers = [DenoToolchainInfo],
        ),
        "_target_runtimes": attrs.dict(
            key = attrs.string(),
            value = attrs.dep(providers = [DenoTargetRuntimeInfo]),
            default = {
                "linux-x86_64": "toolchains//:deno-runtime-linux-x86_64",
                "linux-aarch64": "toolchains//:deno-runtime-linux-aarch64",
                "darwin-x86_64": "toolchains//:deno-runtime-darwin-x86_64",
                "darwin-aarch64": "toolchains//:deno-runtime-darwin-aarch64",
                "windows-x86_64": "toolchains//:deno-runtime-windows-x86_64",
                "windows-aarch64": "toolchains//:deno-runtime-windows-aarch64",
            },
        ),
    },
)

def deno_format_impl(ctx: AnalysisContext) -> list[Provider]:
    """Implementation of the deno_format rule."""
    deno_toolchain = ctx.attrs._deno_toolchain[DenoToolchainInfo]

    cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        deno_toolchain.deno_format[DefaultInfo].default_outputs[0],
        "--deno-exe",
        deno_toolchain.deno_exe,
    )

    for src in ctx.attrs.srcs:
        cmd.add("--input", src)

    if ctx.attrs.check:
        cmd.add("--check")

    if ctx.attrs.ignore:
        for ignore_path in ctx.attrs.ignore:
            cmd.add("--ignore=" + ignore_path)

    return [
        DefaultInfo(),
        RunInfo(args = cmd),
    ]

deno_format = rule(
    impl = deno_format_impl,
    attrs = {
        "srcs": attrs.list(
            attrs.source(),
            default = [],
            doc = "The source files to format",
        ),
        "check": attrs.bool(
            default = False,
            doc = "Check if files are formatted without making changes",
        ),
        "ignore": attrs.list(
            attrs.string(),
            default = [],
            doc = "List of files or directories to ignore",
        ),
        "_python_toolchain": attrs.toolchain_dep(
            default = "toolchains//:python",
            providers = [PythonToolchainInfo],
        ),
        "_deno_toolchain": attrs.toolchain_dep(
            default = "toolchains//:deno",
            providers = [DenoToolchainInfo],
        ),
    },
)

def deno_run_impl(ctx: AnalysisContext) -> list[Provider]:
    out = ctx.actions.declare_output(ctx.attrs.out)
    deno_toolchain = ctx.attrs._deno_toolchain[DenoToolchainInfo]

    cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        deno_toolchain.deno_run[DefaultInfo].default_outputs[0],
        "--deno-exe",
        deno_toolchain.deno_exe,
        "--input",
        ctx.attrs.main,
        "--output",
        out.as_output(),
        hidden = ctx.attrs.srcs,
    )

    if ctx.attrs.deno_cache:
        deno_cache_provider = ctx.attrs.deno_cache[DenoWorkspaceInfo]
        cmd.add("--deno-dir", deno_cache_provider.cache_dir)
        cmd.add("--workspace-dir", deno_cache_provider.workspace_dir)

    if ctx.attrs.permissions:
        cmd.add("--permissions", *ctx.attrs.permissions)

    if ctx.attrs.unstable_flags:
        cmd.add("--unstable-flags", *ctx.attrs.unstable_flags)

    ctx.actions.run(cmd, category = "deno", identifier = "deno_run")

    return [
        DefaultInfo(default_output = out),
    ]

deno_run = rule(
    impl = deno_run_impl,
    attrs = {
        "main": attrs.source(doc = "The entry point TypeScript/JavaScript file"),
        "srcs": attrs.list(
            attrs.source(),
            default = [],
            doc = "All source files that are part of the compilation",
        ),
        "out": attrs.string(doc = "The name of the output binary"),
        "deno_cache": attrs.option(attrs.dep(providers = [DenoWorkspaceInfo]), default = None),
        "permissions": attrs.list(
            attrs.string(),
            default = [],
            doc = "List of Deno permissions to grant (e.g., read, write, net)",
        ),
        "unstable_flags": attrs.list(
            attrs.string(),
            default = [],
            doc = "List of unstable flags to enable",
        ),
        "_python_toolchain": attrs.toolchain_dep(
            default = "toolchains//:python",
            providers = [PythonToolchainInfo],
        ),
        "_deno_toolchain": attrs.toolchain_dep(
            default = "toolchains//:deno",
            providers = [DenoToolchainInfo],
        ),
    },
)

def deno_test_impl(ctx: AnalysisContext) -> list[Provider]:
    """Implementation of the deno_test rule."""
    deno_toolchain = ctx.attrs._deno_toolchain[DenoToolchainInfo]

    shim = cmd_script(
        ctx = ctx,
        name = "deno_shim",
        cmd = cmd_args(deno_toolchain.deno_exe),
    )

    # Build list of hidden inputs
    hidden_inputs = list(ctx.attrs.srcs)
    if ctx.attrs.extra_srcs:
        hidden_inputs.extend(ctx.attrs.extra_srcs.values())

    cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        deno_toolchain.deno_test[DefaultInfo].default_outputs[0],
        "--deno-exe",
        deno_toolchain.deno_exe,
        hidden = hidden_inputs,
    )

    for src in ctx.attrs.srcs:
        cmd.add("--input", src)

    # Add extra_srcs as destination=source pairs
    for (dest, src) in ctx.attrs.extra_srcs.items():
        cmd.add("--extra-src")
        cmd.add(cmd_args(src, format = dest + "={}"))

    if ctx.attrs.filter:
        cmd.add("--filter", ctx.attrs.filter)

    if ctx.attrs.ignore:
        for ignore_path in ctx.attrs.ignore:
            cmd.add("--ignore", ignore_path)

    if ctx.attrs.parallel:
        cmd.add("--parallel")

    if ctx.attrs.watch:
        cmd.add("--watch")

    if ctx.attrs.permissions:
        cmd.add("--permissions", *ctx.attrs.permissions)

    if ctx.attrs.unstable_flags:
        cmd.add("--unstable-flags", *ctx.attrs.unstable_flags)

    if ctx.attrs.env:
        cmd.add("--env", *ctx.attrs.env)

    if ctx.attrs.no_check:
        cmd.add("--no-check")

    return [
        DefaultInfo(),
        RunInfo(args = cmd),
        ExternalRunnerTestInfo(
            type = "custom",
            command = [cmd],
            env = {},
            run_from_project_root = False,
        ),
    ]

deno_test = rule(
    impl = deno_test_impl,
    attrs = {
        "srcs": attrs.list(
            attrs.source(),
            default = [],
            doc = "The test files to run",
        ),
        "extra_srcs": attrs.dict(
            key = attrs.string(),
            value = attrs.source(),
            default = {},
            doc = "Mapping of destination paths (relative to test directory) to source artifacts",
        ),
        "filter": attrs.option(
            attrs.string(),
            default = None,
            doc = "Run tests with this string or pattern in the test name",
        ),
        "ignore": attrs.list(
            attrs.string(),
            default = [],
            doc = "List of files or directories to ignore",
        ),
        "parallel": attrs.bool(
            default = True,
            doc = "Run tests in parallel",
        ),
        "permissions": attrs.list(
            attrs.string(),
            default = [],
            doc = "List of Deno permissions to grant (e.g., read, write, net)",
        ),
        "unstable_flags": attrs.list(
            attrs.string(),
            default = [],
            doc = "List of unstable flags to enable",
        ),
        "watch": attrs.bool(
            default = False,
            doc = "Watch for file changes and restart tests",
        ),
        "env": attrs.list(
            attrs.string(),
            default = [],
            doc = "Environment variables to set (format: KEY=VALUE)",
        ),
        "no_check": attrs.bool(
            default = False,
            doc = "Skip type checking",
        ),
        "_python_toolchain": attrs.toolchain_dep(
            default = "toolchains//:python",
            providers = [PythonToolchainInfo],
        ),
        "_deno_toolchain": attrs.toolchain_dep(
            default = "toolchains//:deno",
            providers = [DenoToolchainInfo],
        ),
    },
)

def _deno_workspace_impl(ctx: AnalysisContext) -> list[Provider]:
    deno_toolchain = ctx.attrs._deno_toolchain[DenoToolchainInfo]

    workspace_out = ctx.actions.declare_output(
        ctx.attrs.workspace_out_name,
        dir = True,
    )
    cache_out = ctx.actions.declare_output(ctx.attrs.cache_out_name, dir = True)

    cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        deno_toolchain.deno_workspace[DefaultInfo].default_outputs[0],
    )

    cmd.add(
        "--root-config",
        ctx.attrs.root_config[DefaultInfo].default_outputs[0],
    )
    cmd.add("--workspace-dir", workspace_out.as_output())
    cmd.add("--deno-exe", deno_toolchain.deno_exe)
    cmd.add("--deno-dir", cache_out.as_output())

    for src in ctx.attrs.srcs:
        cmd.add("--src", src)

    ctx.actions.run(
        cmd,
        category = "deno_workspace",
    )

    return [
        DefaultInfo(default_output = workspace_out),
        DenoWorkspaceInfo(
            workspace_dir = workspace_out,
            cache_dir = cache_out,
        ),
    ]

deno_workspace = rule(
    impl = _deno_workspace_impl,
    attrs = {
        "root_config": attrs.dep(doc = "Root deno.json configuration file"),
        "srcs": attrs.list(
            attrs.source(),
            doc = "All source files and deno.json files for the workspace.",
        ),
        "workspace_out_name": attrs.string(
            default = "workspace",
            doc = "The name of the output directory for the workspace structure.",
        ),
        "cache_out_name": attrs.string(
            default = "deno_cache",
            doc = "The name of the output directory for the Deno cache.",
        ),
        "_python_toolchain": attrs.toolchain_dep(
            default = "toolchains//:python",
            providers = [PythonToolchainInfo],
        ),
        "_deno_toolchain": attrs.toolchain_dep(
            default = "toolchains//:deno",
            providers = [DenoToolchainInfo],
        ),
    },
)

def deno_target_runtime_impl(ctx: AnalysisContext) -> list[Provider]:
    """Downloads a Deno target runtime for cross-compilation."""
    deno_toolchain = ctx.attrs._deno_toolchain[DenoToolchainInfo]

    # Output is the downloaded runtime artifact
    runtime_out = ctx.actions.declare_output(
        "deno-runtime-{}".format(ctx.attrs.target),
        dir = True,
    )

    cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        deno_toolchain.deno_target_runtime[DefaultInfo].default_outputs[0],
        "--deno-exe",
        deno_toolchain.deno_exe,
        "--target",
        ctx.attrs.target,
        "--output-dir",
        runtime_out.as_output(),
    )

    ctx.actions.run(cmd, category = "deno", identifier = "deno_target_runtime")

    return [
        DefaultInfo(default_output = runtime_out),
        DenoTargetRuntimeInfo(
            target = ctx.attrs.target,
            runtime_dir = runtime_out,
        ),
    ]

deno_target_runtime = rule(
    impl = deno_target_runtime_impl,
    attrs = {
        "target": attrs.string(doc = "Target platform string (e.g., linux-x86_64)"),
        "_python_toolchain": attrs.toolchain_dep(
            default = "toolchains//:python",
            providers = [PythonToolchainInfo],
        ),
        "_deno_toolchain": attrs.toolchain_dep(
            default = "toolchains//:deno",
            providers = [DenoToolchainInfo],
        ),
    },
)

def _deno_binary_artifact_impl(ctx):
    """Implementation for deno_binary_artifact rule."""
    binary = ctx.attrs.binary[DefaultInfo].default_outputs[0]
    git_info = _git_info(ctx)

    deno_toolchain = ctx.attrs._deno_toolchain[DenoToolchainInfo]
    artifact_toolchain = ctx.attrs._artifact_toolchain[ArtifactToolchainInfo]

    # Get platform information from Deno toolchain
    target_os = deno_toolchain.target_os
    target_arch = deno_toolchain.target_arch

    # Generate metadata
    build_metadata = ctx.actions.declare_output("build_metadata.json")

    metadata_cmd = cmd_args(
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

    # Create archive (tar.gz for unix, zip for Windows)
    archive_ext = ".zip" if target_os == "windows" else ".tar.gz"
    archive = ctx.actions.declare_output("{}{}".format(ctx.attrs.binary_name, archive_ext))

    archive_cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        artifact_toolchain.create_binary_archive[DefaultInfo].default_outputs,
        "--binary",
        binary,
        "--metadata",
        build_metadata,
        "--output",
        archive.as_output(),
        "--os",
        target_os,
        # Note: no --usr-local-bin flag, using flat layout
    )

    ctx.actions.run(archive_cmd, category = "binary_archive")

    return [
        DefaultInfo(default_output = archive),
        ArtifactInfo(
            artifact = archive,
            metadata = build_metadata,
            family = ctx.attrs.family,
            variant = ctx.attrs.variant,
        ),
    ]

deno_binary_artifact = rule(
    impl = _deno_binary_artifact_impl,
    attrs = {
        "binary": attrs.dep(
            doc = "The built deno_binary target.",
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
        "_git_toolchain": attrs.toolchain_dep(
            default = "toolchains//:git",
            providers = [GitToolchainInfo],
        ),
        "_python_toolchain": attrs.toolchain_dep(
            default = "toolchains//:python",
            providers = [PythonToolchainInfo],
        ),
        "_deno_toolchain": attrs.toolchain_dep(
            default = "toolchains//:deno",
            providers = [DenoToolchainInfo],
        ),
        "_artifact_toolchain": attrs.toolchain_dep(
            default = "toolchains//:artifact",
            providers = [ArtifactToolchainInfo],
        ),
    },
)
