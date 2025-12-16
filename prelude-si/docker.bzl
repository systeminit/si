load("@prelude//python:toolchain.bzl", "PythonToolchainInfo")
load("@prelude-si//:artifact.bzl", "ArtifactInfo")
load("//build_context:toolchain.bzl", "BuildContextToolchainInfo")
load("//build_context.bzl", "BuildContext", _build_context = "build_context")
load("//docker:toolchain.bzl", "DockerToolchainInfo")
load("//git.bzl", "GitInfo")

DockerImageInfo = provider(fields = {
    "artifact": provider_field(typing.Any, default = None),  # [Artifact]
    "build_metadata": provider_field(typing.Any, default = None),  # [Artifact]
    "label_metadata": provider_field(typing.Any, default = None),  # [Artifact]
    "tag_metadata": provider_field(typing.Any, default = None),  # [Artifact]
})

def container_image_impl(ctx: AnalysisContext) -> list[[
    DefaultInfo,
    RunInfo,
    DockerImageInfo,
    GitInfo,
]]:
    # ArtifactInfo,
    srcs = {ctx.attrs.dockerfile: "."}
    if ctx.attrs.srcs:
        srcs.update(ctx.attrs.srcs)
    build_context = _build_context(ctx, ctx.attrs.build_deps, srcs)

    # Extract Git metadata from dependency and wrap in GitInfo provider
    git_metadata_file = ctx.attrs.git_metadata[DefaultInfo].default_outputs[0]
    git_info = GitInfo(file = git_metadata_file)

    image_info = build_docker_image(ctx, build_context, git_info)
    run_args = docker_run_args(ctx, image_info)

    return [
        DefaultInfo(
            default_output = image_info.artifact,
        ),
        RunInfo(args = run_args),
        image_info,
        # ArtifactInfo(
        #     artifact = image_info.artifact,
        #     metadata = build_metadata,
        #     family = ctx.attrs.image_name or ctx.attrs.name,
        #     variant = "container",
        # ),
        git_info,
    ]

container_image = rule(
    impl = container_image_impl,
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
        "build_deps": attrs.list(
            attrs.dep(),
            default = [],
            doc = """Buck2 targets that could be built in an image.""",
        ),
        "flake_lock": attrs.option(
            attrs.dep(),
            default = None,
            doc = """flake.lock dependency if Docker image uses Nix flake internally.""",
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
        "platform_targets": attrs.list(
            attrs.string(),
            default = [],
            doc = """List of target platforms this artifact supports.
            Used by CI to determine which platforms to build.""",
        ),
        "git_metadata": attrs.dep(
            default = "prelude-si//build_metadata:git",
            doc = """Git metadata target providing repository information.""",
        ),
        "_python_toolchain": attrs.toolchain_dep(
            default = "toolchains//:python",
            providers = [PythonToolchainInfo],
        ),
        "_build_context_toolchain": attrs.toolchain_dep(
            default = "toolchains//:build_context",
            providers = [BuildContextToolchainInfo],
        ),
        "_docker_toolchain": attrs.toolchain_dep(
            default = "toolchains//:docker",
            providers = [DockerToolchainInfo],
        ),
    },
)

def build_docker_image(
        ctx: AnalysisContext,
        docker_build_ctx: BuildContext,
        git_info: GitInfo) -> DockerImageInfo:
    if ctx.attrs.full_image_name:
        image_name = ctx.attrs.full_image_name
    elif ctx.attrs.organization:
        image_name = "{}/{}".format(ctx.attrs.organization, ctx.attrs.image_name or ctx.attrs.name)
    else:
        fail("Either full_image_name or organization must be provided")

    tar_name_prefix = "{}".format(image_name.replace("/", "--"))

    artifact = ctx.actions.declare_output("{}.tar".format(tar_name_prefix))
    build_metadata = ctx.actions.declare_output("build_metadata.json")
    tag_metadata = ctx.actions.declare_output("tag_metadata.json")
    label_metadata = ctx.actions.declare_output("label_metadata.json")

    docker_toolchain = ctx.attrs._docker_toolchain[DockerToolchainInfo]

    cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        docker_toolchain.docker_image_build[DefaultInfo].default_outputs,
        "--git-info-json",
        git_info.file,
        "--artifact-out-file",
        artifact.as_output(),
        "--build-metadata-out-file",
        build_metadata.as_output(),
        "--label-metadata-out-file",
        label_metadata.as_output(),
        "--tag-metadata-out-file",
        tag_metadata.as_output(),
        "--docker-context-dir",
        docker_build_ctx.root,
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
        artifact = artifact,
        build_metadata = build_metadata,
        label_metadata = label_metadata,
        tag_metadata = tag_metadata,
    )

def docker_run_args(ctx: AnalysisContext, image_info: DockerImageInfo) -> cmd_args:
    docker_toolchain = ctx.attrs._docker_toolchain[DockerToolchainInfo]

    cmd = cmd_args(
        ctx.attrs._python_toolchain[PythonToolchainInfo].interpreter,
        docker_toolchain.docker_container_run[DefaultInfo].default_outputs,
        "--tags-file",
        image_info.tag_metadata,
    )
    cmd.add(ctx.attrs.run_docker_args)
    cmd.add("--")
    cmd.add(ctx.attrs.run_container_args)

    return cmd
