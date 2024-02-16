def tilt_up_impl(ctx: AnalysisContext) -> list[[DefaultInfo, RunInfo]]:
    return _invoke_tilt(ctx, "up")

tilt_up = rule(
    impl = tilt_up_impl,
    attrs = {
        "tiltfile": attrs.string(
            default = "Tiltfile",
            doc = """The Tiltfile to run.""",
        ),
        "args": attrs.list(
            attrs.string(),
            default = [],
            doc = """Additional arguments passed as <Tiltfile args>.""",
        ),
        "tilt_args": attrs.list(
            attrs.string(),
            default = [],
            doc = """Additional arguments passed as `tilt` arguments.""",
        ),
    },
)

def tilt_down_impl(ctx: AnalysisContext) -> list[[DefaultInfo, RunInfo]]:
    return _invoke_tilt(ctx, "down")

tilt_down = rule(
    impl = tilt_down_impl,
    attrs = {
        "tiltfile": attrs.string(
            default = "Tiltfile",
            doc = """The Tiltfile to run.""",
        ),
        "args": attrs.list(
            attrs.string(),
            default = [],
            doc = """Additional arguments passed as <Tiltfile args>.""",
        ),
        "tilt_args": attrs.list(
            attrs.string(),
            default = [],
            doc = """Additional arguments passed as `tilt` arguments.""",
        ),
    },
)

def tilt_docker_compose_pull_impl(ctx: AnalysisContext) -> list[[DefaultInfo, RunInfo]]:
    docker_compose_file = "{}/{}".format(
        ctx.label.package,
        ctx.attrs.docker_compose_file,
    )

    run_cmd_args = cmd_args([
        "docker",
        "compose",
        "--file",
        docker_compose_file,
        "pull",
    ])
    run_cmd_args.add(ctx.attrs.services)

    args_file = ctx.actions.write("docker-compose-args.txt", run_cmd_args)

    return [
        DefaultInfo(default_output = args_file),
        RunInfo(run_cmd_args),
    ]

tilt_docker_compose_pull = rule(
    impl = tilt_docker_compose_pull_impl,
    attrs = {
        "docker_compose_file": attrs.string(
            default = "docker-compose.yml",
            doc = """The Tiltfile to run.""",
        ),
        "services": attrs.list(
            attrs.string(),
            default = [],
            doc = """Pull Docker images.""",
        ),
    },
)

def tilt_docker_compose_stop_impl(ctx: AnalysisContext) -> list[[DefaultInfo, RunInfo]]:
    docker_compose_file = "{}/{}".format(
        ctx.label.package,
        ctx.attrs.docker_compose_file,
    )

    run_cmd_args = cmd_args([
        "docker",
        "compose",
        "--file",
        docker_compose_file,
        "stop",
    ])
    run_cmd_args.add(ctx.attrs.services)

    args_file = ctx.actions.write("docker-compose-args.txt", run_cmd_args)

    return [
        DefaultInfo(default_output = args_file),
        RunInfo(run_cmd_args),
    ]

tilt_docker_compose_stop = rule(
    impl = tilt_docker_compose_stop_impl,
    attrs = {
        "docker_compose_file": attrs.string(
            default = "docker-compose.yml",
            doc = """The Tiltfile to run.""",
        ),
        "services": attrs.list(
            attrs.string(),
            default = [],
            doc = """Stop specific services.""",
        ),
    },
)

def _invoke_tilt(ctx: AnalysisContext, subcmd: str) -> list[[DefaultInfo, RunInfo]]:
    tiltfile = "{}/{}".format(
        ctx.label.package,
        ctx.attrs.tiltfile,
    )

    run_cmd_args = cmd_args([
        "tilt",
        subcmd,
        "--file",
        tiltfile,
    ])
    run_cmd_args.add(ctx.attrs.tilt_args)
    run_cmd_args.add("--")
    run_cmd_args.add(ctx.attrs.args)

    args_file = ctx.actions.write("tilt-args.txt", run_cmd_args)

    return [
        DefaultInfo(default_output = args_file),
        RunInfo(run_cmd_args),
    ]
