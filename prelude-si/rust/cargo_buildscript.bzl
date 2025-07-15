# This file adapts prelude/rust/cargo_buildscript.bzl to allow
# wrapping the buildscript runner.

# All the original load statements are still needed
load("@prelude//decls:common.bzl", "buck")
load("@prelude//decls:toolchains_common.bzl", "toolchains_common")
load("@prelude//os_lookup:defs.bzl", "Os", "OsLookup")
load("@prelude//rust:rust_toolchain.bzl", "RustToolchainInfo")
load("@prelude//rust:targets.bzl", "targets")
load("@prelude//utils:cmd_script.bzl", "cmd_script")
load("@prelude//rust:build.bzl", "dependency_args")
load("@prelude//rust:build_params.bzl", "MetadataKind")
load("@prelude//rust:context.bzl", "DepCollectionContext")
load(
    "@prelude//rust:link_info.bzl",
    "DEFAULT_STATIC_LINK_STRATEGY",
    "RustProcMacroPlugin",
    "gather_explicit_sysroot_deps",
    "resolve_rust_deps_inner",
)
load("@prelude//rust:rust_toolchain.bzl", "PanicRuntime")
load(":buildscript_wrapper.bzl", "mise_buildscript_runner")


# _make_rustc_shim and _si_cargo_buildscript_impl are EXACT copies
# of the functions from prelude/rust/cargo_buildscript.bzl.
def _make_rustc_shim(ctx: AnalysisContext, cwd: Artifact) -> cmd_args:
    toolchain_info = ctx.attrs._rust_toolchain[RustToolchainInfo]
    explicit_sysroot_deps = toolchain_info.explicit_sysroot_deps
    if explicit_sysroot_deps:
        dep_ctx = DepCollectionContext(
            advanced_unstable_linking=False,
            include_doc_deps=False,
            is_proc_macro=False,
            explicit_sysroot_deps=explicit_sysroot_deps,
            panic_runtime=PanicRuntime("unwind"))
        deps = gather_explicit_sysroot_deps(dep_ctx)
        deps = resolve_rust_deps_inner(ctx, deps)
        dep_args, _ = dependency_args(
            ctx=ctx,
            compile_ctx=None,
            toolchain_info=toolchain_info,
            deps=deps,
            subdir="any",
            dep_link_strategy=DEFAULT_STATIC_LINK_STRATEGY,
            dep_metadata_kind=MetadataKind("full"),
            is_rustdoc_test=False)
        null_path = "nul" if ctx.attrs._exec_os_type[OsLookup].os == Os(
            "windows") else "/dev/null"
        dep_args = cmd_args("--sysroot=" + null_path,
                            dep_args,
                            relative_to=cwd)
        dep_file, _ = ctx.actions.write("rustc_dep_file",
                                        dep_args,
                                        allow_args=True)
        sysroot_args = cmd_args("@", dep_file, delimiter="", hidden=dep_args)
    else:
        sysroot_args = cmd_args()
    shim = cmd_script(ctx=ctx,
                      name="__rustc_shim",
                      cmd=cmd_args(toolchain_info.compiler,
                                   sysroot_args,
                                   relative_to=cwd),
                      language=ctx.attrs._exec_os_type[OsLookup].script)
    return cmd_args(shim, relative_to=cwd)


def _si_cargo_buildscript_impl(ctx: AnalysisContext) -> list[Provider]:
    toolchain_info = ctx.attrs._rust_toolchain[RustToolchainInfo]
    cwd = ctx.actions.declare_output("cwd", dir=True)
    out_dir = ctx.actions.declare_output("OUT_DIR", dir=True)
    rustc_flags = ctx.actions.declare_output("rustc_flags")
    if ctx.attrs.manifest_dir != None:
        manifest_dir = ctx.attrs.manifest_dir[DefaultInfo].default_outputs[0]
    else:
        manifest_dir = ctx.actions.symlinked_dir(
            "manifest_dir", ctx.attrs.filegroup_for_manifest_dir)
    cmd = [
        ctx.attrs.runner[RunInfo],
        cmd_args("--buildscript=",
                 ctx.attrs.buildscript[RunInfo],
                 delimiter=""),
        cmd_args("--rustc-cfg=",
                 ctx.attrs.rustc_cfg[DefaultInfo].default_outputs[0],
                 delimiter=""),
        cmd_args("--manifest-dir=", manifest_dir, delimiter=""),
        cmd_args("--create-cwd=", cwd.as_output(), delimiter=""),
        cmd_args("--outfile=", rustc_flags.as_output(), delimiter="")
    ]
    env = {}
    env["CARGO"] = "/bin/false"
    env["CARGO_PKG_NAME"] = ctx.attrs.package_name
    env["CARGO_PKG_VERSION"] = ctx.attrs.version
    env["OUT_DIR"] = out_dir.as_output()
    env["RUSTC"] = _make_rustc_shim(ctx, cwd)
    env["RUSTC_LINKER"] = "/bin/false"
    env["RUST_BACKTRACE"] = "1"
    if toolchain_info.rustc_target_triple:
        env["TARGET"] = toolchain_info.rustc_target_triple
    else:
        cmd.append(
            cmd_args(
                "--rustc-host-tuple=",
                ctx.attrs.rustc_host_tuple[DefaultInfo].default_outputs[0],
                delimiter=""))
    env["CARGO_ENCODED_RUSTFLAGS"] = cmd_args(toolchain_info.rustc_flags,
                                              delimiter="\037")
    host_triple = targets.exec_triple(ctx)
    if host_triple:
        env["HOST"] = host_triple
    for feature in ctx.attrs.features:
        upper_feature = feature.upper().replace("-", "_")
        env["CARGO_FEATURE_{}".format(upper_feature)] = "1"
    for k, v in ctx.attrs.env.items():
        env[k] = cmd_args(v, relative_to=cwd)
    ctx.actions.run(cmd, env=env, category="buildscript", prefer_remote=True)
    return [
        DefaultInfo(default_output=None,
                    sub_targets={
                        "out_dir": [DefaultInfo(default_output=out_dir)],
                        "rustc_flags":
                        [DefaultInfo(default_output=rustc_flags)]
                    })
    ]


_si_cargo_buildscript_rule = rule(
    impl=_si_cargo_buildscript_impl,
    attrs={
        "buildscript":
        attrs.exec_dep(providers=[RunInfo]),
        "env":
        attrs.dict(key=attrs.string(), value=attrs.arg()),
        "features":
        attrs.list(attrs.string()),
        "filegroup_for_manifest_dir":
        attrs.option(attrs.dict(key=attrs.string(), value=attrs.source()),
                     default=None),
        "manifest_dir":
        attrs.option(attrs.dep(), default=None),
        "package_name":
        attrs.string(),
        # The runner is no longer default_only.
        "runner":
        attrs.exec_dep(providers=[RunInfo],
                       default="prelude//rust/tools:buildscript_run"),
        "rustc_cfg":
        attrs.dep(default="prelude//rust/tools:rustc_cfg"),
        "rustc_host_tuple":
        attrs.dep(default="prelude//rust/tools:rustc_host_tuple"),
        "version":
        attrs.string(),
        "_exec_os_type":
        buck.exec_os_type_arg(),
        "_rust_toolchain":
        toolchains_common.rust(),
    },
    uses_plugins=[RustProcMacroPlugin],
)


# This is the public macro that users will call from BUCK files.
# --- THIS IS THE CORRECTED PART ---
def buildscript_run(name,
                    buildscript_rule,
                    package_name,
                    version,
                    features=[],
                    env={},
                    local_manifest_dir=None,
                    manifest_dir=None,
                    **kwargs):
    if manifest_dir == None and local_manifest_dir == None:
        existing_filegroup_name = "{}-{}.crate".format(package_name, version)
        if rule_exists(existing_filegroup_name):
            manifest_dir = ":{}".format(existing_filegroup_name)
        else:
            local_manifest_dir = "vendor/{}-{}".format(package_name, version)

    filegroup_for_manifest_dir = None
    if local_manifest_dir != None:
        prefix_with_trailing_slash = "{}/".format(local_manifest_dir)
        filegroup_for_manifest_dir = {
            path.removeprefix(prefix_with_trailing_slash): path
            for path in glob(["{}/**".format(local_manifest_dir)])
        }

    runner_name = name + "-runner"
    mise_buildscript_runner(
        name=runner_name,
        visibility=["PUBLIC"],
    )

    _si_cargo_buildscript_rule(
        name=name,
        buildscript=buildscript_rule,
        package_name=package_name,
        version=version,
        features=features,
        env=env,
        filegroup_for_manifest_dir=filegroup_for_manifest_dir,
        manifest_dir=manifest_dir,
        runner=":" + runner_name,
        _rust_toolchain="toolchains//:rust",
        **kwargs)
