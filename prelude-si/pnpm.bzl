## PNPM Install
def pnpm_install_impl(ctx: "context") -> ["provider"]:
    script = ctx.actions.write("pnpm-install.sh", """\
#!/usr/bin/env bash
set -euo pipefail

node_modules_out="$1"
pnpm_lock_out="$2"

pnpm install
cp -vr node_modules/ "$node_modules_out"
cp -v pnpm-lock.yaml "$pnpm_lock_out"
""", is_executable = True);
    node_modules = ctx.actions.declare_output("node_modules", dir = True)
    pnpm_lockfile = ctx.actions.declare_output("pnpm-lock.yaml")

    args = cmd_args([script, node_modules.as_output(), pnpm_lockfile.as_output(),])
    args.hidden([ctx.attrs.srcs])
    ctx.actions.run(args, category = "pnpm", identifier = "install", local_only = True)
    return [DefaultInfo(default_outputs = [node_modules, pnpm_lockfile,])]

pnpm_install = rule(impl = pnpm_install_impl, attrs = {
    "srcs": attrs.list(attrs.source(), default = [], doc = """List of package.json files to track"""),
})

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
