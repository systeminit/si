load(
    "@prelude-si//macros:native.bzl",
    _alias = "alias",
    _export_file = "export_file",
    _filegroup = "filegroup",
    _sh_binary = "sh_binary",
)
alias = _alias
export_file = _export_file
filegroup = _filegroup
sh_binary = _sh_binary

load(
    "@prelude-si//macros:docker.bzl",
    _docker_image = "docker_image",
)
docker_image = _docker_image

load(
    "@prelude-si//macros:pnpm.bzl",
    _eslint = "eslint",
    _node_pkg_bin = "node_pkg_bin",
    _npm_bin = "npm_bin",
    _package_node_modules = "package_node_modules",
    _pnpm_lock = "pnpm_lock",
    _pnpm_workspace = "pnpm_workspace",
    _typescript_check = "typescript_check",
    _typescript_dist = "typescript_dist",
    _vite_app = "vite_app",
    _workspace_node_modules = "workspace_node_modules",
)
eslint = _eslint
node_pkg_bin = _node_pkg_bin
npm_bin = _npm_bin
package_node_modules = _package_node_modules
pnpm_lock = _pnpm_lock
pnpm_workspace = _pnpm_workspace
typescript_check = _typescript_check
typescript_dist = _typescript_dist
vite_app = _vite_app
workspace_node_modules = _workspace_node_modules

load(
    "@prelude-si//macros:rust.bzl",
    _rust_binary = "rust_binary",
    _rust_library = "rust_library",
    _rust_test = "rust_test",
)
rust_binary = _rust_binary
rust_library = _rust_library
rust_test = _rust_test

load(
    "@prelude-si//macros:tilt.bzl",
    _tilt_docker_compose_stop = "tilt_docker_compose_stop",
    _tilt_down = "tilt_down",
    _tilt_up = "tilt_up",
)
tilt_docker_compose_stop = _tilt_docker_compose_stop
tilt_down = _tilt_down
tilt_up = _tilt_up
