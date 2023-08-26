load(
    "@prelude//decls/common.bzl",
    "buck",
)
load(
    "@prelude//test/inject_test_run_info.bzl",
    "inject_test_run_info",
)
load(
    "@prelude//tests:re_utils.bzl",
    "get_re_executor_from_props",
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
    re_executor = get_re_executor_from_props(ctx.attrs.remote_execution)

    return inject_test_run_info(
        ctx,
        ExternalRunnerTestInfo(
            type = "cargo",
            command = [run_cmd_args],
            env = ctx.attrs.env,
            labels = ctx.attrs.labels,
            contacts = ctx.attrs.contacts,
            default_executor = re_executor,
            # We implicitly make this test via the project root, instead of
            # the cell root (e.g. fbcode root).
            run_from_project_root = re_executor != None,
            use_project_relative_paths = re_executor != None,
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
            doc = """The set of Rust source files in the crate."""
        ),
        "env": attrs.dict(
            key = attrs.string(),
            value = attrs.arg(),
            sorted = False,
            default = {},
            doc = """Set environment variables for this rule's invocation of cargo. The environment
            variable values may include macros which are expanded.""",
        ),
        "labels": attrs.list(
            attrs.string(),
            default = [],
        ),
        "contacts": attrs.list(
            attrs.string(),
            default = [],
        ),
        "remote_execution": buck.re_opts_for_tests_arg(),
        "_inject_test_env": attrs.default_only(
            attrs.dep(default = "prelude//test/tools:inject_test_env"),
        ),
    },
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
            doc = """The set of Rust source files in the crate."""
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
    re_executor = get_re_executor_from_props(ctx.attrs.remote_execution)

    return inject_test_run_info(
        ctx,
        ExternalRunnerTestInfo(
            type = "cargo",
            command = [run_cmd_args],
            env = ctx.attrs.env,
            labels = ctx.attrs.labels,
            contacts = ctx.attrs.contacts,
            default_executor = re_executor,
            # We implicitly make this test via the project root, instead of
            # the cell root (e.g. fbcode root).
            run_from_project_root = re_executor != None,
            use_project_relative_paths = re_executor != None,
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
            doc = """The set of Rust source files in the crate."""
        ),
        "env": attrs.dict(
            key = attrs.string(),
            value = attrs.arg(),
            sorted = False,
            default = {},
            doc = """Set environment variables for this rule's invocation of cargo. The environment
            variable values may include macros which are expanded.""",
        ),
        "labels": attrs.list(
            attrs.string(),
            default = [],
        ),
        "contacts": attrs.list(
            attrs.string(),
            default = [],
        ),
        "remote_execution": buck.re_opts_for_tests_arg(),
        "_inject_test_env": attrs.default_only(
            attrs.dep(default = "prelude//test/tools:inject_test_env"),
        ),
    },
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
            doc = """The set of Rust source files in the crate."""
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
    re_executor = get_re_executor_from_props(ctx.attrs.remote_execution)

    return inject_test_run_info(
        ctx,
        ExternalRunnerTestInfo(
            type = "cargo",
            command = [run_cmd_args],
            env = ctx.attrs.env,
            labels = ctx.attrs.labels,
            contacts = ctx.attrs.contacts,
            default_executor = re_executor,
            # We implicitly make this test via the project root, instead of
            # the cell root (e.g. fbcode root).
            run_from_project_root = re_executor != None,
            use_project_relative_paths = re_executor != None,
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
            doc = """The set of Rust source files in the crate."""
        ),
        "env": attrs.dict(
            key = attrs.string(),
            value = attrs.arg(),
            sorted = False,
            default = {},
            doc = """Set environment variables for this rule's invocation of cargo. The environment
            variable values may include macros which are expanded.""",
        ),
        "labels": attrs.list(
            attrs.string(),
            default = [],
        ),
        "contacts": attrs.list(
            attrs.string(),
            default = [],
        ),
        "remote_execution": buck.re_opts_for_tests_arg(),
        "_inject_test_env": attrs.default_only(
            attrs.dep(default = "prelude//test/tools:inject_test_env"),
        ),
    },
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
            doc = """The set of Rust source files in the crate."""
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
    re_executor = get_re_executor_from_props(ctx.attrs.remote_execution)

    return inject_test_run_info(
        ctx,
        ExternalRunnerTestInfo(
            type = "cargo",
            command = [run_cmd_args],
            env = ctx.attrs.env,
            labels = ctx.attrs.labels,
            contacts = ctx.attrs.contacts,
            default_executor = re_executor,
            # We implicitly make this test via the project root, instead of
            # the cell root (e.g. fbcode root).
            run_from_project_root = re_executor != None,
            use_project_relative_paths = re_executor != None,
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
            doc = """The set of Rust source files in the crate."""
        ),
        "env": attrs.dict(
            key = attrs.string(),
            value = attrs.arg(),
            sorted = False,
            default = {},
            doc = """Set environment variables for this rule's invocation of cargo. The environment
            variable values may include macros which are expanded.""",
        ),
        "labels": attrs.list(
            attrs.string(),
            default = [],
        ),
        "contacts": attrs.list(
            attrs.string(),
            default = [],
        ),
        "remote_execution": buck.re_opts_for_tests_arg(),
        "_inject_test_env": attrs.default_only(
            attrs.dep(default = "prelude//test/tools:inject_test_env"),
        ),
    },
)
