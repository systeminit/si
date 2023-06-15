load("@prelude//python:toolchain.bzl", "PythonToolchainInfo")
load("//docker:toolchain.bzl", "DockerToolchainInfo")
load("//git:toolchain.bzl", "GitToolchainInfo")

DockerImageInfo = provider(fields = {
    "tar_archive": "artifact",
    "metadata": "artifact",
    "tags": "artifact",
})

def docker_image_impl(ctx: "context") -> [[DefaultInfo.type, RunInfo.type, DockerImageInfo.type]]:
    docker_build_ctx = docker_build_context(ctx)
    image_info = build_docker_image(ctx, docker_build_ctx)
    run_args = docker_run_args(ctx, image_info)

    return [
        DefaultInfo(
            default_output = image_info.tar_archive,
        ),
        RunInfo(args = run_args),
        image_info,
    ]

docker_image = rule(
    impl = docker_image_impl,
    attrs = {
        "organization": attrs.option(
            attrs.string(),
            default = None,
            doc = """Organization name when creating image name.""",
        ),
        "image_name": attrs.option(
            attrs.string(),
            default = None,
            doc = """image name, excluding organization (default: 'attrs.name').""",
        ),
        "full_image_name": attrs.option(
            attrs.string(),
            default = None,
            doc = """Full image name, including organization (default:
            'attrs.organization/attrs.name').""",
        ),
        "dockerfile": attrs.source(
            doc = """Dockerfile describing the build.""",
        ),
        "srcs": attrs.dict(
            attrs.source(allow_directory = True),
            attrs.string(),
            default = {},
            doc = """Mapping of sources files to the relative directory in a Dockerfile context..""",
        ),
        "build_args": attrs.dict(
            attrs.string(),
            attrs.string(),
            default = {},
            doc = """Docker build --build-arg entries.""",
        ),
        "run_docker_args": attrs.list(
            attrs.string(),
            default = [],
            doc = """Extra arguments for `docker run` (not for container args).""",
        ),
        "run_container_args": attrs.list(
            attrs.string(),
            default = [],
            doc = """Extra arguments for running container (not for docker args).""",
        ),
        "author": attrs.string(
            doc = """Image author to be used in image metadata.""",
        ),
        "source_url": attrs.string(
            doc = """Source code URL to be used in image metadata.""",
        ),
        "license": attrs.string(
            doc = """Image license string to be used in image metadata.""",
        ),
        "_python_toolchain": attrs.toolchain_dep(
            default = "toolchains//:python",
            providers = [PythonToolchainInfo],
        ),
        "_docker_toolchain": attrs.toolchain_dep(
            default = "toolchains//:docker",
            providers = [DockerToolchainInfo],
        ),
        "_git_toolchain": attrs.toolchain_dep(
            default = "toolchains//:git",
            providers = [GitToolchainInfo],
        ),
    },
)

def docker_image_release_impl(ctx: "context") -> [[DefaultInfo.type, RunInfo.type]]:
    cli_args = ctx.actions.declare_output("args.txt")

    docker_toolchain = ctx.attrs._docker_toolchain[DockerToolchainInfo]

    cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        docker_toolchain.docker_image_push[DefaultInfo].default_outputs,
        "--archive-file",
        ctx.attrs.docker_image[DockerImageInfo].tar_archive,
        "--tags-file",
        ctx.attrs.docker_image[DockerImageInfo].tags,
        "--metadata-file",
        ctx.attrs.docker_image[DockerImageInfo].metadata,
    )

    ctx.actions.write(cli_args.as_output(), cmd)

    return [
        DefaultInfo(default_output = cli_args),
        RunInfo(args = cmd),
    ]

docker_image_release = rule(
    impl = docker_image_release_impl,
    attrs = {
        "docker_image": attrs.dep(
            providers = [DockerImageInfo],
            doc = """The `docker_image` artifact to release.""",
        ),
        "_python_toolchain": attrs.toolchain_dep(
            default = "toolchains//:python",
            providers = [PythonToolchainInfo],
        ),
        "_docker_toolchain": attrs.toolchain_dep(
            default = "toolchains//:docker",
            providers = [DockerToolchainInfo],
        ),
    },
)

def docker_image_promote_impl(ctx: "context") -> [[DefaultInfo.type, RunInfo.type]]:
    cli_args = ctx.actions.declare_output("args.txt")

    docker_toolchain = ctx.attrs._docker_toolchain[DockerToolchainInfo]

    cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        docker_toolchain.docker_image_promote[DefaultInfo].default_outputs,
    )
    if ctx.attrs.stable_tag:
        cmd.add("--stable-tag")
        cmd.add(ctx.attrs.stable_tag)
    if ctx.attrs.multi_arches:
        for multi_arch in ctx.attrs.multi_arches:
            cmd.add("--multi-arch")
            cmd.add(multi_arch)

    cmd.add(ctx.attrs.image_name)

    ctx.actions.write(cli_args.as_output(), cmd)

    return [
        DefaultInfo(default_output = cli_args),
        RunInfo(args = cmd),
    ]

docker_image_promote = rule(
    impl = docker_image_promote_impl,
    attrs = {
        "image_name": attrs.string(
            doc = """Docker image name minus tag (ex: `acme/myapp`).""",
        ),
        "stable_tag": attrs.option(
            attrs.string(),
            default = None,
            doc = """Override default stable tag name.""",
        ),
        "multi_arches": attrs.option(
            attrs.list(
                attrs.string(),
            ),
            default = None,
            doc = """Override default multi-arch platforms when promoting.""",
        ),
        "_python_toolchain": attrs.toolchain_dep(
            default = "toolchains//:python",
            providers = [PythonToolchainInfo],
        ),
        "_docker_toolchain": attrs.toolchain_dep(
            default = "toolchains//:docker",
            providers = [DockerToolchainInfo],
        ),
    },
)

DockerBuildContext = record(
    context_tree = field("artifact"),
)

def docker_build_context(ctx: "context") -> DockerBuildContext.type:
    context_tree = ctx.actions.declare_output("__docker_context")

    docker_toolchain = ctx.attrs._docker_toolchain[DockerToolchainInfo]

    cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        docker_toolchain.docker_build_context[DefaultInfo].default_outputs,
        "--dockerfile",
        ctx.attrs.dockerfile,
    )
    for src, rel_path in ctx.attrs.srcs.items():
        cmd.add("--src")
        cmd.add(cmd_args(src, format = "{}=" + rel_path))
    cmd.add(context_tree.as_output())

    ctx.actions.run(cmd, category = "docker_build_context")

    return DockerBuildContext(
        context_tree = context_tree,
    )

def build_docker_image(
    ctx: "context",
    docker_build_ctx: DockerBuildContext.type,
) -> DockerImageInfo.type:
    if ctx.attrs.full_image_name:
        image_name = ctx.attrs.full_image_name
    elif ctx.attrs.organization:
        image_name = "{}/{}".format(ctx.attrs.organization, ctx.attrs.image_name or ctx.attrs.name)
    else:
        fail("Either full_image_name or organization must be provided")

    tar_archive = ctx.actions.declare_output("{}.tar".format(image_name.replace("/", "--")))
    tags = ctx.actions.declare_output("tags.json")
    metadata = ctx.actions.declare_output("metadata.json")

    docker_toolchain = ctx.attrs._docker_toolchain[DockerToolchainInfo]
    git_toolchain = ctx.attrs._git_toolchain[GitToolchainInfo]

    cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        docker_toolchain.docker_image_build[DefaultInfo].default_outputs,
        "--git-info-program",
        git_toolchain.git_info[DefaultInfo].default_outputs,
        "--archive-out-file",
        tar_archive.as_output(),
        "--metadata-out-file",
        metadata.as_output(),
        "--tags-out-file",
        tags.as_output(),
        "--docker-context-dir",
        docker_build_ctx.context_tree,
        "--image-name",
        image_name,
        "--author",
        ctx.attrs.author,
        "--source-url",
        ctx.attrs.source_url,
        "--license",
        ctx.attrs.license,
    )
    for key, value in ctx.attrs.build_args.items():
        cmd.add("--build-arg")
        cmd.add("{}={}".format(key, value))

    ctx.actions.run(cmd, category = "docker_build")

    return DockerImageInfo(
        tar_archive = tar_archive,
        metadata = metadata,
        tags = tags,
    )

def docker_run_args(ctx: "context", archive: DockerImageInfo.type) -> "cmd_args":
    docker_toolchain = ctx.attrs._docker_toolchain[DockerToolchainInfo]

    cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        docker_toolchain.docker_container_run[DefaultInfo].default_outputs,
        "--tags-file",
        archive.tags,
    )
    cmd.add(ctx.attrs.run_docker_args)
    cmd.add("--")
    cmd.add(ctx.attrs.run_container_args)

    return cmd
