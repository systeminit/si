load("@prelude//rust:rust_toolchain.bzl", "PanicRuntime", "RustToolchainInfo")

def _nix_rust_toolchain_impl(ctx):
    wrapper_tool = ctx.attrs._rustc_wrapper[RunInfo]

    # Create a cmd_args object to properly combine the wrapper with the rustc path
    c_compiler = ctx.attrs.c_compiler
    cxx_compiler = ctx.attrs.cxx_compiler

    # Create command args: wrapper script + C compiler + C++ compiler + rustc path
    compiler_args = cmd_args(wrapper_tool)
    compiler_args.add(c_compiler)
    compiler_args.add(cxx_compiler)
    compiler_args.add(ctx.attrs.rustc_path)

    return [
        DefaultInfo(),
        RustToolchainInfo(
            advanced_unstable_linking = ctx.attrs.advanced_unstable_linking,
            allow_lints = ctx.attrs.allow_lints,
            clippy_driver = RunInfo(args = [ctx.attrs.clippy_path]),
            clippy_toml = ctx.attrs.clippy_toml[DefaultInfo].default_outputs[0] if ctx.attrs.clippy_toml else None,
            compiler = RunInfo(args = compiler_args),  # Use cmd_args object
            default_edition = ctx.attrs.default_edition,
            panic_runtime = PanicRuntime("unwind"),
            deny_lints = ctx.attrs.deny_lints,
            doctests = ctx.attrs.doctests,
            nightly_features = ctx.attrs.nightly_features,
            report_unused_deps = ctx.attrs.report_unused_deps,
            rustc_binary_flags = ctx.attrs.rustc_binary_flags,
            rustc_flags = ctx.attrs.rustc_flags,
            rustc_target_triple = ctx.attrs.rustc_target_triple,
            rustc_test_flags = ctx.attrs.rustc_test_flags,
            rustdoc = RunInfo(args = [ctx.attrs.rustfmt_path]),
            rustdoc_flags = ctx.attrs.rustdoc_flags,
            warn_lints = ctx.attrs.warn_lints,
        ),
    ]

rust_toolchain_attrs = {
    "advanced_unstable_linking": attrs.bool(default = False),
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
    "c_compiler": attrs.string(),
    "cxx_compiler": attrs.string(),
    "rustc_path": attrs.string(),
    "cargo_path": attrs.string(),
    "clippy_path": attrs.string(),
    "rustfmt_path": attrs.string(),
    # Add a dependency on our wrapper script
    "_rustc_wrapper": attrs.default_only(attrs.dep(providers = [RunInfo], default = "toolchains//:rustc_wrapper")),
    # Tools - we still need these for the implementation but they're no longer passed to RustToolchainInfo
    "failure_filter_action": attrs.default_only(attrs.dep(providers = [RunInfo], default = "prelude//rust/tools:failure_filter_action")),
    "rustc_action": attrs.default_only(attrs.dep(providers = [RunInfo], default = "prelude//rust/tools:rustc_action")),
    "rustdoc_test_with_resources": attrs.default_only(attrs.dep(providers = [RunInfo], default = "prelude//rust/tools:rustdoc_test_with_resources")),
    "rustdoc_coverage": attrs.default_only(attrs.dep(providers = [RunInfo], default = "prelude//rust/tools:rustdoc_coverage")),
    "transitive_dependency_symlinks_tool": attrs.default_only(attrs.dep(providers = [RunInfo], default = "prelude//rust/tools:transitive_dependency_symlinks")),
}

nix_rust_toolchain = rule(
    impl = _nix_rust_toolchain_impl,
    attrs = rust_toolchain_attrs,
    is_toolchain_rule = True,
)
