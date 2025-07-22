load(
    "@prelude-si//:macros.bzl",
    "alias",
    "deno_workspace",
    "export_file",
    "nix_flake_lock",
    "pnpm_lock",
    "pnpm_workspace",
    "toml_format",
    "toml_format_check",
    "workspace_node_modules",
)

alias(
    name = "edda",
    actual = "//bin/edda:edda",
)

alias(
    name = "forklift",
    actual = "//bin/forklift:forklift",
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

alias(
    name = "docs",
    actual = "//app/docs:dev",
)

alias(
    name = "sync-cargo-deps",
    actual = "//support/buck2:sync-cargo-deps",
)

alias(
    name = "all-rust-targets",
    actual = "//support/buck2:all-rust-targets",
)

alias(
    name = "update-api-docs",
    actual = "//app/docs:generate-api-docs",
    visibility = ["PUBLIC"],
)

export_file(
    name = ".editorconfig",
)

export_file(
    name = ".taplo.toml",
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

export_file(
    name = "deno.json",
)

pnpm_workspace(
    name = "pnpm-workspace.yaml",
    packages = [
        "//app/auth-portal:package.json",
        "//app/web:package.json",
        "//bin/auth-api:package.json",
        "//lib/eslint-config:package.json",
        "//lib/ts-lib:package.json",
        "//lib/tsconfig:package.json",
        "//lib/vue-lib:package.json",
    ],
)

deno_workspace(
    name = "deno_workspace",
    root_config = ":deno.json",
    packages = [
        "//bin/clover:deno.json",
        "//bin/lang-js:deno.json",
        "//lib/ts-lib-deno:deno.json",
        "//lib/jsr-systeminit/remove-empty:deno.json",
        "//lib/jsr-systeminit/ecs-template-qualification:deno.json",
        "//lib/jsr-systeminit/cf-db:deno.json",
        "//lib/jsr-systeminit/ai-agent:deno.json",
        "//bin/si-luminork-api-tests:deno.json",
    ],
    visibility = ["PUBLIC"],
)

pnpm_lock(
    name = "pnpm-lock.yaml",
)

export_file(
    name = "Cargo.toml",
)

toml_srcs = [
    "Cargo.toml",
    "clippy.toml",
    "rustfmt.toml",
    "tomlfmt.toml",
    "typos.toml",
]

toml_format_check(
    name = "check-format-toml",
    srcs = toml_srcs,
)

toml_format(
    name = "fix-format-toml",
    srcs = toml_srcs,
)

export_file(
    name = "rust-toolchain",
)

export_file(
    name = "rustfmt.toml",
)

export_file(
    name = "tomlfmt.toml",
)

workspace_node_modules(
    name = "node_modules",
)
