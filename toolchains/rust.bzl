load("@prelude//rust:rust_toolchain.bzl", "PanicRuntime", "RustToolchainInfo")
load("@prelude-si//:mise.bzl", "MiseInfo")

_DEFAULT_TRIPLE = select({
    "config//os:linux": select({
        "config//cpu:arm64": "aarch64-unknown-linux-gnu",
        "config//cpu:x86_64": "x86_64-unknown-linux-gnu",
    }),
    "config//os:macos": select({
        "config//cpu:arm64": "aarch64-apple-darwin",
        "config//cpu:x86_64": "x86_64-apple-darwin",
    }),
    "config//os:windows": select({
        "config//cpu:arm64": select({
            # Rustup's default ABI for the host on Windows is MSVC, not GNU.
            # When you do `rustup install stable` that's the one you get. It
            # makes you opt in to GNU by `rustup install stable-gnu`.
            "DEFAULT": "aarch64-pc-windows-msvc",
            "config//abi:gnu": "aarch64-pc-windows-gnu",
            "config//abi:msvc": "aarch64-pc-windows-msvc",
        }),
        "config//cpu:x86_64": select({
            "DEFAULT": "x86_64-pc-windows-msvc",
            "config//abi:gnu": "x86_64-pc-windows-gnu",
            "config//abi:msvc": "x86_64-pc-windows-msvc",
        }),
    }),
})

def _mise_rust_toolchain_impl(ctx):
    mise_info = ctx.attrs.mise_install[MiseInfo]
    mise_binary = cmd_args(mise_info.mise_bootstrap)
    shims = cmd_args(mise_info.mise_tools_dir, "/shims", delimiter="")
    wrapper_tool = ctx.attrs._wrapper[RunInfo]
    wrapper = cmd_args(wrapper_tool)
    wrapper.add(shims)

    rustc_cmd = cmd_args(
        [wrapper, mise_binary, "exec", "rust", "vfox:https://github.com/systeminit/vfox-clang@20.1.7", "--", "rustc"]
    )
    rustdoc_cmd = cmd_args(
        [wrapper, mise_binary, "exec", "rust",  "--", "rustdoc"]
    )

    # the way clippy is invoked deep in the prelude means
    # we need to have a single thing on the commandline
    # or everything gets mangled. We generate a one-off
    # script here for that purpose instead of reusing the
    # wrapper.
    clippy_wrapper_script = ctx.actions.declare_output("clippy_mise_wrapper.sh")
    ctx.actions.write(
        clippy_wrapper_script,
        cmd_args(
            "#!/bin/bash",
            "set -e",
            cmd_args("MISE_BINARY=", mise_binary, delimiter=""),
            'exec "$MISE_BINARY" exec rust -- clippy-driver "$@"',
        ),
        is_executable = True,
    )
    clippy_driver = RunInfo(args = [clippy_wrapper_script])

    return [
        DefaultInfo(),
        RustToolchainInfo(
            allow_lints = ctx.attrs.allow_lints,
            clippy_driver = clippy_driver,
            clippy_toml = ctx.attrs.clippy_toml[DefaultInfo].default_outputs[0] if ctx.attrs.clippy_toml else None,
            compiler = RunInfo(args = rustc_cmd),
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
            rustdoc = RunInfo(args = rustdoc_cmd),
            rustdoc_flags = ctx.attrs.rustdoc_flags,
            warn_lints = ctx.attrs.warn_lints,
        ),
    ]

mise_rust_toolchain = rule(
    impl = _mise_rust_toolchain_impl,
    attrs = {
        "mise_install": attrs.dep(
            providers = [MiseInfo],
            doc = "The mise_install target that provides the Rust installation",
            default = "toolchains//:rust_compiler",
        ),
        "allow_lints": attrs.list(attrs.string(), default = []),
        "clippy_toml": attrs.option(attrs.dep(providers = [DefaultInfo]), default = None),
        "default_edition": attrs.option(attrs.string(), default = None),
        "deny_lints": attrs.list(attrs.string(), default = []),
        "doctests": attrs.bool(default = False),
        "nightly_features": attrs.bool(default = False),
        "report_unused_deps": attrs.bool(default = False),
        "rustc_binary_flags": attrs.list(attrs.arg(), default = []),
        "rustc_flags": attrs.list(attrs.arg(), default = []),
        "rustc_target_triple": attrs.string(default = _DEFAULT_TRIPLE),
        "rustc_test_flags": attrs.list(attrs.arg(), default = []),
        "rustdoc_flags": attrs.list(attrs.arg(), default = []),
        "warn_lints": attrs.list(attrs.string(), default = []),
        "_wrapper": attrs.default_only(attrs.dep(providers = [RunInfo], default = "toolchains//:mise_wrapper")),
    },
    is_toolchain_rule = True,
)
