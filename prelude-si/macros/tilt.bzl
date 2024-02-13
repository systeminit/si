load(
    "@prelude-si//:tilt.bzl",
    _tilt_docker_compose_pull = "tilt_docker_compose_pull",
    _tilt_docker_compose_stop = "tilt_docker_compose_stop",
    _tilt_down = "tilt_down",
    _tilt_up = "tilt_up",
)

def tilt_docker_compose_pull(
        visibility = ["PUBLIC"],
        **kwargs):
    _tilt_docker_compose_pull(
        visibility = visibility,
        **kwargs,
    )

def tilt_docker_compose_stop(
        visibility = ["PUBLIC"],
        **kwargs):
    _tilt_docker_compose_stop(
        visibility = visibility,
        **kwargs,
    )

def tilt_down(
        visibility = ["PUBLIC"],
        **kwargs):
    _tilt_down(
        visibility = visibility,
        **kwargs,
    )

def tilt_up(
        visibility = ["PUBLIC"],
        **kwargs):
    _tilt_up(
        visibility = visibility,
        **kwargs,
    )
