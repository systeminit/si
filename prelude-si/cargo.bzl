def cargo_clippy_impl(ctx: "context") -> [[DefaultInfo.type, RunInfo.type]]:
    run_cmd_args = cmd_args([
        "cargo",
        "clippy",
        "--all-targets",
        "--no-deps",
        "--package",
        ctx.attrs.crate,
        "--",
    ])
    run_cmd_args.hidden(ctx.attrs.srcs)

    args_file = ctx.actions.write("cargo-clippy-args.txt", run_cmd_args)

    return [
        DefaultInfo(
            default_output = args_file,
        ),
        RunInfo(
            args = run_cmd_args,
        ),
    ]

cargo_clippy = rule(impl = cargo_clippy_impl, attrs = {
    "crate": attrs.string(),
    "srcs": attrs.list(
        attrs.source(),
        default = [],
        doc = """The set of Rust source files in the crate."""
    ),
})

def cargo_clippy_fix_impl(ctx: "context") -> [[DefaultInfo.type, RunInfo.type]]:
    run_cmd_args = cmd_args([
        "cargo",
        "clippy",
        "--fix",
        "--allow-dirty",
        "--allow-staged",
        "--all-targets",
        "--no-deps",
        "--package",
        ctx.attrs.crate,
        "--",
    ])
    run_cmd_args.hidden(ctx.attrs.srcs)

    args_file = ctx.actions.write("cargo-clippy-args.txt", run_cmd_args)

    return [
        DefaultInfo(
            default_output = args_file,
        ),
        RunInfo(
            args = run_cmd_args,
        ),
    ]

cargo_clippy_fix = rule(impl = cargo_clippy_fix_impl, attrs = {
    "crate": attrs.string(),
    "srcs": attrs.list(
        attrs.source(),
        default = [],
        doc = """The set of Rust source files in the crate."""
    ),
})

def cargo_check_impl(ctx: "context") -> [[DefaultInfo.type, RunInfo.type]]:
    run_cmd_args = cmd_args([
        "cargo",
        "check",
        "--all-targets",
        "--package",
        ctx.attrs.crate,
        "--",
    ])
    run_cmd_args.hidden(ctx.attrs.srcs)

    args_file = ctx.actions.write("cargo-check-args.txt", run_cmd_args)

    return [
        DefaultInfo(
            default_output = args_file,
        ),
        RunInfo(
            args = run_cmd_args,
        ),
    ]

cargo_check = rule(impl = cargo_check_impl, attrs = {
    "crate": attrs.string(),
    "srcs": attrs.list(
        attrs.source(),
        default = [],
        doc = """The set of Rust source files in the crate."""
    ),
})

def cargo_doc_impl(ctx: "context") -> [[DefaultInfo.type, RunInfo.type]]:
    run_cmd_args = cmd_args([
        "cargo",
        "doc",
        "--no-deps",
        "--package",
        ctx.attrs.crate,
    ])
    run_cmd_args.hidden(ctx.attrs.srcs)

    args_file = ctx.actions.write("cargo-doc-args.txt", run_cmd_args)

    return [
        DefaultInfo(
            default_output = args_file,
        ),
        RunInfo(
            args = run_cmd_args,
        ),
    ]

cargo_doc = rule(impl = cargo_doc_impl, attrs = {
    "crate": attrs.string(),
    "srcs": attrs.list(
        attrs.source(),
        default = [],
        doc = """The set of Rust source files in the crate."""
    ),
})

def cargo_doc_check_impl(ctx: "context") -> [[DefaultInfo.type, RunInfo.type]]:
    run_cmd_args = cmd_args([
        "cargo",
        "doc",
        "--no-deps",
        "--package",
        ctx.attrs.crate,
    ])

    args_file = ctx.actions.write("cargo-doc-args.txt", run_cmd_args)

    # Write a script to set an environment variable which doesn't appear possible otherwise with
    # `cmd_args`.
    script, _ = ctx.actions.write(
        "cargo-doc.sh",
        [
            "#!/usr/bin/env bash",
            "export RUSTDOCFLAGS='-Dwarnings'",
            cmd_args(cmd_args(run_cmd_args, delimiter = " "), format = "exec {} $@"),
        ],
        is_executable = True,
        allow_args = True,
    )
    run_cmd_args = cmd_args([script]).hidden(ctx.attrs.srcs)

    return [
        DefaultInfo(
            default_output = args_file,
        ),
        RunInfo(
            args = run_cmd_args,
        ),
    ]

cargo_doc_check = rule(impl = cargo_doc_check_impl, attrs = {
    "crate": attrs.string(),
    "srcs": attrs.list(
        attrs.source(),
        default = [],
        doc = """The set of Rust source files in the crate."""
    ),
})

def cargo_fmt_impl(ctx: "context") -> [[DefaultInfo.type, RunInfo.type]]:
    run_cmd_args = cmd_args([
        "cargo",
        "fmt",
        "--package",
        ctx.attrs.crate,
        "--",
    ])
    run_cmd_args.hidden(ctx.attrs.srcs)

    args_file = ctx.actions.write("cargo-fmt-args.txt", run_cmd_args)

    return [
        DefaultInfo(
            default_output = args_file,
        ),
        RunInfo(
            args = run_cmd_args,
        ),
    ]

cargo_fmt = rule(impl = cargo_fmt_impl, attrs = {
    "crate": attrs.string(),
    "srcs": attrs.list(
        attrs.source(),
        default = [],
        doc = """The set of Rust source files in the crate."""
    ),
})

def cargo_fmt_check_impl(ctx: "context") -> [[DefaultInfo.type, RunInfo.type]]:
    run_cmd_args = cmd_args([
        "cargo",
        "fmt",
        "--check",
        "--package",
        ctx.attrs.crate,
        "--",
    ])
    run_cmd_args.hidden(ctx.attrs.srcs)

    args_file = ctx.actions.write("cargo-fmt-args.txt", run_cmd_args)

    return [
        DefaultInfo(
            default_output = args_file,
        ),
        RunInfo(
            args = run_cmd_args,
        ),
    ]

cargo_fmt_check = rule(impl = cargo_fmt_check_impl, attrs = {
    "crate": attrs.string(),
    "srcs": attrs.list(
        attrs.source(),
        default = [],
        doc = """The set of Rust source files in the crate."""
    ),
})
