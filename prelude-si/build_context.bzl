load("@prelude//python:toolchain.bzl", "PythonToolchainInfo")
load("//build_context:toolchain.bzl", "BuildContextToolchainInfo")

BuildContext = record(
    root = field(Artifact),
)

def build_context(
        ctx: AnalysisContext,
        build_deps: list[Dependency],
        srcs: dict[Artifact, str]) -> BuildContext:
    context_tree = ctx.actions.declare_output("__build_context")

    build_context_toolchain = ctx.attrs._build_context_toolchain[BuildContextToolchainInfo]

    cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        build_context_toolchain.build_context[DefaultInfo].default_outputs,
        "--bxl-file",
        build_context_toolchain.build_context_srcs_from_deps[DefaultInfo].default_outputs,
        "--bxl-script",
        "build_context_srcs_from_deps",
    )
    for src, rel_path in srcs.items():
        cmd.add("--src")
        cmd.add(cmd_args(src, format = "{}=" + rel_path))
    for dep in build_deps or []:
        cmd.add("--dep")
        cmd.add(dep.label.raw_target())
    cmd.add(context_tree.as_output())

    ctx.actions.run(cmd, category = "build_context", prefer_local = True)

    return BuildContext(
        root = context_tree,
    )
