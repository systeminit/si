load(
    "@prelude-si//:pnpm.bzl",
    _node_pkg_bin = "node_pkg_bin",
    _npm_bin = "npm_bin",
    _package_node_modules = "package_node_modules",
    _pnpm_lock = "pnpm_lock",
    _typescript_dist = "typescript_dist",
    _vite_app = "vite_app",
    _workspace_node_modules = "workspace_node_modules",
)

def node_pkg_bin(
        pkg = ":pkg",
        dist = ":dist",
        package_node_modules = ":node_modules",
        visibility = ["PUBLIC"],
        **kwargs):
    _node_pkg_bin(
        pkg = pkg,
        dist = dist,
        package_node_modules = package_node_modules,
        visibility = visibility,
        **kwargs,
    )

def npm_bin(
        visibility = ["PUBLIC"],
        node_modules = ":node_modules",
        **kwargs):
    _npm_bin(
        node_modules = node_modules,
        visibility = visibility,
        **kwargs,
    )

def package_node_modules(
        visibility = ["PUBLIC"],
        **kwargs):
    _package_node_modules(visibility = visibility, **kwargs)

def pnpm_lock(
        visibility = ["PUBLIC"],
        **kwargs):
    _pnpm_lock(visibility = visibility, **kwargs)

def typescript_dist(
        tsc = ":tsc",
        package_node_modules = ":node_modules",
        visibility = ["PUBLIC"],
        **kwargs):
    _typescript_dist(
        tsc = tsc,
        package_node_modules = package_node_modules,
        visibility = visibility,
        **kwargs,
    )

def vite_app(
        vite = ":vite",
        package_node_modules = ":node_modules",
        visibility = ["PUBLIC"],
        **kwargs):
    _vite_app(
        vite = vite,
        package_node_modules = package_node_modules,
        visibility = visibility,
        **kwargs,
    )

def workspace_node_modules(
        visibility = ["PUBLIC"],
        **kwargs):
    _workspace_node_modules(visibility = visibility, **kwargs)
