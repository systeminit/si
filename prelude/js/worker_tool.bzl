# Copyright (c) Meta Platforms, Inc. and affiliates.
#
# This source code is licensed under both the MIT license found in the
# LICENSE-MIT file in the root directory of this source tree and the Apache
# License, Version 2.0 found in the LICENSE-APACHE file in the root directory
# of this source tree.

load("@prelude//utils:expect.bzl", "expect")

WorkerToolInfo = provider(
    fields = {
        "command": provider_field(typing.Any, default = None),  # cmd_args
    },
)

def worker_tool(ctx: AnalysisContext) -> list[Provider]:
    """
     worker_tool() rule implementation

    Args:
        ctx: rule analysis context
    Returns:
        list of created providers (DefaultInfo with an empty output and TemplatePlaceholderInfo with $(worker) macro key)
    """

    executable = ctx.attrs.exe
    worker_tool_run_info = executable[RunInfo]
    expect(worker_tool_run_info != None, "Worker tool executable must have a RunInfo!")

    worker_tool_runner = ctx.attrs._worker_tool_runner[RunInfo]
    worker_tool_cmd = [worker_tool_runner]
    worker_tool_cmd.append("--worker-tool")
    worker_tool_cmd.append(worker_tool_run_info)

    worker_args = ctx.attrs.args
    if worker_args:
        worker_args_file, _ = ctx.actions.write(
            "worker_tool_args",
            worker_args,
            allow_args = True,
        )

        worker_tool_cmd.append("--worker-args-file")
        worker_tool_cmd.append(worker_args_file)

    worker_env = ctx.attrs.env
    if worker_env:
        env_args = []
        for key, value in worker_env.items():
            env_args.append(key)
            env_args.append(value)

        env_args_file, _ = ctx.actions.write(
            "worker_tool_envs",
            env_args,
            allow_args = True,
        )

        worker_tool_cmd.append("--worker-env-file")
        worker_tool_cmd.append(env_args_file)

    worker_tool_cmd = cmd_args(worker_tool_cmd)
    return [
        DefaultInfo(),
        RunInfo(
            args = worker_tool_cmd,
        ),
        WorkerToolInfo(
            command = worker_tool_cmd,
        ),
    ]
