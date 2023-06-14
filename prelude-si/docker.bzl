load("@prelude//python:toolchain.bzl", "PythonToolchainInfo")
load("//docker:toolchain.bzl", "DockerToolchainInfo")
load("//git:toolchain.bzl", "GitToolchainInfo")

def docker_image_impl(ctx: "context") -> [[DefaultInfo.type, RunInfo.type]]:
    docker_build_ctx = docker_build_context(ctx)
    image_archive = build_docker_image(ctx, docker_build_ctx)
    run_args = docker_run_args(ctx, image_archive)

    return [
        DefaultInfo(
            default_outputs = [
                image_archive.tar_archive,
                image_archive.metadata,
                image_archive.tags,
            ],
        ),
        RunInfo(args = run_args),
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

DockerImageArchive = record(
    tar_archive = field("artifact"),
    metadata = field("artifact"),
    tags = field("artifact"),
)

def build_docker_image(
    ctx: "context",
    docker_build_ctx: DockerBuildContext.type,
) -> DockerImageArchive.type:
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
        docker_toolchain.docker_build[DefaultInfo].default_outputs,
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

    return DockerImageArchive(
        tar_archive = tar_archive,
        metadata = metadata,
        tags = tags,
    )

def docker_run_args(ctx: "context", archive: DockerImageArchive.type) -> "cmd_args":
    docker_toolchain = ctx.attrs._docker_toolchain[DockerToolchainInfo]

    cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        docker_toolchain.docker_run[DefaultInfo].default_outputs,
        "--tags-file",
        archive.tags,
    )
    cmd.add(ctx.attrs.run_docker_args)
    cmd.add("--")
    cmd.add(ctx.attrs.run_container_args)

    return cmd
