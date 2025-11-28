DenoToolchainInfo = provider(fields = {
    "deno_exe": provider_field(typing.Any, default = None),
    "target_string": provider_field(typing.Any, default = None),
    "deno_binary": provider_field(typing.Any, default = None),
    "deno_format": provider_field(typing.Any, default = None),
    "deno_run": provider_field(typing.Any, default = None),
    "deno_test": provider_field(typing.Any, default = None),
    "deno_workspace": provider_field(typing.Any, default = None),
    "deno_target_runtime": provider_field(typing.Any, default = None),
})

def deno_toolchain_impl(ctx) -> list[[DefaultInfo, DenoToolchainInfo]]:
    """
    A deno toolchain.
    """
    return [
        DefaultInfo(default_outputs = []),
        DenoToolchainInfo(
            deno_exe = ctx.attrs.deno_exe,
            target_string = ctx.attrs.target_string,
            deno_binary = ctx.attrs._deno_binary,
            deno_format = ctx.attrs._deno_format,
            deno_run = ctx.attrs._deno_run,
            deno_test = ctx.attrs._deno_test,
            deno_workspace = ctx.attrs._deno_workspace,
            deno_target_runtime = ctx.attrs._deno_target_runtime,
        ),
    ]

deno_toolchain = rule(
    impl = deno_toolchain_impl,
    attrs = {
        "deno_exe": attrs.arg(),
        "target_string": attrs.string(default = ""),
        "_deno_binary": attrs.dep(
            default = "prelude-si//deno:deno_binary.py",
            providers = [DefaultInfo],
        ),
       "_deno_format": attrs.dep(
            default = "prelude-si//deno:deno_format.py",
            providers = [DefaultInfo],
        ),
       "_deno_run": attrs.dep(
            default = "prelude-si//deno:deno_run.py",
            providers = [DefaultInfo],
        ),
       "_deno_test": attrs.dep(
            default = "prelude-si//deno:deno_test.py",
            providers = [DefaultInfo],
        ),
       "_deno_workspace": attrs.dep(
            default = "prelude-si//deno:deno_workspace.py",
            providers = [DefaultInfo],
        ),
        "_deno_target_runtime": attrs.dep(
            default = "prelude-si//deno:deno_target_runtime.py",
            providers = [DefaultInfo],
        ),
    },
    is_toolchain_rule = True,
)

DenoWorkspaceInfo = provider(
    fields = {
        "workspace_dir": provider_field(typing.Any, default = None),
        "cache_dir": provider_field(typing.Any, default = None),
    },
)
