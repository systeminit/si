# Copyright (c) Meta Platforms, Inc. and affiliates.
#
# This source code is licensed under both the MIT license found in the
# LICENSE-MIT file in the root directory of this source tree and the Apache
# License, Version 2.0 found in the LICENSE-APACHE file in the root directory
# of this source tree.

load("@prelude//:paths.bzl", "paths")
load(":erlang_build.bzl", "erlang_build")
load(":erlang_dependencies.bzl", "flatten_dependencies")
load(":erlang_info.bzl", "ErlangAppInfo")
load(
    ":erlang_toolchain.bzl",
    "Toolchain",  # @unused Used as type
    "get_primary",
    "select_toolchains",
)
load(":erlang_utils.bzl", "action_identifier")

def erlang_escript_impl(ctx: AnalysisContext) -> list[Provider]:
    # select the correct tools from the toolchain
    toolchain = select_toolchains(ctx)[get_primary(ctx)]

    # collect all dependencies
    dependencies = flatten_dependencies(ctx, ctx.attrs.deps)
    toolchain_name = get_primary(ctx)
    artifacts = {}

    for dep in dependencies.values():
        if ErlangAppInfo not in dep:
            # skip extra includes
            continue
        dep_info = dep[ErlangAppInfo]
        if dep_info.virtual:
            # skip virtual apps
            continue
        app_folder = dep_info.app_folders[toolchain_name]

        artifacts[_ebin_path(dep_info.name)] = app_folder.project("ebin")
        if ctx.attrs.include_priv:
            artifacts[_priv_path(dep_info.name)] = app_folder.project("priv")

    # additional resources
    for res in ctx.attrs.resources:
        for artifact in res[DefaultInfo].default_outputs + res[DefaultInfo].other_outputs:
            if artifact.short_path in artifacts:
                fail("multiple artifacts defined for path %s", (artifact.short_path))
            artifacts[artifact.short_path] = artifact

    escript_name = _escript_name(ctx)
    output = ctx.actions.declare_output(escript_name)

    args = ctx.attrs.emu_args

    config_files = _escript_config_files(ctx)
    for config_file in config_files:
        artifacts[config_file.short_path] = config_file

    escript_trampoline = build_escript_bundled_trampoline(ctx, toolchain, config_files)
    artifacts[escript_trampoline.basename] = escript_trampoline

    args += ["-escript", "main", "erlang_escript_trampoline"]

    escript_build_spec = {
        "artifacts": artifacts,
        "emu_args": args,
        "output": output.as_output(),
    }

    spec_file = ctx.actions.write_json(
        "escript_build_spec.json",
        escript_build_spec,
        with_inputs = True,
    )

    create_escript(ctx, spec_file, toolchain, escript_name)

    escript_cmd = cmd_args(
        [
            toolchain.otp_binaries.escript,
            output,
        ],
    )

    return [
        DefaultInfo(default_output = output),
        RunInfo(escript_cmd),
    ]

def create_escript(
        ctx: AnalysisContext,
        spec_file: WriteJsonCliArgs,
        toolchain: Toolchain,
        escript_name: str) -> None:
    """ build the escript with the escript builder tool
    """

    erlang_build.utils.run_with_env(
        ctx,
        toolchain,
        cmd_args(toolchain.escript_builder, spec_file),
        category = "escript",
        identifier = action_identifier(toolchain, escript_name),
    )
    return None

def _escript_name(ctx: AnalysisContext) -> str:
    if ctx.attrs.script_name:
        return ctx.attrs.script_name
    else:
        return ctx.attrs.name + ".escript"

def _main_module(ctx: AnalysisContext) -> str:
    if ctx.attrs.main_module:
        return ctx.attrs.main_module
    else:
        return ctx.attrs.name

def build_escript_unbundled_trampoline(ctx: AnalysisContext, toolchain, config_files: list[Artifact]) -> Artifact:
    data = cmd_args()

    data.add("#!/usr/bin/env escript")
    data.add("%% -*- erlang -*-")
    data.add("%%! {}".format(" ".join(ctx.attrs.emu_args)))

    data.add("-module('{}').".format(_escript_name(ctx)))
    data.add("-export([main/1]).")
    data.add("main(Args) ->")
    data.add("EscriptDir = filename:dirname(escript:script_name()),")
    data.add(_config_files_code_to_erl(config_files))
    data.add('    EBinDirs = filelib:wildcard(filename:join([EscriptDir, "lib", "*", "ebin"])),')
    data.add("    code:add_paths(EBinDirs),")
    data.add("    {}:main(Args).".format(_main_module(ctx)))
    data.add(_parse_bin())

    return ctx.actions.write(
        paths.join(erlang_build.utils.build_dir(toolchain), "run.escript"),
        data,
        is_executable = True,
    )

def build_escript_bundled_trampoline(ctx: AnalysisContext, toolchain, config_files: list[Artifact]) -> Artifact:
    data = cmd_args()

    data.add("-module('erlang_escript_trampoline').")
    data.add("-export([main/1]).")
    data.add("main(Args) ->")
    data.add("EscriptDir = escript:script_name(),")
    data.add(_config_files_code_to_erl(config_files))
    data.add("    {}:main(Args).".format(_main_module(ctx)))
    data.add(_parse_bin())
    escript_trampoline_erl = ctx.actions.write(
        paths.join(erlang_build.utils.build_dir(toolchain), "erlang_escript_trampoline.erl"),
        data,
    )
    my_output = ctx.actions.declare_output("erlang_escript_trampoline.beam")

    ctx.actions.run(
        cmd_args(
            toolchain.otp_binaries.erlc,
            "-o",
            cmd_args(my_output.as_output(), parent = 1),
            escript_trampoline_erl,
        ),
        category = "erlc_escript_trampoline",
    )

    return my_output

def _ebin_path(app_name: str) -> str:
    return paths.join(app_name, "ebin")

def _priv_path(app_name: str) -> str:
    return paths.join(app_name, "priv")

def _escript_config_files(ctx: AnalysisContext) -> list[Artifact]:
    config_files = []
    for config_dep in ctx.attrs.configs:
        for artifact in config_dep[DefaultInfo].default_outputs + config_dep[DefaultInfo].other_outputs:
            (_, ext) = paths.split_extension(artifact.short_path)
            if ext == ".config":
                config_files.append(artifact)
    return config_files

def _config_files_code_to_erl(config_files: list[Artifact]) -> list[str]:
    cmd = []
    cmd.append("ConfigFiles = [")
    for i in range(0, len(config_files)):
        cmd.append('"{}"'.format(config_files[i].short_path))
        if i < len(config_files) - 1:
            cmd.append(",")
    cmd.append("],")
    cmd.append("[begin ")
    cmd.append("{ok, AppConfigBin, _FullName} = erl_prim_loader:get_file(filename:join(EscriptDir, ConfigFile)),")
    cmd.append("{ok, AppConfig} = parse_bin(AppConfigBin), ")
    cmd.append(" ok = application:set_env(AppConfig, [{persistent, true}])")
    cmd.append("end || ConfigFile <- ConfigFiles],")
    return cmd

def _parse_bin() -> str:
    return """
parse_bin(<<"">>) ->
    [];
parse_bin(Bin) ->
        {ok, Tokens, _} = erl_scan:string(binary_to_list(Bin)),
        erl_parse:parse_term(Tokens).
        """
