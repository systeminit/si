def si_typescript_bin_impl(ctx: "context") -> ["provider"]:
    return [DefaultInfo()]

si_typescript_bin = rule(impl = si_typescript_bin_impl, attrs = {
  "deps": attrs.list(attrs.dep()),
  "out": attrs.string(),
})

def pnpm_install_impl(ctx: "context") -> ["provider"]:
    script = ctx.actions.write("pnpm-install.sh", """#!/bin/bash
set -euxo pipefail
pnpm install 
cp -r node_modules/ $1
cp pnpm-lock.yaml $2
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

def pnpm_run_binary_impl(ctx: "context") -> ["provider"]:
    script = ctx.actions.write("pnpm-run.sh", """#!/bin/bash
set -euxo pipefail
cd $1
pnpm run $2
""", is_executable = True);
    args = cmd_args([script, ctx.attrs.path, ctx.attrs.command])
    args.hidden([ctx.attrs.deps])
    args.hidden([ctx.attrs.srcs])
    return [DefaultInfo(), RunInfo(args = args)]

pnpm_run_binary = rule(impl = pnpm_run_binary_impl, attrs = {
    "command": attrs.string(default = "start", doc = """pnpm command to run"""),
    "path": attrs.string(default = "path", doc = """the path to run the comamnd from"""),
    "srcs": attrs.list(attrs.source(), default = [], doc = """List of sources we require"""),
    "deps": attrs.list(attrs.source(), default = [], doc = """List of dependencies we require"""),
})

def pnpm_run_library_impl(ctx: "context") -> ["provider"]:
    script = ctx.actions.write("pnpm-run.sh", """#!/bin/bash
set -euxo pipefail
pushd $1
pnpm run $2
popd
cp -r $1/$3 $4
""", is_executable = True);
    out = ctx.actions.declare_output(ctx.attrs.out, dir = True)
    args = cmd_args([script, ctx.attrs.path, ctx.attrs.command, ctx.attrs.out, out.as_output()])
    args.hidden([ctx.attrs.srcs])
    args.hidden([ctx.attrs.deps])
    ctx.actions.run(args, category = "pnpm", identifier = "run_library", local_only = True)
    return [DefaultInfo(default_outputs=[out])]

pnpm_run_library = rule(impl = pnpm_run_library_impl, attrs = {
    "command": attrs.string(default = "start", doc = """pnpm command to run"""),
    "path": attrs.string(default = "path", doc = """the path to run the comamnd from"""),
    "deps": attrs.list(attrs.source(), default = [], doc = """List of dependencies we require"""),
    "srcs": attrs.list(attrs.source(), default = [], doc = """List of sources we require"""),
    "out": attrs.string(default = "dist"),
})
