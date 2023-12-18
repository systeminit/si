load(
    "@prelude-si//:pnpm.bzl",
    _eslint = "eslint",
    _ts_test = "ts_test",
    _node_pkg_bin = "node_pkg_bin",
    _npm_bin = "npm_bin",
    _package_node_modules = "package_node_modules",
    _pnpm_lock = "pnpm_lock",
    _pnpm_workspace = "pnpm_workspace",
    _prettier_check = "prettier_check",
    _typescript_check = "typescript_check",
    _typescript_dist = "typescript_dist",
    _typescript_runnable_dist = "typescript_runnable_dist",
    _typescript_runnable_dist_bin = "typescript_runnable_dist_bin",
    _vite_app = "vite_app",
    _workspace_node_modules = "workspace_node_modules",
)

def eslint(
        eslint_bin = "eslint",
        directories = ["src"],
        package_node_modules = ":node_modules",
        visibility = ["PUBLIC"],
        **kwargs):
    if not rule_exists(eslint_bin):
        _npm_bin(
            name = eslint_bin,
            node_modules = package_node_modules,
            visibility = visibility,
        )

    _eslint(
        eslint = ":{}".format(eslint_bin),
        directories = directories,
        package_node_modules = package_node_modules,
        visibility = visibility,
        **kwargs
    )

def ts_test(
        program = ":vitest",
        args = ["run"],
        package_node_modules = ":node_modules",
        visibility = ["PUBLIC"],
        **kwargs):

    npm_bin_name = program.replace(":","")
    if not rule_exists(npm_bin_name):
        _npm_bin(
            name = npm_bin_name,
            node_modules = package_node_modules,
            visibility = visibility,
        )

    _ts_test(
        program = program,
        args = args,
        package_node_modules = package_node_modules,
        visibility = visibility,
        **kwargs
    )

def node_pkg_bin(
        pkg_bin = "pkg",
        dist = ":dist",
        package_node_modules = ":node_modules",
        visibility = ["PUBLIC"],
        **kwargs):
    if not rule_exists(pkg_bin):
        _npm_bin(
            name = pkg_bin,
            node_modules = package_node_modules,
            visibility = visibility,
        )

    _node_pkg_bin(
        pkg = ":{}".format(pkg_bin),
        dist = dist,
        package_node_modules = package_node_modules,
        visibility = visibility,
        **kwargs
    )

def npm_bin(
        visibility = ["PUBLIC"],
        node_modules = ":node_modules",
        **kwargs):
    _npm_bin(
        node_modules = node_modules,
        visibility = visibility,
        **kwargs
    )

def package_node_modules(
        visibility = ["PUBLIC"],
        **kwargs):
    _package_node_modules(visibility = visibility, **kwargs)

def pnpm_lock(
        name,
        src = None,
        visibility = ["PUBLIC"],
        **kwargs):
    _pnpm_lock(
        name = name,
        src = src or name,
        visibility = visibility,
        **kwargs
    )

def pnpm_workspace(
        name,
        src = None,
        workspace_package = ":package.json",
        visibility = ["PUBLIC"],
        **kwargs):
    _pnpm_workspace(
        name = name,
        src = src or name,
        workspace_package = workspace_package,
        visibility = visibility,
        **kwargs
    )

def prettier_check(
        prettier_bin = "prettier",
        prettier = ":prettier",
        package_node_modules = ":node_modules",
        visibility = ["PUBLIC"],
        **kwargs):
    if not rule_exists(prettier_bin):
        _npm_bin(
            name = prettier_bin,
            node_modules = package_node_modules,
            visibility = visibility,
        )

    _prettier_check(
        prettier = ":{}".format(prettier_bin),
        package_node_modules = package_node_modules,
        visibility = visibility,
        **kwargs
    )

def typescript_check(
        tsc_bin = "tsc",
        tsc = ":tsc",
        package_node_modules = ":node_modules",
        visibility = ["PUBLIC"],
        **kwargs):
    if not rule_exists(tsc_bin):
        _npm_bin(
            name = tsc_bin,
            node_modules = package_node_modules,
            visibility = visibility,
        )

    _typescript_check(
        tsc = ":{}".format(tsc_bin),
        package_node_modules = package_node_modules,
        visibility = visibility,
        **kwargs
    )

def typescript_dist(
        package_node_modules = ":node_modules",
        visibility = ["PUBLIC"],
        **kwargs):
    _typescript_dist(
        package_node_modules = package_node_modules,
        visibility = visibility,
        **kwargs
    )

def typescript_runnable_dist(
        typescript_dist = ":dist",
        package_node_modules_prod = ":node_modules_prod",
        visibility = ["PUBLIC"],
        **kwargs):
    if not rule_exists(package_node_modules_prod.replace(":", "")):
        _package_node_modules(
            name = package_node_modules_prod.replace(":", ""),
            prod_only = True,
            visibility = visibility,
        )

    _typescript_runnable_dist(
        typescript_dist = typescript_dist,
        package_node_modules_prod = package_node_modules_prod,
        visibility = visibility,
        **kwargs
    )

def typescript_runnable_dist_bin(
        visibility = ["PUBLIC"],
        **kwargs):
    _typescript_runnable_dist_bin(
        visibility = visibility,
        **kwargs
    )

def vite_app(
        vite_bin = "vite",
        package_node_modules = ":node_modules",
        visibility = ["PUBLIC"],
        **kwargs):
    if not rule_exists(vite_bin):
        _npm_bin(
            name = vite_bin,
            node_modules = package_node_modules,
            visibility = visibility,
        )

    _vite_app(
        vite = ":{}".format(vite_bin),
        package_node_modules = package_node_modules,
        visibility = visibility,
        **kwargs
    )

def workspace_node_modules(
        visibility = ["PUBLIC"],
        **kwargs):
    _workspace_node_modules(visibility = visibility, **kwargs)
