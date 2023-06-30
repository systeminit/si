load(
    "@prelude//python:toolchain.bzl",
    "PythonToolchainInfo",
)
load(
    "//rust:toolchain.bzl",
    "RustClippyToolchainInfo",
)
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

def clippy_check_impl(ctx: "context") -> [[
    DefaultInfo.type,
    RunInfo.type,
    ExternalRunnerTestInfo.type,
]]:
    clippy_txt = ctx.attrs.clippy_txt_dep[DefaultInfo].default_outputs

    rust_clippy_toolchain = ctx.attrs._rust_clippy_toolchain[RustClippyToolchainInfo]

    run_cmd_args = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        rust_clippy_toolchain.clippy_output[DefaultInfo].default_outputs,
        clippy_txt,
    )

    args_file = ctx.actions.write("args.txt", run_cmd_args)

    # Setup a RE executor based on the `remote_execution` param.
    re_executor = get_re_executor_from_props(ctx.attrs.remote_execution)

    return inject_test_run_info(
        ctx,
        ExternalRunnerTestInfo(
            type = "clippy",
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

clippy_check = rule(
    impl = clippy_check_impl,
    attrs = {
        "clippy_txt_dep": attrs.dep(
            doc = """Clippy sub target dep from a Rust library or binary""",
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
        "_python_toolchain": attrs.toolchain_dep(
            default = "toolchains//:python",
            providers = [PythonToolchainInfo],
        ),
        "_rust_clippy_toolchain": attrs.toolchain_dep(
            default = "toolchains//:rust_clippy",
            providers = [RustClippyToolchainInfo],
        ),
    },
)
