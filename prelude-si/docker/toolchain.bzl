DockerToolchainInfo = provider(fields = {
    "capture_stdout": typing.Any,
    "docker_container_run": typing.Any,
    "docker_image_build": typing.Any,
})

def docker_toolchain_impl(ctx) -> list[[DefaultInfo, DockerToolchainInfo]]:
    """
    A Docker toolchain.
    """
    return [
        DefaultInfo(),
        DockerToolchainInfo(
            capture_stdout = ctx.attrs._capture_stdout,
            docker_container_run = ctx.attrs._docker_container_run,
            docker_image_build = ctx.attrs._docker_image_build,
        ),
    ]

docker_toolchain = rule(
    impl = docker_toolchain_impl,
    attrs = {
        "_capture_stdout": attrs.dep(
            default = "prelude-si//docker:capture_stdout.py",
        ),
        "_docker_container_run": attrs.dep(
            default = "prelude-si//docker:docker_container_run.py",
        ),
        "_docker_image_build": attrs.dep(
            default = "prelude-si//docker:docker_image_build.py",
        ),
    },
    is_toolchain_rule = True,
)
