load(
    "@prelude-si//:docker.bzl",
    _docker_image = "docker_image",
)

def docker_image(
        dockerfile = "Dockerfile",
        organization = "systeminit",
        source_url = "http://github.com/systeminit/si.git",
        author = "The System Initiative <dev@systeminit.com>",
        license = "Apache-2.0",
        visibility = ["PUBLIC"],
        **kwargs):
    _docker_image(
        dockerfile = dockerfile,
        organization = organization,
        source_url = source_url,
        author = author,
        license = license,
        visibility = visibility,
        **kwargs,
    )
