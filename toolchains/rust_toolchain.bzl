load("@prelude//rust:rust_toolchain.bzl", "PanicRuntime", "RustToolchainInfo")
load("@prelude//rust/tools:attrs.bzl", "internal_tool_attrs")

def _nix_rust_toolchain_impl(ctx):
    return [
        DefaultInfo(),
        RustToolchainInfo(
            allow_lints = ctx.attrs.allow_lints,
            clippy_driver = RunInfo(args = [ctx.attrs.clippy_path]),
            clippy_toml = ctx.attrs.clippy_toml[DefaultInfo].default_outputs[0] if ctx.attrs.clippy_toml else None,
            compiler = RunInfo(args = [ctx.attrs.rustc_path]),
            default_edition = ctx.attrs.default_edition,
            panic_runtime = PanicRuntime("unwind"),
            deny_lints = ctx.attrs.deny_lints,
            doctests = ctx.attrs.doctests,
            failure_filter_action = ctx.attrs.failure_filter_action[RunInfo],
            nightly_features = ctx.attrs.nightly_features,
            report_unused_deps = ctx.attrs.report_unused_deps,
            rustc_action = ctx.attrs.rustc_action[RunInfo],
            rustc_binary_flags = ctx.attrs.rustc_binary_flags,
            rustc_flags = ctx.attrs.rustc_flags,
            rustc_target_triple = ctx.attrs.rustc_target_triple,
            rustc_test_flags = ctx.attrs.rustc_test_flags,
            rustdoc = RunInfo(args = [ctx.attrs.rustfmt_path]),
            rustdoc_flags = ctx.attrs.rustdoc_flags,
            rustdoc_test_with_resources = ctx.attrs.rustdoc_test_with_resources[RunInfo],
            rustdoc_coverage = ctx.attrs.rustdoc_coverage[RunInfo],
            transitive_dependency_symlinks_tool = ctx.attrs.transitive_dependency_symlinks_tool[RunInfo],
            warn_lints = ctx.attrs.warn_lints,
        ),
    ]

nix_rust_toolchain = rule(
    impl = _nix_rust_toolchain_impl,
    attrs = internal_tool_attrs | {
        "allow_lints": attrs.list(attrs.string(), default = []),
        "clippy_toml": attrs.option(attrs.dep(providers = [DefaultInfo]), default = None),
        "default_edition": attrs.option(attrs.string(), default = None),
        "deny_lints": attrs.list(attrs.string(), default = []),
        "doctests": attrs.bool(default = False),
        "nightly_features": attrs.bool(default = False),
        "report_unused_deps": attrs.bool(default = False),
        "rustc_binary_flags": attrs.list(attrs.string(), default = []),
        "rustc_flags": attrs.list(attrs.string(), default = []),
        "rustc_target_triple": attrs.string(default = "x86_64-unknown-linux-gnu"),
        "rustc_test_flags": attrs.list(attrs.string(), default = []),
        "rustdoc_flags": attrs.list(attrs.string(), default = []),
        "warn_lints": attrs.list(attrs.string(), default = []),
        # Tool paths
        "rustc_path": attrs.string(),
        "cargo_path": attrs.string(),
        "clippy_path": attrs.string(),
        "rustfmt_path": attrs.string(),
    },
    is_toolchain_rule = True,
)
