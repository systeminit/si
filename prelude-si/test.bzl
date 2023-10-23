def _inject_test_env_args() -> dict[str, Attr]:
    return {
        "env": attrs.dict(
            key = attrs.string(),
            value = attrs.arg(),
            sorted = False,
            default = {},
            doc = """Set environment variables for this rule's invocation of the program.
            The environment variable values may include macros which are expanded.""",
        ),
        "labels": attrs.list(
            attrs.string(),
            default = [],
        ),
        "contacts": attrs.list(
            attrs.string(),
            default = [],
        ),
        "_inject_test_env": attrs.default_only(
            attrs.dep(default = "prelude//test/tools:inject_test_env"),
        ),
    }

inject_test_env = struct(
    args = _inject_test_env_args,
)
