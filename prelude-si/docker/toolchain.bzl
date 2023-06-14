DockerToolchainInfo = provider(fields = [
    "capture_stdout",
    "docker_build",
    "docker_build_context",
    "docker_run",
])

def docker_toolchain_impl(ctx) -> [[DefaultInfo.type, DockerToolchainInfo.type]]:
    """
    A Docker toolchain.
    """
    return [
        DefaultInfo(),
        DockerToolchainInfo(
            capture_stdout = ctx.attrs._capture_stdout,
            docker_build= ctx.attrs._docker_build,
            docker_build_context = ctx.attrs._docker_build_context,
            docker_run= ctx.attrs._docker_run,
        ),
    ]

docker_toolchain = rule(
    impl = docker_toolchain_impl,
    attrs = {
        "_capture_stdout": attrs.dep(
            default = "prelude-si//docker:capture_stdout.py",
        ),
        "_docker_build": attrs.dep(
            default = "prelude-si//docker:docker_build.py",
        ),
        "_docker_build_context": attrs.dep(
            default = "prelude-si//docker:docker_build_context.py",
        ),
        "_docker_run": attrs.dep(
            default = "prelude-si//docker:docker_run.py",
        ),
    },
    is_toolchain_rule = True,
)
