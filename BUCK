load(
    "@prelude-si//:macros.bzl",
    "alias",
    "export_file",
    "nix_flake_lock",
    "pnpm_lock",
    "pnpm_workspace",
    "workspace_node_modules",
)

alias(
    name = "council",
    actual = "//bin/council:council",
)

alias(
    name = "module-index",
    actual = "//bin/module-index:module-index",
)

alias(
    name = "pinga",
    actual = "//bin/pinga:pinga",
)

alias(
    name = "sdf",
    actual = "//bin/sdf:sdf",
)

alias(
    name = "si",
    actual = "//bin/si:si",
)

alias(
    name = "veritech",
    actual = "//bin/veritech:veritech",
)

alias(
    name = "web",
    actual = "//app/web:dev",
)

alias(
    name = "auth-portal",
    actual = "//app/auth-portal:dev",
)

alias(
    name = "auth-api",
    actual = "//bin/auth-api:dev",
)

export_file(
    name = ".editorconfig",
)

export_file(
    name = "clippy.toml",
)

export_file(
    name = "flake.nix",
)

nix_flake_lock(
    name = "flake.lock",
)

export_file(
    name = "package.json",
)

pnpm_workspace(
    name = "pnpm-workspace.yaml",
    packages = [
        "//app/auth-portal:package.json",
        "//app/web:package.json",
        "//bin/auth-api:package.json",
        "//bin/lang-js:package.json",
        "//lib/eslint-config:package.json",
        "//lib/ts-lib:package.json",
        "//lib/tsconfig:package.json",
        "//lib/vue-lib:package.json",
    ],
)

pnpm_lock(
    name = "pnpm-lock.yaml",
)

export_file(
    name = "rust-toolchain",
)

export_file(
    name = "rustfmt.toml",
)

workspace_node_modules(
    name = "node_modules",
)

