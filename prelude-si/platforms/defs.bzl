# Copyright (c) Meta Platforms, Inc. and affiliates.
#
# This source code is licensed under both the MIT license found in the
# LICENSE-MIT file in the root directory of this source tree and the Apache
# License, Version 2.0 found in the LICENSE-APACHE file in the root directory
# of this source tree.

# This is used to match the correct remote build worker pool
def _get_remote_os_family(constraints: dict) -> str:
    """Get the OS family string for remote execution based on CPU constraints."""
    for constraint_key, constraint_value in constraints.items():
        constraint_value_str = str(constraint_value)
        if "prelude//cpu/constraints:arm64" in constraint_value_str or "cpu/constraints:arm64" in constraint_value_str:
            return "linux-arm64"
    return "linux-x64"

def _execution_platform_impl(ctx: AnalysisContext) -> list[Provider]:
    constraints = dict()
    constraints.update(ctx.attrs.cpu_configuration[ConfigurationInfo].constraints)
    constraints.update(ctx.attrs.os_configuration[ConfigurationInfo].constraints)
    constraints.update(ctx.attrs.rust_build_mode[ConfigurationInfo].constraints)
    cfg = ConfigurationInfo(constraints = constraints, values = {})

    # Check if this is macOS to disable remote builds
    is_macos = False
    for constraint_key, constraint_value in constraints.items():
        constraint_value_str = str(constraint_value)
        if "prelude//os/constraints:macos" in constraint_value_str or "os/constraints:macos" in constraint_value_str:
            is_macos = True
            break

    # Get dynamic OS family for remote execution
    os_family = _get_remote_os_family(constraints)

    name = ctx.label.raw_target()

    remote_enabled = False if (is_macos) else True

    platform = ExecutionPlatformInfo(
        label = name,
        configuration = cfg,
        executor_config = CommandExecutorConfig(
            local_enabled = True,
            remote_enabled = remote_enabled,
            use_limited_hybrid = True,
            remote_cache_enabled = True,
            allow_limited_hybrid_fallbacks = True,
            allow_hybrid_fallbacks_on_failure = True,
            allow_cache_uploads = True,
            remote_output_paths = "output_paths",
            remote_execution_properties = {
                "OSFamily": os_family,
                "container-image": "docker://buildpack-deps:bookworm",
            },
            remote_execution_use_case = "buck2-default",
            use_windows_path_separators = ctx.attrs.use_windows_path_separators,
        ),
    )

    return [
        DefaultInfo(),
        platform,
        PlatformInfo(label = str(name), configuration = cfg),
        ExecutionPlatformRegistrationInfo(platforms = [platform]),
    ]

execution_platform = rule(
    impl = _execution_platform_impl,
    attrs = {
        "cpu_configuration": attrs.dep(providers = [ConfigurationInfo]),
        "os_configuration": attrs.dep(providers = [ConfigurationInfo]),
        "rust_build_mode": attrs.dep(providers = [ConfigurationInfo]),
        "use_windows_path_separators": attrs.bool(),
    },
)

def _host_cpu_configuration() -> str:
    arch = host_info().arch
    if arch.is_aarch64:
        return "prelude//cpu:arm64"
    elif arch.is_arm:
        return "prelude//cpu:arm32"
    elif arch.is_i386:
        return "prelude//cpu:x86_32"
    else:
        return "prelude//cpu:x86_64"

def _host_os_configuration() -> str:
    os = host_info().os
    if os.is_macos:
        return "prelude//os:macos"
    elif os.is_windows:
        return "prelude//os:windows"
    else:
        return "prelude//os:linux"

host_configuration = struct(
    cpu = _host_cpu_configuration(),
    os = _host_os_configuration(),
)
