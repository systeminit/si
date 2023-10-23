load(
    "@prelude//decls/re_test_common.bzl",
    "re_test_common",
)
load(
    "@prelude//test/inject_test_run_info.bzl",
    "inject_test_run_info",
)
load(
    "@prelude//tests:re_utils.bzl",
    "get_re_executor_from_props",
)
load(
    "@prelude-si//:test.bzl",
    "inject_test_env",
)

def cargo_clippy_impl(ctx: AnalysisContext) -> list[[
    DefaultInfo,
    RunInfo,
    ExternalRunnerTestInfo,
]]:
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

    # Setup a RE executor based on the `remote_execution` param.
    re_executor = get_re_executor_from_props(ctx)

    # We implicitly make the target run from the project root if remote
    # excution options were specified
    run_from_project_root = "buck2_run_from_project_root" in (
        ctx.attrs.labels or []
    ) or re_executor != None

    return inject_test_run_info(
        ctx,
        ExternalRunnerTestInfo(
            type = "cargo",
            command = [run_cmd_args],
            env = ctx.attrs.env,
            labels = ctx.attrs.labels,
            contacts = ctx.attrs.contacts,
            default_executor = re_executor,
            run_from_project_root = run_from_project_root,
            use_project_relative_paths = run_from_project_root,
        ),
    ) + [
        DefaultInfo(default_output = args_file),
    ]

cargo_clippy = rule(
    impl = cargo_clippy_impl,
    attrs = {
        "crate": attrs.string(),
        "srcs": attrs.list(
            attrs.source(),
            default = [],
            doc = """The set of Rust source files in the crate.""",
        ),
    } | re_test_common.test_args() | inject_test_env.args(),
)

def cargo_clippy_fix_impl(ctx: AnalysisContext) -> list[[DefaultInfo, RunInfo]]:
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

cargo_clippy_fix = rule(
    impl = cargo_clippy_fix_impl,
    attrs = {
        "crate": attrs.string(),
        "srcs": attrs.list(
            attrs.source(),
            default = [],
            doc = """The set of Rust source files in the crate.""",
        ),
    },
)

def cargo_check_impl(ctx: AnalysisContext) -> list[[
    DefaultInfo,
    RunInfo,
    ExternalRunnerTestInfo,
]]:
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

    # Setup a RE executor based on the `remote_execution` param.
    re_executor = get_re_executor_from_props(ctx)

    # We implicitly make the target run from the project root if remote
    # excution options were specified
    run_from_project_root = "buck2_run_from_project_root" in (
        ctx.attrs.labels or []
    ) or re_executor != None

    return inject_test_run_info(
        ctx,
        ExternalRunnerTestInfo(
            type = "cargo",
            command = [run_cmd_args],
            env = ctx.attrs.env,
            labels = ctx.attrs.labels,
            contacts = ctx.attrs.contacts,
            default_executor = re_executor,
            run_from_project_root = run_from_project_root,
            use_project_relative_paths = run_from_project_root,
        ),
    ) + [
        DefaultInfo(default_output = args_file),
    ]

cargo_check = rule(
    impl = cargo_check_impl,
    attrs = {
        "crate": attrs.string(),
        "srcs": attrs.list(
            attrs.source(),
            default = [],
            doc = """The set of Rust source files in the crate.""",
        ),
    } | re_test_common.test_args() | inject_test_env.args(),
)

def cargo_doc_impl(ctx: AnalysisContext) -> list[[DefaultInfo, RunInfo]]:
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

cargo_doc = rule(
    impl = cargo_doc_impl,
    attrs = {
        "crate": attrs.string(),
        "srcs": attrs.list(
            attrs.source(),
            default = [],
            doc = """The set of Rust source files in the crate.""",
        ),
    },
)

def cargo_doc_check_impl(ctx: AnalysisContext) -> list[[
    DefaultInfo,
    RunInfo,
    ExternalRunnerTestInfo,
]]:
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

    # Setup a RE executor based on the `remote_execution` param.
    re_executor = get_re_executor_from_props(ctx)

    # We implicitly make the target run from the project root if remote
    # excution options were specified
    run_from_project_root = "buck2_run_from_project_root" in (
        ctx.attrs.labels or []
    ) or re_executor != None

    return inject_test_run_info(
        ctx,
        ExternalRunnerTestInfo(
            type = "cargo",
            command = [run_cmd_args],
            env = ctx.attrs.env,
            labels = ctx.attrs.labels,
            contacts = ctx.attrs.contacts,
            default_executor = re_executor,
            run_from_project_root = run_from_project_root,
            use_project_relative_paths = run_from_project_root,
        ),
    ) + [
        DefaultInfo(default_output = args_file),
    ]

cargo_doc_check = rule(
    impl = cargo_doc_check_impl,
    attrs = {
        "crate": attrs.string(),
        "srcs": attrs.list(
            attrs.source(),
            default = [],
            doc = """The set of Rust source files in the crate.""",
        ),
    } | re_test_common.test_args() | inject_test_env.args(),
)

def cargo_fmt_impl(ctx: AnalysisContext) -> list[[DefaultInfo, RunInfo]]:
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

cargo_fmt = rule(
    impl = cargo_fmt_impl,
    attrs = {
        "crate": attrs.string(),
        "srcs": attrs.list(
            attrs.source(),
            default = [],
            doc = """The set of Rust source files in the crate.""",
        ),
    },
)

def cargo_fmt_check_impl(ctx: AnalysisContext) -> list[[
    DefaultInfo,
    RunInfo,
    ExternalRunnerTestInfo,
]]:
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

    # Setup a RE executor based on the `remote_execution` param.
    re_executor = get_re_executor_from_props(ctx)

    # We implicitly make the target run from the project root if remote
    # excution options were specified
    run_from_project_root = "buck2_run_from_project_root" in (
        ctx.attrs.labels or []
    ) or re_executor != None

    return inject_test_run_info(
        ctx,
        ExternalRunnerTestInfo(
            type = "cargo",
            command = [run_cmd_args],
            env = ctx.attrs.env,
            labels = ctx.attrs.labels,
            contacts = ctx.attrs.contacts,
            default_executor = re_executor,
            run_from_project_root = run_from_project_root,
            use_project_relative_paths = run_from_project_root,
        ),
    ) + [
        DefaultInfo(default_output = args_file),
    ]

cargo_fmt_check = rule(
    impl = cargo_fmt_check_impl,
    attrs = {
        "crate": attrs.string(),
        "srcs": attrs.list(
            attrs.source(),
            default = [],
            doc = """The set of Rust source files in the crate.""",
        ),
    } | re_test_common.test_args() | inject_test_env.args(),
)
