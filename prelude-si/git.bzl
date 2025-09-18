load("@prelude//python:toolchain.bzl", "PythonToolchainInfo")
load("//git:toolchain.bzl", "GitToolchainInfo")

GitInfo = provider(fields = {
    "file": provider_field(typing.Any, default = None),  # [Artifact]
})

def git_info(ctx: AnalysisContext) -> GitInfo:
    json_file = ctx.actions.declare_output("git_info.json")

    git_toolchain = ctx.attrs._git_toolchain[GitToolchainInfo]

    cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        git_toolchain.git_info[DefaultInfo].default_outputs,
        json_file.as_output(),
    )
    ctx.actions.run(cmd, category = "git_info", prefer_local = True)

    return GitInfo(
        file = json_file,
    )
