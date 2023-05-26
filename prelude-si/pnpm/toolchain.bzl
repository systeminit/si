PnpmToolchainInfo = provider(fields = [
    "build_npm_bin",
    "build_package_node_modules",
    "build_pkg_bin",
    "build_pnpm_lock",
    "build_workspace_node_modules",
    "exec_cmd",
    "package_build_context",
    "run_pnpm_script",
])

def pnpm_toolchain_impl(ctx) -> [[DefaultInfo.type, PnpmToolchainInfo.type]]:
    """
    A Pnpm toolchain.
    """
    return [
        DefaultInfo(),
        PnpmToolchainInfo(
            build_npm_bin = ctx.attrs._build_npm_bin,
            build_package_node_modules = ctx.attrs._build_package_node_modules,
            build_pkg_bin = ctx.attrs._build_pkg_bin,
            build_pnpm_lock = ctx.attrs._build_pnpm_lock,
            build_workspace_node_modules = ctx.attrs._build_workspace_node_modules,
            exec_cmd = ctx.attrs._exec_cmd,
            package_build_context = ctx.attrs._package_build_context,
            run_pnpm_script = ctx.attrs._run_pnpm_script,
        )
    ]

pnpm_toolchain = rule(
    impl = pnpm_toolchain_impl,
    attrs = {
        "_build_npm_bin": attrs.dep(
            default = "prelude-si//pnpm:build_npm_bin.py",
        ),
        "_build_package_node_modules": attrs.dep(
            default = "prelude-si//pnpm:build_package_node_modules.py",
        ),
        "_build_pkg_bin": attrs.dep(
            default = "prelude-si//pnpm:build_pkg_bin.py",
        ),
        "_build_pnpm_lock": attrs.dep(
            default = "prelude-si//pnpm:build_pnpm_lock.py",
        ),
        "_build_workspace_node_modules": attrs.dep(
            default = "prelude-si//pnpm:build_workspace_node_modules.py",
        ),
        "_exec_cmd": attrs.dep(
            default = "prelude-si//pnpm:exec_cmd.py",
        ),
        "_package_build_context": attrs.dep(
            default = "prelude-si//pnpm:package_build_context.py",
        ),
        "_run_pnpm_script": attrs.dep(
            default = "prelude-si//pnpm:run_pnpm_script.py",
        ),
    },
    is_toolchain_rule = True,
)
