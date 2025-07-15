load("@prelude//toolchains:cxx.bzl", "CxxToolsInfo", "cxx_tools_info_toolchain")
load("@prelude-si//:mise.bzl", "MiseInfo")
load("@prelude//decls:common.bzl", "buck")
load("@prelude//os_lookup:defs.bzl", "Os", "OsLookup")
load("@prelude//cxx:cxx_toolchain_types.bzl", "LinkerType", "CxxPlatformInfo", "CxxToolchainInfo",
     "BinaryUtilitiesInfo", "CxxCompilerInfo", "CCompilerInfo", "LinkerInfo", "ShlibInterfacesMode",
     "CxxInternalTools", "PicBehavior")
load("@prelude//linking:link_info.bzl", "LinkStyle")
load("@prelude//cxx:headers.bzl", "HeaderMode")

def _mise_cxx_tools_impl(ctx):
    mise_info = ctx.attrs.mise_install[MiseInfo]
    mise_binary = cmd_args(mise_info.mise_bootstrap)
    shims = cmd_args(mise_info.mise_tools_dir, "/shims", delimiter="")
    wrapper_tool = ctx.attrs._wrapper[RunInfo]
    compiler_args = cmd_args(wrapper_tool)
    compiler_args.add(shims)

    compiler_path = cmd_args(
        [compiler_args, mise_binary, "exec", "vfox:https://github.com/systeminit/vfox-clang@20.1.7", "--", "clang"]
    )
    cxx_compiler_path = cmd_args(
        [compiler_args, mise_binary, "exec", "vfox:https://github.com/systeminit/vfox-clang@20.1.7", "--", "clang++"]
    )

    os = ctx.attrs._target_os_type[OsLookup]
    target_name = os.os.value
    if os.cpu:
        target_name += "-" + os.cpu

    # Set platform-specific values
    if os.os == Os("macos"):
        pic_behavior = PicBehavior("always_enabled")
        shared_lib_ext = ".dylib"
        shared_lib_format = "lib{}" + shared_lib_ext
        shared_lib_versioned_format = "lib{}.{}" + shared_lib_ext
        shared_lib_prefix = "lib"
    elif os.os == Os("windows"):
        pic_behavior = PicBehavior("not_supported")
        shared_lib_ext = ".dll"
        shared_lib_format = "{}" + shared_lib_ext
        shared_lib_versioned_format = "{}.{}" + shared_lib_ext
        shared_lib_prefix = ""
    else:  # Linux and others
        pic_behavior = PicBehavior("supported")
        shared_lib_ext = ".so"
        shared_lib_format = "lib{}" + shared_lib_ext
        shared_lib_versioned_format = "lib{}.{}" + shared_lib_ext
        shared_lib_prefix = "lib"

    return [
        DefaultInfo(),
        CxxToolsInfo(
            compiler = compiler_path,
            compiler_type = "clang",
            cxx_compiler = cxx_compiler_path,
            asm_compiler = compiler_path,
            asm_compiler_type = "clang",
            linker = cxx_compiler_path,
            linker_type = LinkerType("gnu"),
            archiver = "ar",
            archiver_type = "gnu",
            rc_compiler = None,
            cvtres_compiler = None,
        ),
        CxxPlatformInfo(name = target_name),
        CxxToolchainInfo(
            internal_tools = ctx.attrs.internal_tools[CxxInternalTools],
            linker_info = LinkerInfo(
                linker = RunInfo(args = [cxx_compiler_path]),
                linker_flags = [],
                archiver = RunInfo(args = ["ar"]),
                archiver_type = "gnu",
                archiver_supports_argfiles = True,
                link_style = LinkStyle("static_pic"),
                link_weight = 1,
                binary_extension = "",
                shared_library_name_format = shared_lib_format,
                shared_library_versioned_name_format = shared_lib_versioned_format,
                shared_library_name_default_prefix = shared_lib_prefix,  # Add this
                static_library_extension = "a",
                object_file_extension = "o",
                type = LinkerType("gnu"),
                shlib_interfaces = ShlibInterfacesMode("disabled"),
                link_binaries_locally = True,
                link_libraries_locally = True,
                archive_objects_locally = True,
                use_archiver_flags = True,
                static_dep_runtime_ld_flags = [],
                static_pic_dep_runtime_ld_flags = [],
                shared_dep_runtime_ld_flags = [],
                independent_shlib_interface_linker_flags = [],
                force_full_hybrid_if_capable = False,
                post_linker_flags = [],
                generate_linker_maps = False,
            ),
            binary_utilities_info = BinaryUtilitiesInfo(
                nm = RunInfo(args = ["nm"]),
                strip = RunInfo(args = ["strip"]),
                dwp = None,
                objcopy = RunInfo(args = ["objcopy"]),
                objdump = RunInfo(args = ["objdump"]),
                ranlib = RunInfo(args = ["ranlib"]),
            ),
            cxx_compiler_info = CxxCompilerInfo(
                compiler = RunInfo(args = [cxx_compiler_path]),
                compiler_flags = [],
                compiler_type = "clang",
            ),
            c_compiler_info = CCompilerInfo(
                compiler = RunInfo(args = [compiler_path]),
                compiler_flags = [],
                compiler_type = "clang",
            ),
            as_compiler_info = CCompilerInfo(
                compiler = RunInfo(args = [compiler_path]),
                compiler_type = "clang",
            ),
            header_mode = HeaderMode("symlink_tree_only"),
            cpp_dep_tracking_mode = "makefile",
            pic_behavior = pic_behavior,
        ),
    ]

mise_cxx_toolchain = rule(
    impl = _mise_cxx_tools_impl,
    attrs = {
        "mise_install": attrs.dep(providers = [MiseInfo]),
        "_wrapper": attrs.default_only(attrs.dep(providers = [RunInfo], default = "toolchains//:mise_wrapper")),
        "_target_os_type": buck.target_os_type_arg(),
        "internal_tools": attrs.exec_dep(providers = [CxxInternalTools], default = "prelude//cxx/tools:internal_tools"),
    },
    is_toolchain_rule = True,
)
