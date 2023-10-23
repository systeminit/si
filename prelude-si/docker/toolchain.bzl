DockerToolchainInfo = provider(fields = {
    "capture_stdout": typing.Any,
    "docker_build_context": typing.Any,
    "docker_build_context_srcs_from_deps": typing.Any,
    "docker_container_run": typing.Any,
    "docker_image_build": typing.Any,
    "docker_image_push": typing.Any,
    "docker_image_promote": typing.Any,
})

def docker_toolchain_impl(ctx) -> list[[DefaultInfo, DockerToolchainInfo]]:
    """
    A Docker toolchain.
    """
    return [
        DefaultInfo(),
        DockerToolchainInfo(
            capture_stdout = ctx.attrs._capture_stdout,
            docker_build_context = ctx.attrs._docker_build_context,
            docker_build_context_srcs_from_deps = ctx.attrs._docker_build_context_srcs_from_deps,
            docker_container_run = ctx.attrs._docker_container_run,
            docker_image_build = ctx.attrs._docker_image_build,
            docker_image_push = ctx.attrs._docker_image_push,
            docker_image_promote = ctx.attrs._docker_image_promote,
        ),
    ]

docker_toolchain = rule(
    impl = docker_toolchain_impl,
    attrs = {
        "_capture_stdout": attrs.dep(
            default = "prelude-si//docker:capture_stdout.py",
        ),
        "_docker_build_context": attrs.dep(
            default = "prelude-si//docker:docker_build_context.py",
        ),
        "_docker_build_context_srcs_from_deps": attrs.dep(
            default = "prelude-si//docker:docker_build_context_srcs_from_deps.bxl",
        ),
        "_docker_container_run": attrs.dep(
            default = "prelude-si//docker:docker_container_run.py",
        ),
        "_docker_image_build": attrs.dep(
            default = "prelude-si//docker:docker_image_build.py",
        ),
        "_docker_image_push": attrs.dep(
            default = "prelude-si//docker:docker_image_push.py",
        ),
        "_docker_image_promote": attrs.dep(
            default = "prelude-si//docker:docker_image_promote.py",
        ),
    },
    is_toolchain_rule = True,
)
