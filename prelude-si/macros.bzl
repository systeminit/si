load(
    "@prelude-si//macros:native.bzl",
    _alias = "alias",
    _export_file = "export_file",
    _filegroup = "filegroup",
    _sh_binary = "sh_binary",
    _test_suite = "test_suite",
)
alias = _alias
export_file = _export_file
filegroup = _filegroup
sh_binary = _sh_binary
test_suite = _test_suite

load(
    "@prelude-si//macros:deno.bzl",
    _deno_binary = "deno_binary",
)
deno_binary = _deno_binary

load(
    "@prelude-si//macros:docker.bzl",
    _docker_image = "docker_image",
)
docker_image = _docker_image

load(
    "@prelude-si//macros:rootfs.bzl",
    _rootfs = "rootfs",
)
rootfs = _rootfs

load(
    "@prelude-si//macros:nix.bzl",
    _nix_binary = "nix_binary",
    _nix_flake_lock = "nix_flake_lock",
    _nix_omnibus_pkg = "nix_omnibus_pkg",
)
nix_binary = _nix_binary
nix_flake_lock = _nix_flake_lock
nix_omnibus_pkg = _nix_omnibus_pkg

load(
    "@prelude-si//macros:e2e.bzl",
    _e2e_test = "e2e_test",
)
e2e_test = _e2e_test

load(
    "@prelude-si//macros:pnpm.bzl",
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

eslint = _eslint
ts_test = _ts_test
node_pkg_bin = _node_pkg_bin
npm_bin = _npm_bin
package_node_modules = _package_node_modules
pnpm_lock = _pnpm_lock
pnpm_workspace = _pnpm_workspace
prettier_check = _prettier_check
typescript_check = _typescript_check
typescript_dist = _typescript_dist
typescript_runnable_dist = _typescript_runnable_dist
typescript_runnable_dist_bin = _typescript_runnable_dist_bin
vite_app = _vite_app
workspace_node_modules = _workspace_node_modules

load(
    "@prelude-si//macros:python.bzl",
    _yapf_check = "yapf_check",
)
yapf_check = _yapf_check

load(
    "@prelude-si//macros:rust.bzl",
    _rust_binary = "rust_binary",
    _rust_library = "rust_library",
    _rust_test = "rust_test",
    _rust_binary_pkg = "rust_binary_pkg",
)
rust_binary = _rust_binary
rust_library = _rust_library
rust_test = _rust_test
rust_binary_pkg = _rust_binary_pkg

load(
    "@prelude-si//macros:shell.bzl",
    _shellcheck = "shellcheck",
    _shfmt_check = "shfmt_check",
)
shellcheck = _shellcheck
shfmt_check = _shfmt_check

load(
    "@prelude-si//macros:toml.bzl",
    _toml_format = "toml_format",
    _toml_format_check = "toml_format_check",
)
toml_format = _toml_format
toml_format_check = _toml_format_check

load(
    "@prelude-si//macros:tilt.bzl",
    _tilt_docker_compose_pull = "tilt_docker_compose_pull",
    _tilt_docker_compose_stop = "tilt_docker_compose_stop",
    _tilt_down = "tilt_down",
    _tilt_up = "tilt_up",
)
tilt_docker_compose_pull = _tilt_docker_compose_pull
tilt_docker_compose_stop = _tilt_docker_compose_stop
tilt_down = _tilt_down
tilt_up = _tilt_up
