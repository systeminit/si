load(":mise.bzl", "MiseInfo")

def _mise_buildscript_runner_impl(ctx: AnalysisContext):
    mise_info = ctx.attrs._mise_install[MiseInfo]
    original_runner_info = ctx.attrs._original_runner[RunInfo]

    # The original runner's arguments already contain the python script
    # and all its arguments with the correct relative paths.
    # We just need to prefix it with `mise exec --`.
    return [
        RunInfo(
            args = cmd_args(
                mise_info.mise_bootstrap,
                "exec",
                "--",
                original_runner_info.args,
            ),
        ),
        # Pass through the DefaultInfo from the original runner so that
        # other rules can depend on the outputs of the build script.
        DefaultInfo(
            default_outputs = ctx.attrs._original_runner[DefaultInfo].default_outputs,
        ),
    ]

mise_buildscript_runner = rule(
    impl = _mise_buildscript_runner_impl,
    attrs = {
        "_mise_install": attrs.dep(
            providers = [MiseInfo],
            default = "toolchains//:rust_compiler",
        ),
        # We need both RunInfo and DefaultInfo from the original runner.
        "_original_runner": attrs.exec_dep(
            providers = [RunInfo, DefaultInfo],
            default = "prelude//rust/tools:buildscript_run",
        ),
    },
)
