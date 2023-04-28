## PNPM Install
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

def pnpm_task_library_impl(ctx: "context") -> ["provider"]:
    script = ctx.actions.write("pnpm-run.sh", """#!/bin/bash
set -euxo pipefail
ROOTPATH=$PWD
NPM_PACKAGE_PATH=$1
NPM_RUN_COMMAND=$2
BUCK_OUT_DIRECTORY=$3
OUTPUT_PATHS=$4
mkdir -p $BUCK_OUT_DIRECTORY
pushd $NPM_PACKAGE_PATH
pnpm run $NPM_RUN_COMMAND | tee $ROOTPATH/$3/command-output.txt
if [ ! -z "$OUTPUT_PATHS" ]; then
  cp -r $OUTPUT_PATHS $ROOTPATH/$3
fi
popd
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
    #out = ctx.actions.declare_output("command-output.txt")
    script = ctx.actions.write("pnpm-run.sh", """#!/bin/bash
set -euxo pipefail
ROOTPATH=$PWD
NPM_PACKAGE_PATH=$1
NPM_RUN_COMMAND=$2
pushd $NPM_PACKAGE_PATH
pnpm run $NPM_RUN_COMMAND 
popd
""", is_executable = True);
    args = cmd_args([script, ctx.attrs.path, ctx.attrs.command])
    args.hidden([ctx.attrs.deps])
    args.hidden([ctx.attrs.srcs])
    # ctx.actions.run(args, category = "pnpm", identifier = "run")
    return [DefaultInfo(), RunInfo(args = args)]

pnpm_task_binary = rule(impl = pnpm_task_binary_impl, attrs = {
    "command": attrs.string(default = "start", doc = """pnpm command to run"""),
    "path": attrs.string(default = "path", doc = """the path to run the comamnd from"""),
    "srcs": attrs.list(attrs.source(), default = [], doc = """List of sources we require"""),
    "deps": attrs.list(attrs.source(), default = [], doc = """List of dependencies we require"""),
})

def pnpm_task_test_impl(ctx: "context") -> ["provider"]:
    #out = ctx.actions.declare_output("command-output.txt")
    script = ctx.actions.write("pnpm-run.sh", """#!/bin/bash
set -euxo pipefail
ROOTPATH=$PWD
NPM_PACKAGE_PATH=$1
NPM_RUN_COMMAND=$2
pushd $NPM_PACKAGE_PATH
pnpm run $NPM_RUN_COMMAND 
popd
""", is_executable = True);
    args = cmd_args([script, ctx.attrs.path, ctx.attrs.command])
    args.hidden([ctx.attrs.deps])
    args.hidden([ctx.attrs.srcs])
    # ctx.actions.run(args, category = "pnpm", identifier = "run")
    return [DefaultInfo(), ExternalRunnerTestInfo(type = "integration", command = [script, ctx.attrs.path, ctx.attrs.command])]

pnpm_task_test = rule(impl = pnpm_task_test_impl, attrs = {
    "command": attrs.string(default = "start", doc = """pnpm command to run"""),
    "path": attrs.string(default = "path", doc = """the path to run the comamnd from"""),
    "srcs": attrs.list(attrs.source(), default = [], doc = """List of sources we require"""),
    "deps": attrs.list(attrs.source(), default = [], doc = """List of dependencies we require"""),
})
