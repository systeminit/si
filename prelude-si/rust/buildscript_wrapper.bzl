load("@prelude-si//:mise.bzl", "MiseInfo")


def _mise_buildscript_runner_impl(ctx: AnalysisContext):
    mise_info = ctx.attrs._mise_install[MiseInfo]
    original_runner_info = ctx.attrs._original_runner[RunInfo]

    return [
        RunInfo(args=cmd_args(
            mise_info.mise_bootstrap,
            "exec",
            "--",
            original_runner_info.args,
        ), ),
        DefaultInfo(default_outputs=ctx.attrs._original_runner[DefaultInfo].
                    default_outputs, ),
    ]


mise_buildscript_runner = rule(
    impl=_mise_buildscript_runner_impl,
    attrs={
        "_mise_install":
        attrs.dep(
            providers=[MiseInfo],
            default="toolchains//:rust_compiler",
        ),
        "_original_runner":
        attrs.exec_dep(
            providers=[RunInfo, DefaultInfo],
            default="prelude//rust/tools:buildscript_run",
        ),
    },
)
