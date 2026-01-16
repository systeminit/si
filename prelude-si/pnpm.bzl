load(
    "@prelude//:artifacts.bzl",
    "ArtifactGroupInfo",
)
load(
    "@prelude//:paths.bzl",
    "paths",
)
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
    "@prelude-si//:test.bzl",
    "inject_test_env",
)
load(
    "//pnpm:toolchain.bzl",
    "PnpmToolchainInfo",
)

TypescriptDistInfo = provider(fields = {
    "index_file": provider_field(str, default = "index.js"),  # [str]
})

TypescriptRunnableDistInfo = provider(fields = {
    "runnable_dist": provider_field(typing.Any, default = None),  # [Artifact]
    "bin": provider_field(typing.Any, default = None),  # [str]
})

def _npm_test_impl(
        ctx: AnalysisContext,
        program_run_info: RunInfo,
        program_args: cmd_args,
        test_info_type: str) -> list[[
    DefaultInfo,
    RunInfo,
    ExternalRunnerTestInfo,
]]:
    pnpm_toolchain = ctx.attrs._pnpm_toolchain[PnpmToolchainInfo]
    package_build_ctx = package_build_context(ctx)

    if ctx.attrs.pnpm_exec_cmd_override:
        exec_cmd = cmd_args(
            "pnpm",
            "exec",
            ctx.attrs.pnpm_exec_cmd_override,
        )
    else:
        exec_cmd = cmd_args(program_run_info, format = "{}::abspath")

    run_cmd_args = cmd_args([
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        pnpm_toolchain.exec_cmd[DefaultInfo].default_outputs,
        "--cwd",
        cmd_args([package_build_ctx.srcs_tree, ctx.label.package], delimiter = "/"),
        "--",
        exec_cmd,
    ])
    run_cmd_args.add(program_args)

    args_file = ctx.actions.write("args.txt", run_cmd_args)

    # We implicitly make the target run from the project root if remote
    # excution options were specified
    run_from_project_root = "buck2_run_from_project_root" in (
        ctx.attrs.labels or []
    )

    return inject_test_run_info(
        ctx,
        ExternalRunnerTestInfo(
            type = test_info_type,
            command = [run_cmd_args],
            env = ctx.attrs.env,
            labels = ctx.attrs.labels,
            contacts = ctx.attrs.contacts,
            run_from_project_root = run_from_project_root,
            use_project_relative_paths = run_from_project_root,
        ),
    ) + [
        DefaultInfo(default_output = args_file),
    ]

def eslint_impl(ctx: AnalysisContext) -> list[[
    DefaultInfo,
    RunInfo,
    ExternalRunnerTestInfo,
]]:
    args = cmd_args()
    args.add(ctx.attrs.directories)
    args.add("--ext")
    args.add(",".join(ctx.attrs.extensions))
    args.add("--quiet")
    args.add(ctx.attrs.args)

    return _npm_test_impl(
        ctx,
        ctx.attrs.eslint[RunInfo],
        args,
        "eslint",
    )

eslint = rule(
    impl = eslint_impl,
    attrs = {
        "srcs": attrs.list(
            attrs.source(),
            default = [],
            doc = """List of package source files to track.""",
        ),
        "prod_deps_srcs": attrs.dict(
            attrs.string(),
            attrs.source(allow_directory = True),
            default = {},
            doc = """Mapping of dependent prod package paths to source files to track.""",
        ),
        "dev_deps_srcs": attrs.dict(
            attrs.string(),
            attrs.source(allow_directory = True),
            default = {},
            doc = """Mapping of dependent dev package paths to source files from to track.""",
        ),
        "eslint": attrs.dep(
            providers = [RunInfo],
            doc = """eslint dependency.""",
        ),
        "directories": attrs.list(
            attrs.string(),
            default = ["src", "test"],
            doc = """Directories under which to check.""",
        ),
        "extensions": attrs.list(
            attrs.string(),
            default = [".ts", ".js", ".cjs", ".vue"],
            doc = """File extensions to search for.""",
        ),
        "args": attrs.list(
            attrs.string(),
            default = [],
            doc = """Extra arguments passed to eslint.""",
        ),
        "warnings": attrs.bool(
            default = False,
            doc = """If `False`, then exit non-zero (treat warnings as errors).""",
        ),
        "package_node_modules": attrs.source(
            doc = """Target which builds package `node_modules`.""",
        ),
        "pnpm_exec_cmd_override": attrs.option(
            attrs.string(),
            default = None,
            doc = """Invoke a command via 'pnpm exec' rather than npm_bin script.""",
        ),
        "_python_toolchain": attrs.toolchain_dep(
            default = "toolchains//:python",
            providers = [PythonToolchainInfo],
        ),
        "_pnpm_toolchain": attrs.toolchain_dep(
            default = "toolchains//:pnpm",
            providers = [PnpmToolchainInfo],
        ),
    } | re_test_common.test_args() | inject_test_env.args(),
)

def ts_test_impl(ctx: AnalysisContext) -> list[[
    DefaultInfo,
    RunInfo,
    ExternalRunnerTestInfo,
]]:
    args = cmd_args()
    args.add(ctx.attrs.args)

    return _npm_test_impl(
        ctx,
        ctx.attrs.program[RunInfo],
        args,
        "ts_test",
    )

ts_test = rule(
    impl = ts_test_impl,
    attrs = {
        "srcs": attrs.list(
            attrs.source(),
            default = [],
            doc = """List of package source files to track.""",
        ),
        "prod_deps_srcs": attrs.dict(
            attrs.string(),
            attrs.source(allow_directory = True),
            default = {},
            doc = """Mapping of dependent prod package paths to source files to track.""",
        ),
        "dev_deps_srcs": attrs.dict(
            attrs.string(),
            attrs.source(allow_directory = True),
            default = {},
            doc = """Mapping of dependent dev package paths to source files from to track.""",
        ),
        "program": attrs.dep(
            providers = [RunInfo],
            doc = """test program to run.""",
        ),
        "args": attrs.list(
            attrs.string(),
            default = [],
            doc = """Extra arguments passed to test program.""",
        ),
        "package_node_modules": attrs.source(
            doc = """Target which builds package `node_modules`.""",
        ),
        "pnpm_exec_cmd_override": attrs.option(
            attrs.string(),
            default = None,
            doc = """Invoke a command via 'pnpm exec' rather than npm_bin script.""",
        ),
        "_python_toolchain": attrs.toolchain_dep(
            default = "toolchains//:python",
            providers = [PythonToolchainInfo],
        ),
        "_pnpm_toolchain": attrs.toolchain_dep(
            default = "toolchains//:pnpm",
            providers = [PnpmToolchainInfo],
        ),
    } | re_test_common.test_args() | inject_test_env.args(),
)

def typescript_check_impl(ctx: AnalysisContext) -> list[[
    DefaultInfo,
    RunInfo,
    ExternalRunnerTestInfo,
]]:
    args = cmd_args()
    args.add("--noEmit")
    args.add(ctx.attrs.args)

    return _npm_test_impl(
        ctx,
        ctx.attrs.tsc[RunInfo],
        args,
        "tsc",
    )

typescript_check = rule(
    impl = typescript_check_impl,
    attrs = {
        "srcs": attrs.list(
            attrs.source(),
            default = [],
            doc = """List of package source files to track.""",
        ),
        "prod_deps_srcs": attrs.dict(
            attrs.string(),
            attrs.source(allow_directory = True),
            default = {},
            doc = """Mapping of dependent prod package paths to source files to track.""",
        ),
        "dev_deps_srcs": attrs.dict(
            attrs.string(),
            attrs.source(allow_directory = True),
            default = {},
            doc = """Mapping of dependent dev package paths to source files from to track.""",
        ),
        "tsc": attrs.dep(
            providers = [RunInfo],
            doc = """tsc dependency.""",
        ),
        "args": attrs.list(
            attrs.string(),
            default = [],
            doc = """Extra arguments passed to tsc.""",
        ),
        "package_node_modules": attrs.source(
            doc = """Target which builds package `node_modules`.""",
        ),
        "pnpm_exec_cmd_override": attrs.option(
            attrs.string(),
            default = None,
            doc = """Invoke a command via 'pnpm exec' rather than npm_bin script.""",
        ),
        "_python_toolchain": attrs.toolchain_dep(
            default = "toolchains//:python",
            providers = [PythonToolchainInfo],
        ),
        "_pnpm_toolchain": attrs.toolchain_dep(
            default = "toolchains//:pnpm",
            providers = [PnpmToolchainInfo],
        ),
    } | re_test_common.test_args() | inject_test_env.args(),
)

def prettier_check_impl(ctx: AnalysisContext) -> list[[
    DefaultInfo,
    RunInfo,
    ExternalRunnerTestInfo,
]]:
    args = cmd_args()
    args.add("--check")
    args.add(ctx.attrs.args)
    args.add(".")

    return _npm_test_impl(
        ctx,
        ctx.attrs.prettier[RunInfo],
        args,
        "prettier",
    )

prettier_check = rule(
    impl = prettier_check_impl,
    attrs = {
        "srcs": attrs.list(
            attrs.source(),
            default = [],
            doc = """List of package source files to track.""",
        ),
        "prod_deps_srcs": attrs.dict(
            attrs.string(),
            attrs.source(allow_directory = True),
            default = {},
            doc = """Mapping of dependent prod package paths to source files to track.""",
        ),
        "dev_deps_srcs": attrs.dict(
            attrs.string(),
            attrs.source(allow_directory = True),
            default = {},
            doc = """Mapping of dependent dev package paths to source files from to track.""",
        ),
        "prettier": attrs.dep(
            providers = [RunInfo],
            doc = """prettier dependency.""",
        ),
        "args": attrs.list(
            attrs.string(),
            default = [],
            doc = """Extra arguments passed to prettier.""",
        ),
        "package_node_modules": attrs.source(
            doc = """Target which builds package `node_modules`.""",
        ),
        "pnpm_exec_cmd_override": attrs.option(
            attrs.string(),
            default = None,
            doc = """Invoke a command via 'pnpm exec' rather than npm_bin script.""",
        ),
        "_python_toolchain": attrs.toolchain_dep(
            default = "toolchains//:python",
            providers = [PythonToolchainInfo],
        ),
        "_pnpm_toolchain": attrs.toolchain_dep(
            default = "toolchains//:pnpm",
            providers = [PnpmToolchainInfo],
        ),
    } | re_test_common.test_args() | inject_test_env.args(),
)

def node_pkg_bin_impl(ctx: AnalysisContext) -> list[[DefaultInfo, RunInfo]]:
    bin_name = ctx.attrs.bin_name or ctx.attrs.name
    out = ctx.actions.declare_output(bin_name)

    pnpm_toolchain = ctx.attrs._pnpm_toolchain[PnpmToolchainInfo]
    package_dir = cmd_args(ctx.label.package).relative_to(ctx.label.cell_root)

    cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        pnpm_toolchain.build_pkg_bin[DefaultInfo].default_outputs,
        "--pkg-bin",
        ctx.attrs.pkg[RunInfo],
        "--package-dir",
        package_dir,
        "--package-node-modules-path",
        ctx.attrs.package_node_modules,
        "--dist-path",
        ctx.attrs.dist,
    )
    for src in ctx.attrs.extra_srcs:
        cmd.add(["--extra-src", src])
    cmd.add(out.as_output())

    ctx.actions.run(cmd, category = "pkg", local_only = True)

    return [
        DefaultInfo(default_output = out),
        RunInfo(out),
    ]

node_pkg_bin = rule(
    impl = node_pkg_bin_impl,
    attrs = {
        "pkg": attrs.dep(
            providers = [RunInfo],
            doc = """pkg dependency.""",
        ),
        "bin_name": attrs.option(
            attrs.string(),
            default = None,
            doc = """Output bin name (default: attrs.name).""",
        ),
        "dist": attrs.source(
            doc = """Target which which builds a `dist`.""",
        ),
        "package_node_modules": attrs.source(
            doc = """Target which builds package `node_modules`.""",
        ),
        "extra_srcs": attrs.list(
            attrs.source(),
            default = [],
            doc = "Additional file(s) needed to produce the binary",
        ),
        "_python_toolchain": attrs.toolchain_dep(
            default = "toolchains//:python",
            providers = [PythonToolchainInfo],
        ),
        "_pnpm_toolchain": attrs.toolchain_dep(
            default = "toolchains//:pnpm",
            providers = [PnpmToolchainInfo],
        ),
    },
)

def npm_bin_impl(ctx: AnalysisContext) -> list[[DefaultInfo, RunInfo, TemplatePlaceholderInfo]]:
    bin_name = ctx.attrs.bin_name or ctx.attrs.name

    exe = ctx.actions.declare_output(bin_name)

    pnpm_toolchain = ctx.attrs._pnpm_toolchain[PnpmToolchainInfo]

    cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        pnpm_toolchain.build_npm_bin[DefaultInfo].default_outputs,
        "--bin-out-path",
        exe.as_output(),
    )
    if not ctx.attrs.workspace:
        cmd.add("--package-dir")
        cmd.add(ctx.label.package)
    cmd.add([
        ctx.attrs.node_modules,
        bin_name,
    ])

    ctx.actions.run(cmd, category = "build_npm_bin", identifier = bin_name, local_only = True)

    return [
        DefaultInfo(default_output = exe),
        RunInfo(exe),
        TemplatePlaceholderInfo(
            keyed_variables = {
                "exe": exe,
            },
        ),
    ]

npm_bin = rule(
    impl = npm_bin_impl,
    attrs = {
        "bin_name": attrs.option(
            attrs.string(),
            default = None,
            doc = """Node module bin name (default: attrs.name).""",
        ),
        "workspace": attrs.bool(
            default = False,
            doc = """Whether the binary script is in the workspace root.""",
        ),
        "node_modules": attrs.source(
            doc = """Target which builds `node_modules`.""",
        ),
        "_python_toolchain": attrs.toolchain_dep(
            default = "toolchains//:python",
            providers = [PythonToolchainInfo],
        ),
        "_pnpm_toolchain": attrs.toolchain_dep(
            default = "toolchains//:pnpm",
            providers = [PnpmToolchainInfo],
        ),
    },
)

def package_node_modules_impl(ctx: AnalysisContext) -> list[DefaultInfo]:
    node_modules_ctx = node_modules_context(ctx, ctx.attrs.prod_only)
    return [DefaultInfo(default_output = node_modules_ctx.root)]

package_node_modules = rule(
    impl = package_node_modules_impl,
    attrs = {
        "turbo": attrs.dep(
            providers = [RunInfo],
            default = "root//third-party/node/turbo:turbo",
            doc = """Turbo dependency.""",
        ),
        "pnpm_lock": attrs.source(
            default = "root//:pnpm-lock.yaml",
            doc = """Workspace Pnpm lock file""",
        ),
        "package_name": attrs.option(
            attrs.string(),
            default = None,
            doc = "Explicitly set the npm package name if it differs from directory name",
        ),
        "root_workspace": attrs.bool(
            default = True,
        ),
        "prod_only": attrs.bool(
            default = False,
            doc = "Only install production dependencies",
        ),
        "_python_toolchain": attrs.toolchain_dep(
            default = "toolchains//:python",
            providers = [PythonToolchainInfo],
        ),
        "_pnpm_toolchain": attrs.toolchain_dep(
            default = "toolchains//:pnpm",
            providers = [PnpmToolchainInfo],
        ),
    },
)

def pnpm_lock_impl(ctx: AnalysisContext) -> list[DefaultInfo]:
    out = ctx.actions.declare_output("pnpm-lock.yaml")

    output = ctx.actions.copy_file(out, ctx.attrs.src)

    return [DefaultInfo(default_output = out)]

pnpm_lock = rule(
    impl = pnpm_lock_impl,
    attrs = {
        "src": attrs.source(
            doc = """pnpm-lock.yaml source.""",
        ),
        "pnpm_workspace": attrs.dep(
            default = "root//:pnpm-workspace.yaml",
            doc = """Pnpm Workspace dependency.""",
        ),
    },
)

def pnpm_workspace_impl(ctx: AnalysisContext) -> list[[DefaultInfo, ArtifactGroupInfo]]:
    out = ctx.actions.declare_output("pnpm-workspace.yaml")

    output = ctx.actions.copy_file(out, ctx.attrs.src)
    ctx.actions.write_json("member-packages.json", ctx.attrs.packages)

    return [
        DefaultInfo(default_output = output),
        ArtifactGroupInfo(artifacts = [ctx.attrs.workspace_package] + ctx.attrs.packages),
    ]

pnpm_workspace = rule(
    impl = pnpm_workspace_impl,
    attrs = {
        "src": attrs.source(
            doc = """pnpm-workspace.yaml source.""",
        ),
        "workspace_package": attrs.source(
            doc = """Workspace root package.json source.""",
        ),
        "packages": attrs.list(
            attrs.source(),
            default = [],
            doc = """List of package.json files to track.""",
        ),
    },
)

def typescript_dist_impl(ctx: AnalysisContext) -> list[[DefaultInfo, TypescriptDistInfo]]:
    out = ctx.actions.declare_output("dist", dir = True)

    if ctx.attrs.tsc and ctx.attrs.tsup:
        fail("Only one of `tsc` or `tsup` must be set")
    elif ctx.attrs.tsc:
        ts_compiler = ctx.attrs.tsc[RunInfo]
        index_file = "index.js"
    elif ctx.attrs.tsup:
        ts_compiler = ctx.attrs.tsup[RunInfo]
        index_file = "index.cjs"
    else:
        fail("Either `tsc` or `tsup` attribute is required")

    pnpm_toolchain = ctx.attrs._pnpm_toolchain[PnpmToolchainInfo]
    package_build_ctx = package_build_context(ctx)

    cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        pnpm_toolchain.exec_cmd[DefaultInfo].default_outputs,
        "--cwd",
        cmd_args([package_build_ctx.srcs_tree, ctx.label.package], delimiter = "/"),
        "--",
        cmd_args(ts_compiler, format = "{}::abspath"),
    )
    if ctx.attrs.tsup:
        cmd.add("--out-dir")
    else:
        cmd.add("--outDir")
    cmd.add(cmd_args(out.as_output(), format = "{}::relpath"))

    if ctx.attrs.args:
        cmd.append(ctx.attrs.args)

    ctx.actions.run(cmd, category = "tsc", local_only = True)

    return [
        DefaultInfo(default_output = out),
        TypescriptDistInfo(index_file = index_file),
    ]

typescript_dist = rule(
    impl = typescript_dist_impl,
    attrs = {
        "tsc": attrs.option(
            attrs.dep(providers = [RunInfo]),
            default = None,
            doc = """TypeScript compiler dependency.""",
        ),
        "tsup": attrs.option(
            attrs.dep(providers = [RunInfo]),
            default = None,
            doc = """tsup compiler dependency.""",
        ),
        "srcs": attrs.list(
            attrs.source(),
            default = [],
            doc = """List of package source files to track.""",
        ),
        "prod_deps_srcs": attrs.dict(
            attrs.string(),
            attrs.source(allow_directory = True),
            default = {},
            doc = """Mapping of dependent prod package paths to source files to track.""",
        ),
        "dev_deps_srcs": attrs.dict(
            attrs.string(),
            attrs.source(allow_directory = True),
            default = {},
            doc = """Mapping of dependent dev package paths to source files from to track.""",
        ),
        "args": attrs.option(
            attrs.list(attrs.arg()),
            default = None,
            doc = """Extra script arguments.""",
        ),
        "dist_dir": attrs.string(
            default = "dist",
            doc = """Output directory from the compilation.""",
        ),
        "package_node_modules": attrs.source(
            doc = """Target which builds package `node_modules`.""",
        ),
        "_python_toolchain": attrs.toolchain_dep(
            default = "toolchains//:python",
            providers = [PythonToolchainInfo],
        ),
        "_pnpm_toolchain": attrs.toolchain_dep(
            default = "toolchains//:pnpm",
            providers = [PnpmToolchainInfo],
        ),
    },
)

def typescript_runnable_dist_impl(ctx: AnalysisContext) -> list[[
    DefaultInfo,
    RunInfo,
    TypescriptRunnableDistInfo,
]]:
    runnable_dist_ctx = package_runnable_dist_context(ctx)
    out = runnable_dist_ctx.tree

    pnpm_toolchain = ctx.attrs._pnpm_toolchain[PnpmToolchainInfo]
    package_dir = cmd_args(ctx.label.package).relative_to(ctx.label.cell_root)

    bin = paths.join("bin", paths.basename(ctx.label.package))
    run_cmd = cmd_args(
        cmd_args([out, bin], delimiter = "/"),
    )

    return [
        DefaultInfo(default_output = out),
        RunInfo(run_cmd),
        TypescriptRunnableDistInfo(
            runnable_dist = out,
            bin = bin,
        ),
    ]

typescript_runnable_dist = rule(
    impl = typescript_runnable_dist_impl,
    attrs = {
        "typescript_dist": attrs.dep(
            providers = [TypescriptDistInfo],
            doc = """Target which builds the Typescript dist artifact.""",
        ),
        "package_node_modules_prod": attrs.dep(
            doc = """Target which builds package `node_modules` with prod-only modules.""",
        ),
        "_python_toolchain": attrs.toolchain_dep(
            default = "toolchains//:python",
            providers = [PythonToolchainInfo],
        ),
        "_pnpm_toolchain": attrs.toolchain_dep(
            default = "toolchains//:pnpm",
            providers = [PnpmToolchainInfo],
        ),
    },
)

def typescript_runnable_dist_bin_impl(ctx: AnalysisContext) -> list[[DefaultInfo, RunInfo]]:
    base_path = ctx.attrs.typescript_runnable_dist[DefaultInfo].default_outputs[0]
    bin = ctx.attrs.typescript_runnable_dist[TypescriptRunnableDistInfo].bin
    cd_path = cmd_args([base_path, paths.dirname(bin)], delimiter = "/")

    out = ctx.actions.declare_output(paths.basename(bin))

    pnpm_toolchain = ctx.attrs._pnpm_toolchain[PnpmToolchainInfo]

    cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        pnpm_toolchain.build_typescript_runnable_dist_bin[DefaultInfo].default_outputs,
        "--cd-path",
        cd_path,
        "--rel-path",
        paths.basename(bin),
        out.as_output(),
        hidden = [base_path],
    )

    ctx.actions.run(cmd, category = "pnpm", identifier = "typescript_runnable_dist_bin", local_only = True)

    return [
        DefaultInfo(default_output = out),
        RunInfo(out),
    ]

typescript_runnable_dist_bin = rule(
    impl = typescript_runnable_dist_bin_impl,
    attrs = {
        "typescript_runnable_dist": attrs.dep(
            doc = """Target which builds the runnable Typescript dist artifact.""",
        ),
        "_python_toolchain": attrs.toolchain_dep(
            default = "toolchains//:python",
            providers = [PythonToolchainInfo],
        ),
        "_pnpm_toolchain": attrs.toolchain_dep(
            default = "toolchains//:pnpm",
            providers = [PnpmToolchainInfo],
        ),
    },
)

def vite_app_impl(ctx: AnalysisContext) -> list[[DefaultInfo, RunInfo]]:
    out = ctx.actions.declare_output("dist", dir = True)

    pnpm_toolchain = ctx.attrs._pnpm_toolchain[PnpmToolchainInfo]
    package_build_ctx = package_build_context(ctx)

    if ctx.attrs.pnpm_exec_cmd_override:
        exec_cmd = cmd_args(
            "pnpm",
            "exec",
            ctx.attrs.pnpm_exec_cmd_override,
        )
    else:
        exec_cmd = cmd_args(ctx.attrs.vite[RunInfo], format = "{}::abspath")

    cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        pnpm_toolchain.exec_cmd[DefaultInfo].default_outputs,
        "--cwd",
        cmd_args([package_build_ctx.srcs_tree, ctx.label.package], delimiter = "/"),
        "--copy-tree",
        cmd_args(["dist", cmd_args(out.as_output(), format = "{}::abspath")], delimiter = "="),
        "--",
        exec_cmd,
        "build",
    )

    ctx.actions.run(cmd, category = "vite", identifier = "build", local_only = True)

    run_cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        pnpm_toolchain.exec_cmd[DefaultInfo].default_outputs,
        "--cwd",
        cmd_args([out, ".."], delimiter = "/"),
        "--",
        ctx.attrs.vite[RunInfo],
        "preview",
    )

    return [
        DefaultInfo(default_output = out),
        RunInfo(run_cmd),
    ]

vite_app = rule(
    impl = vite_app_impl,
    attrs = {
        "vite": attrs.dep(
            providers = [RunInfo],
            doc = """vite dependency.""",
        ),
        "srcs": attrs.list(
            attrs.source(),
            default = [],
            doc = """List of package source files to track.""",
        ),
        "prod_deps_srcs": attrs.dict(
            attrs.string(),
            attrs.source(allow_directory = True),
            default = {},
            doc = """Mapping of dependent prod package paths to source files to track.""",
        ),
        "dev_deps_srcs": attrs.dict(
            attrs.string(),
            attrs.source(allow_directory = True),
            default = {},
            doc = """Mapping of dependent dev package paths to source files from to track.""",
        ),
        "package_node_modules": attrs.source(
            doc = """Target which builds package `node_modules`.""",
        ),
        "pnpm_exec_cmd_override": attrs.option(
            attrs.string(),
            default = None,
            doc = """Invoke a command via 'pnpm exec' rather than npm_bin script.""",
        ),
        "_python_toolchain": attrs.toolchain_dep(
            default = "toolchains//:python",
            providers = [PythonToolchainInfo],
        ),
        "_pnpm_toolchain": attrs.toolchain_dep(
            default = "toolchains//:pnpm",
            providers = [PnpmToolchainInfo],
        ),
    },
)

def workspace_node_modules_impl(ctx: AnalysisContext) -> list[DefaultInfo]:
    out = ctx.actions.declare_output("root", dir = True)

    pnpm_toolchain = ctx.attrs._pnpm_toolchain[PnpmToolchainInfo]
    package_dir = cmd_args(ctx.label.package).relative_to(ctx.label.cell_root)

    cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        pnpm_toolchain.build_workspace_node_modules[DefaultInfo].default_outputs,
        hidden = [ctx.attrs.pnpm_lock],
    )
    if ctx.attrs.root_workspace:
        cmd.add("--package-dir")
        cmd.add(package_dir)
    else:
        cmd.add("--root-dir")
        cmd.add(package_dir)
    cmd.add(out.as_output())

    ctx.actions.run(cmd, category = "pnpm", identifier = "install", local_only = True)

    return [DefaultInfo(default_output = out)]

workspace_node_modules = rule(
    impl = workspace_node_modules_impl,
    attrs = {
        "pnpm_lock": attrs.source(
            default = "root//:pnpm-lock.yaml",
            doc = """Workspace Pnpm lock file""",
        ),
        "root_workspace": attrs.bool(
            default = True,
        ),
        "_python_toolchain": attrs.toolchain_dep(
            default = "toolchains//:python",
            providers = [PythonToolchainInfo],
        ),
        "_pnpm_toolchain": attrs.toolchain_dep(
            default = "toolchains//:pnpm",
            providers = [PnpmToolchainInfo],
        ),
    },
)

NodeModulesContext = record(
    root = field(Artifact),
)

def node_modules_context(
        ctx: AnalysisContext,
        prod_only: bool = False,
        out_dir: str = "root") -> NodeModulesContext:
    out = ctx.actions.declare_output(out_dir, dir = True)

    pnpm_toolchain = ctx.attrs._pnpm_toolchain[PnpmToolchainInfo]
    package_dir = cmd_args(ctx.label.package).relative_to(ctx.label.cell_root)

    cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        pnpm_toolchain.build_package_node_modules[DefaultInfo].default_outputs,
        "--turbo-bin",
        ctx.attrs.turbo[RunInfo],
        hidden = [ctx.attrs.pnpm_lock],
    )
    if ctx.attrs.package_name:
        cmd.add("--package-name")
        cmd.add(ctx.attrs.package_name)
    if ctx.attrs.root_workspace:
        cmd.add("--package-dir")
        cmd.add(package_dir)
    else:
        cmd.add("--root-dir")
        cmd.add(package_dir)
    if prod_only:
        cmd.add("--prod-only")
    cmd.add(out.as_output())

    ctx.actions.run(cmd, category = "pnpm", identifier = "install " + ctx.label.package, local_only = True)

    return NodeModulesContext(root = out)

PackageBuildContext = record(
    # Root of a workspace source tree containing a pruned sub-package and all node_modules
    srcs_tree = field(Artifact),
)

def package_build_context(ctx: AnalysisContext) -> PackageBuildContext:
    srcs_tree = ctx.actions.declare_output("__src")

    pnpm_toolchain = ctx.attrs._pnpm_toolchain[PnpmToolchainInfo]
    package_dir = cmd_args(ctx.label.package).relative_to(ctx.label.cell_root)

    cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        pnpm_toolchain.package_build_context[DefaultInfo].default_outputs,
        "--package-dir",
        package_dir,
        "--package-node-modules-path",
        ctx.attrs.package_node_modules,
    )
    for src in ctx.attrs.srcs:
        cmd.add("--src")
        cmd.add(cmd_args(src, format = ctx.label.package + "={}"))
    for (name, src) in ctx.attrs.prod_deps_srcs.items():
        cmd.add("--src")
        cmd.add(cmd_args(src, format = name + "={}"))
    for (name, src) in ctx.attrs.dev_deps_srcs.items():
        cmd.add("--src")
        cmd.add(cmd_args(src, format = name + "={}"))
    if pnpm_toolchain.editorconfig:
        cmd.add("--editorconfig")
        cmd.add(pnpm_toolchain.editorconfig)
    cmd.add(srcs_tree.as_output())

    ctx.actions.run(cmd, category = "package_build_context", local_only = True)

    return PackageBuildContext(
        srcs_tree = srcs_tree,
    )

PackageDistContext = record(
    tree = field(Artifact),
)

def package_runnable_dist_context(
        ctx: AnalysisContext,
        dist_path: [Artifact, None] = None) -> PackageDistContext:
    out = ctx.actions.declare_output("runnable_dist", dir = True)

    pnpm_toolchain = ctx.attrs._pnpm_toolchain[PnpmToolchainInfo]
    package_dir = cmd_args(ctx.label.package).relative_to(ctx.label.cell_root)

    if not dist_path:
        dist_path = ctx.attrs.typescript_dist[DefaultInfo].default_outputs[0]

    cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        pnpm_toolchain.package_dist_context[DefaultInfo].default_outputs,
        "--package-dir",
        package_dir,
        "--package-node-modules-path",
        ctx.attrs.package_node_modules_prod[DefaultInfo].default_outputs[0],
        "--dist-path",
        dist_path,
        "--index-file",
        ctx.attrs.typescript_dist[TypescriptDistInfo].index_file,
    )
    cmd.add(out.as_output())

    ctx.actions.run(cmd, category = "package_runnable_dist_context", local_only = True)

    return PackageDistContext(
        tree = out,
    )

def pnpm_task_library_impl(ctx: AnalysisContext) -> list[DefaultInfo]:
    script = ctx.actions.write("pnpm-run.sh", """\
#!/usr/bin/env bash
set -euo pipefail

rootpath="$(git rev-parse --show-toplevel)"
npm_package_path="$1"
npm_run_command="$2"
buck_out_directory="$3"
output_paths="$4"

mkdir -p "$buck_out_directory"
cd "$rootpath/$npm_package_path"
pnpm run --report-summary "$npm_run_command"
if [[ ! -z "$output_paths" ]]; then
  cp -vr "$output_paths" "$rootpath/$buck_out_directory/"
fi
""", is_executable = True)
    out = ctx.actions.declare_output("out", dir = True)
    output_join = " ".join(ctx.attrs.outs)
    args = cmd_args(
        [script, ctx.attrs.path, ctx.attrs.command, out.as_output(), output_join],
        hidden = [ctx.attrs.srcs, ctx.attrs.deps],
    )
    ctx.actions.run(args, category = "pnpm", identifier = "run_library", local_only = True)
    return [DefaultInfo(default_outputs = [out])]

pnpm_task_library = rule(impl = pnpm_task_library_impl, attrs = {
    "command": attrs.string(default = "start", doc = """pnpm command to run"""),
    "path": attrs.string(default = "path", doc = """the path to run the command from"""),
    "deps": attrs.list(attrs.source(), default = [], doc = """List of dependencies we require"""),
    "srcs": attrs.list(attrs.source(), default = [], doc = """List of sources we require"""),
    "outs": attrs.list(attrs.string(), default = [], doc = "List of outputs to copy"),
})

def pnpm_task_binary_impl(ctx: AnalysisContext) -> list[[DefaultInfo, RunInfo]]:
    script = ctx.actions.write("pnpm-run.sh", """\
#!/usr/bin/env bash
set -euo pipefail

rootpath="$(git rev-parse --show-toplevel)"
npm_package_path="$1"
npm_run_command="$2"

cd "$rootpath/$npm_package_path"
pnpm run --report-summary "$npm_run_command"
""", is_executable = True)
    args = cmd_args(
        [script, ctx.attrs.path, ctx.attrs.command],
        hidden = [ctx.attrs.deps, ctx.attrs.srcs],
    )
    return [DefaultInfo(), RunInfo(args = args)]

pnpm_task_binary = rule(impl = pnpm_task_binary_impl, attrs = {
    "command": attrs.string(default = "start", doc = """pnpm command to run"""),
    "path": attrs.string(default = "path", doc = """the path to run the comamnd from"""),
    "srcs": attrs.list(attrs.source(), default = [], doc = """List of sources we require"""),
    "deps": attrs.list(attrs.source(), default = [], doc = """List of dependencies we require"""),
})

def pnpm_task_test_impl(ctx: AnalysisContext) -> list[[DefaultInfo, ExternalRunnerTestInfo]]:
    script = ctx.actions.write("pnpm-run.sh", """\
#!/usr/bin/env bash
set -euo pipefail

rootpath="$(git rev-parse --show-toplevel)"
npm_package_path="$1"
npm_run_command="$2"

cd "$rootpath/$npm_package_path"
pnpm install
pnpm run --report-summary "$npm_run_command"
""", is_executable = True)
    args = cmd_args(
        [script, ctx.attrs.path, ctx.attrs.command],
        hidden = [ctx.attrs.deps, ctx.attrs.srcs],
    )
    return [DefaultInfo(), ExternalRunnerTestInfo(type = "integration", command = [script, ctx.attrs.path, ctx.attrs.command])]

pnpm_task_test = rule(impl = pnpm_task_test_impl, attrs = {
    "command": attrs.string(default = "start", doc = """pnpm command to run"""),
    "path": attrs.string(default = "path", doc = """the path to run the comamnd from"""),
    "srcs": attrs.list(attrs.source(), default = [], doc = """List of sources we require"""),
    "deps": attrs.list(attrs.source(), default = [], doc = """List of dependencies we require"""),
})
