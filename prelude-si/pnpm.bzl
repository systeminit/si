load("@prelude//python:toolchain.bzl", "PythonToolchainInfo")
load("//pnpm:toolchain.bzl", "PnpmToolchainInfo")

def node_pkg_bin_impl(ctx: "context") -> [[DefaultInfo.type, RunInfo.type]]:
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

    ctx.actions.run(cmd, category = "pkg")

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
            doc = "Additional file(s) needed to produce the binary"
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

def npm_bin_impl(ctx: "context") -> [[DefaultInfo.type, RunInfo.type, TemplatePlaceholderInfo.type]]:
    bin_name = ctx.attrs.bin_name or ctx.attrs.name

    exe = ctx.actions.declare_output(bin_name)

    pnpm_toolchain = ctx.attrs._pnpm_toolchain[PnpmToolchainInfo]

    cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        pnpm_toolchain.build_npm_bin[DefaultInfo].default_outputs,
        "--bin-out-path",
        exe.as_output(),
    )
    if ctx.attrs.workspace:
        cmd.add("--workspace")
    cmd.add([
        ctx.attrs.node_modules,
        bin_name,
    ])

    ctx.actions.run(cmd, category = "build_npm_bin", identifier = bin_name)

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

def package_node_modules_impl(ctx: "context") -> ["provider"]:
    out = ctx.actions.declare_output("root", dir = True)

    pnpm_toolchain = ctx.attrs._pnpm_toolchain[PnpmToolchainInfo]
    package_dir = cmd_args(ctx.label.package).relative_to(ctx.label.cell_root)

    cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        pnpm_toolchain.build_package_node_modules[DefaultInfo].default_outputs,
        "--turbo-bin",
        ctx.attrs.turbo[RunInfo],
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
    cmd.add(out.as_output())
    cmd.hidden([ctx.attrs.pnpm_lock])

    ctx.actions.run(cmd, category = "pnpm", identifier = "install " + ctx.label.package)

    return [DefaultInfo(default_output = out)]

package_node_modules = rule(
    impl = package_node_modules_impl,
    attrs = {
        "turbo": attrs.dep(
            providers = [RunInfo],
            default = "//third-party/node/turbo:turbo",
            doc = """Turbo dependency.""",
        ),
        "pnpm_lock": attrs.source(
            default = "//:pnpm-lock.yaml",
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

def pnpm_lock_impl(ctx: "context") -> ["provider"]:
    pnpm_lock = ctx.actions.declare_output("pnpm-lock.yaml")

    pnpm_toolchain = ctx.attrs._pnpm_toolchain[PnpmToolchainInfo]
    package_dir = cmd_args(ctx.label.package).relative_to(ctx.label.cell_root)

    cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        pnpm_toolchain.build_pnpm_lock[DefaultInfo].default_outputs,
        "--package-dir",
        package_dir,
        "--pnpm-lock-out-path",
        pnpm_lock.as_output(),
    )
    cmd.hidden([
        ctx.attrs.package_json,
        ctx.attrs.pnpm_workspace,
        ctx.attrs.packages,
    ])

    ctx.actions.run(cmd, category = "pnpm", identifier = "install --lockfile-only")

    return [DefaultInfo(default_outputs = [pnpm_lock])]

pnpm_lock = rule(
    impl = pnpm_lock_impl,
    attrs = {
        "package_json": attrs.source(
            default = "//:package.json",
            doc = """Workspace package.json""",
        ),
        "pnpm_workspace": attrs.source(
            default = "//:pnpm-workspace.yaml",
            doc = """Workspace Pnpm file""",
        ),
        "packages": attrs.list(
            attrs.source(),
            default = [],
            doc = """List of package.json files to track.""",
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

def typescript_dist_impl(ctx: "context") -> [DefaultInfo.type]:
    out = ctx.actions.declare_output("dist", dir = True)

    pnpm_toolchain = ctx.attrs._pnpm_toolchain[PnpmToolchainInfo]
    package_build_ctx = package_build_context(ctx)

    cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        pnpm_toolchain.exec_cmd[DefaultInfo].default_outputs,
        "--cwd",
        cmd_args([package_build_ctx.srcs_tree, ctx.label.package], delimiter = "/"),
        "--",
        cmd_args(ctx.attrs.tsc[RunInfo], format = "{}::abspath"),
        "--outDir",
        cmd_args(out.as_output(), format = "{}::relpath"),
    )
    if ctx.attrs.args:
        cmd.append(ctx.attrs.args)

    ctx.actions.run(cmd, category = "tsc")

    return [DefaultInfo(default_outputs = [out])]

typescript_dist = rule(
    impl = typescript_dist_impl,
    attrs = {
        "tsc": attrs.dep(
            providers = [RunInfo],
            doc = """TypeScript compiler dependency.""",
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
    }
)

def vite_app_impl(ctx: "context") -> [[DefaultInfo.type, RunInfo.type]]:
    out = ctx.actions.declare_output("dist", dir = True)

    pnpm_toolchain = ctx.attrs._pnpm_toolchain[PnpmToolchainInfo]
    package_build_ctx = package_build_context(ctx)

    cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        pnpm_toolchain.exec_cmd[DefaultInfo].default_outputs,
        "--cwd",
        cmd_args([package_build_ctx.srcs_tree, ctx.label.package], delimiter = "/"),
        "--",
        cmd_args(ctx.attrs.vite[RunInfo], format = "{}::abspath"),
        "build",
        "--outDir",
        cmd_args(out.as_output(), format = "{}::relpath"),
    )

    ctx.actions.run(cmd, category = "vite", identifier = "build")

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

def workspace_node_modules_impl(ctx: "context") -> ["provider"]:
    out = ctx.actions.declare_output("root", dir = True)

    pnpm_toolchain = ctx.attrs._pnpm_toolchain[PnpmToolchainInfo]
    package_dir = cmd_args(ctx.label.package).relative_to(ctx.label.cell_root)

    cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        pnpm_toolchain.build_workspace_node_modules[DefaultInfo].default_outputs,
    )
    if ctx.attrs.root_workspace:
        cmd.add("--package-dir")
        cmd.add(package_dir)
    else:
        cmd.add("--root-dir")
        cmd.add(package_dir)
    cmd.add(out.as_output())
    cmd.hidden([ctx.attrs.pnpm_lock])

    ctx.actions.run(cmd, category = "pnpm", identifier = "install")

    return [DefaultInfo(default_output = out)]

workspace_node_modules = rule(
    impl = workspace_node_modules_impl,
    attrs = {
        "pnpm_lock": attrs.source(
            default = "//:pnpm-lock.yaml",
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

PackageBuildContext = record(
    # Root of a workspace source tree containing a pruned sub-package and all node_modules
    srcs_tree = field("artifact"),
)

def package_build_context(ctx: "context") -> PackageBuildContext.type:
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
    cmd.add(srcs_tree.as_output())

    ctx.actions.run(cmd, category = "package_build_context")

    return PackageBuildContext(
        srcs_tree = srcs_tree,
    )

def pnpm_task_library_impl(ctx: "context") -> ["provider"]:
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
""", is_executable = True);
    out = ctx.actions.declare_output("out", dir = True)
    output_join = " ".join(ctx.attrs.outs)
    args = cmd_args([script, ctx.attrs.path, ctx.attrs.command, out.as_output(), output_join])
    args.hidden([ctx.attrs.srcs])
    args.hidden([ctx.attrs.deps])
    ctx.actions.run(args, category = "pnpm", identifier = "run_library", local_only = True)
    return [DefaultInfo(default_outputs=[out])]

pnpm_task_library = rule(impl = pnpm_task_library_impl, attrs = {
    "command": attrs.string(default = "start", doc = """pnpm command to run"""),
    "path": attrs.string(default = "path", doc = """the path to run the command from"""),
    "deps": attrs.list(attrs.source(), default = [], doc = """List of dependencies we require"""),
    "srcs": attrs.list(attrs.source(), default = [], doc = """List of sources we require"""),
    "outs": attrs.list(attrs.string(), default = [], doc = "List of outputs to copy"),
})

def pnpm_task_binary_impl(ctx: "context") -> ["provider"]:
    script = ctx.actions.write("pnpm-run.sh", """\
#!/usr/bin/env bash
set -euo pipefail

rootpath="$(git rev-parse --show-toplevel)"
npm_package_path="$1"
npm_run_command="$2"

cd "$rootpath/$npm_package_path"
pnpm run --report-summary "$npm_run_command"
""", is_executable = True);
    args = cmd_args([script, ctx.attrs.path, ctx.attrs.command])
    args.hidden([ctx.attrs.deps])
    args.hidden([ctx.attrs.srcs])
    return [DefaultInfo(), RunInfo(args = args)]

pnpm_task_binary = rule(impl = pnpm_task_binary_impl, attrs = {
    "command": attrs.string(default = "start", doc = """pnpm command to run"""),
    "path": attrs.string(default = "path", doc = """the path to run the comamnd from"""),
    "srcs": attrs.list(attrs.source(), default = [], doc = """List of sources we require"""),
    "deps": attrs.list(attrs.source(), default = [], doc = """List of dependencies we require"""),
})

def pnpm_task_test_impl(ctx: "context") -> ["provider"]:
    script = ctx.actions.write("pnpm-run.sh", """\
#!/usr/bin/env bash
set -euo pipefail

rootpath="$(git rev-parse --show-toplevel)"
npm_package_path="$1"
npm_run_command="$2"

cd "$rootpath/$npm_package_path"
pnpm run --report-summary "$npm_run_command"
""", is_executable = True);
    args = cmd_args([script, ctx.attrs.path, ctx.attrs.command])
    args.hidden([ctx.attrs.deps])
    args.hidden([ctx.attrs.srcs])
    return [DefaultInfo(), ExternalRunnerTestInfo(type = "integration", command = [script, ctx.attrs.path, ctx.attrs.command])]

pnpm_task_test = rule(impl = pnpm_task_test_impl, attrs = {
    "command": attrs.string(default = "start", doc = """pnpm command to run"""),
    "path": attrs.string(default = "path", doc = """the path to run the comamnd from"""),
    "srcs": attrs.list(attrs.source(), default = [], doc = """List of sources we require"""),
    "deps": attrs.list(attrs.source(), default = [], doc = """List of dependencies we require"""),
})
