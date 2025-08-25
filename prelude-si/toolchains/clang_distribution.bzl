"""Hermetic Clang/LLVM toolchain distribution rules using S3 artifacts.

Downloads pre-packaged Clang toolchains from si-artifacts-prod.
Includes libgcc in unified archives where needed.
"""

load("@prelude//cxx:cxx_toolchain_types.bzl",
     "BinaryUtilitiesInfo", "CCompilerInfo", "CxxCompilerInfo", 
     "LinkerInfo", "LinkerType", "ShlibInterfacesMode",
     "CxxInternalTools", "StripFlagsInfo",
     "cxx_toolchain_infos")
load("@prelude//cxx:headers.bzl", "HeaderMode")
load("@prelude//cxx:linker.bzl", "is_pdb_generated")
load("@prelude//linking:link_info.bzl", "LinkStyle")
load("@prelude-si//toolchains:common.bzl", "create_download_distribution_function", "create_distribution_provider")
load("@prelude-si//toolchains:extraction.bzl", "ToolchainExtractionInfo")

# Clang version checksums for our S3 artifacts
_CLANG_S3_CHECKSUMS = {
    "20.1.0": {
        "linux": {
            "x86_64": "1aa13150f61144bb4718aab8238f7d7239741534bf064952c4eca6c630000a3d",
            "aarch64": "d3a41c8d2cc6e9ba98a6fb7ebb4bdfa6494788503c85d90849d0ec6894474d7f",
        },
        "darwin": {
            "aarch64": "8ceb584b7e38743274eb2ceae15a0e7562e1c64c7b87f86cfe82cc0a657f5787",
        },
    },
}

# Create provider using shared utility  
SimpleClangDistributionInfo = create_distribution_provider({
    "directory": provider_field(typing.Any, default = None),
})

def _clang_distribution_impl(ctx: AnalysisContext) -> list[Provider]:
    """Create Clang distribution from extracted S3 toolchain."""

    extraction = ctx.attrs.extraction[ToolchainExtractionInfo]
    
    # With unified structure, the complete LLVM/Clang installation 
    # (including libgcc where needed) is in toolchain/
    directory = extraction.toolchain_dir

    return [
        DefaultInfo(),
        SimpleClangDistributionInfo(
            version = ctx.attrs.version,
            target = ctx.attrs.target,
            directory = directory,
        ),
    ]

clang_distribution = rule(
    impl = _clang_distribution_impl,
    attrs = {
        "version": attrs.string(),
        "target": attrs.string(), 
        "extraction": attrs.dep(providers = [ToolchainExtractionInfo]),
    },
)

# Create download function using shared utility
download_clang_distribution = create_download_distribution_function(
    family = "clang",
    checksums_dict = _CLANG_S3_CHECKSUMS,
    distribution_rule = clang_distribution,
    toolchain_name = "Clang"
)

def _get_linker_type(target: str) -> LinkerType:
    """Determine linker type based on target."""
    if "linux" in target:
        return LinkerType("gnu")
    elif "darwin" in target or "apple" in target:
        return LinkerType("darwin")
    elif "windows" in target:
        return LinkerType("windows")
    else:
        return LinkerType("gnu")  # Default

def _hermetic_clang_toolchain_impl(ctx: AnalysisContext) -> list[Provider]:
    """Create a hermetic Clang toolchain from a distribution."""

    dist = ctx.attrs.distribution[SimpleClangDistributionInfo]

    # Create single wrapper script that can handle all LLVM tools
    # Takes clang directory and tool name as arguments
    clang_wrapper = ctx.actions.declare_output("clang_wrapper.sh")
    ctx.actions.write(
        clang_wrapper,
        [
            "#!/bin/bash",
            "# Generic wrapper script for LLVM tools with LD_LIBRARY_PATH set to bundled libraries",
            "CLANG_DIR=\"$1\"",
            "TOOL=\"$2\"",
            "shift 2",  # Remove the first two arguments (CLANG_DIR and TOOL) from $@
            "export LD_LIBRARY_PATH=\"$CLANG_DIR/lib:${LD_LIBRARY_PATH:-}\"",
            "# Disable terminal features to avoid libtinfo.so.5 dependency",
            "export NO_COLOR=1",
            "export TERM=dumb",
            "",
            "# Handle llvm-ar compatibility with GNU ar arguments",
            "if [ \"$TOOL\" = \"llvm-ar\" ]; then",
            "  # Convert GNU ar style arguments to LLVM ar format",
            "  first_arg=\"$1\"",
            "  if [[ \"$first_arg\" =~ ^[a-z]*r[a-z]*$ ]]; then",
            "    # Replace GNU ar flags like 'rf', 'rcs' with just 'r' and add 'c' modifier separately",
            "    shift",
            "    exec \"$CLANG_DIR/bin/llvm-ar\" r \"$@\"",
            "  fi",
            "fi",
            "",
            "# For compilers and linkers on macOS, add system SDK path",
            "if [[ \"$TOOL\" == \"clang\" || \"$TOOL\" == \"clang++\" ]]; then",
            "  if [[ \"$OSTYPE\" == \"darwin\"* ]]; then",
            "    # Try multiple methods to find the SDK",
            "    SDK_PATH=\"\"",
            "    # Method 1: xcrun (works in most environments)",
            "    if command -v xcrun >/dev/null 2>&1; then",
            "      SDK_PATH=\"$(xcrun --show-sdk-path 2>/dev/null)\"",
            "    fi",
            "    # Method 2: Standard Xcode paths",
            "    if [ -z \"$SDK_PATH\" ] || [ ! -d \"$SDK_PATH\" ]; then",
            "      for path in /Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX*.sdk /Library/Developer/CommandLineTools/SDKs/MacOSX*.sdk; do",
            "        if [ -d \"$path\" ]; then",
            "          SDK_PATH=\"$path\"",
            "          break",
            "        fi",
            "      done",
            "    fi",
            "    # If we found a valid SDK, use it",
            "    if [ -n \"$SDK_PATH\" ] && [ -d \"$SDK_PATH\" ]; then",
            "      # Check if this is a linking operation by looking for linking flags",
            "      if [[ \" $* \" =~ \" -o \".*\" \" ]] && [[ ! \" $* \" =~ \" -c \" ]]; then",
            "        # This is linking, add library search path",
            "        exec \"$CLANG_DIR/bin/$TOOL\" -fno-color-diagnostics --sysroot=\"$SDK_PATH\" -L\"$SDK_PATH/usr/lib\" \"$@\"",
            "      else",
            "        # This is compilation, only add sysroot",
            "        exec \"$CLANG_DIR/bin/$TOOL\" -fno-color-diagnostics --sysroot=\"$SDK_PATH\" \"$@\"",
            "      fi",
            "    else",
            "      exec \"$CLANG_DIR/bin/$TOOL\" -fno-color-diagnostics \"$@\"",
            "    fi",
            "  else",
            "    exec \"$CLANG_DIR/bin/$TOOL\" -fno-color-diagnostics \"$@\"",
            "  fi",
            "else",
            "  exec \"$CLANG_DIR/bin/$TOOL\" \"$@\"",
            "fi",
        ],
        is_executable = True,
    )

    # libgcc path is now included directly in the unified archive for Linux x86_64
    # For Darwin, add clang runtime library path
    libgcc_linker_flags = []
    if "x86_64-unknown-linux-gnu" == dist.target:
        libgcc_linker_flags = [cmd_args(dist.directory, format = "-L{}/lib/x86_64-linux-gnu")]
    elif "aarch64-apple-darwin" == dist.target:
        libgcc_linker_flags = [cmd_args(dist.directory, format = "-L{}/lib/clang/20/lib/darwin")]

    return [ctx.attrs.distribution[DefaultInfo]] + cxx_toolchain_infos(
        internal_tools = ctx.attrs._cxx_internal_tools[CxxInternalTools],
        platform_name = dist.target,
        c_compiler_info = CCompilerInfo(
            compiler = RunInfo(args = cmd_args(clang_wrapper, dist.directory, "clang", hidden = [dist.directory])),
            compiler_type = "clang",
            compiler_flags = cmd_args(ctx.attrs.c_compiler_flags),
            preprocessor_flags = cmd_args(ctx.attrs.c_preprocessor_flags),
        ),
        as_compiler_info = CCompilerInfo(
            compiler = RunInfo(args = cmd_args(clang_wrapper, dist.directory, "clang", hidden = [dist.directory])),
            compiler_type = "clang",
            compiler_flags = cmd_args(ctx.attrs.c_compiler_flags),
            preprocessor_flags = cmd_args(ctx.attrs.c_preprocessor_flags),
        ),
        asm_compiler_info = CCompilerInfo(
            compiler = RunInfo(args = cmd_args(clang_wrapper, dist.directory, "clang", hidden = [dist.directory])),
            compiler_type = "clang",
            compiler_flags = cmd_args(ctx.attrs.c_compiler_flags),
            preprocessor_flags = cmd_args(ctx.attrs.c_preprocessor_flags),
        ),
        cxx_compiler_info = CxxCompilerInfo(
            compiler = RunInfo(args = cmd_args(clang_wrapper, dist.directory, "clang++", hidden = [dist.directory])),
            compiler_type = "clang",
            compiler_flags = cmd_args(ctx.attrs.cxx_compiler_flags),
            preprocessor_flags = cmd_args(ctx.attrs.cxx_preprocessor_flags),
        ),
        linker_info = LinkerInfo(
            archiver = RunInfo(args = cmd_args(clang_wrapper, dist.directory, "llvm-ar", hidden = [dist.directory])),
            archiver_type = "gnu",
            archiver_supports_argfiles = True,
            archive_objects_locally = False,
            binary_extension = "",
            binary_linker_flags = cmd_args(
                ctx.attrs.binary_linker_flags,
                libgcc_linker_flags,
            ),
            generate_linker_maps = False,
            link_binaries_locally = False,
            link_libraries_locally = False,
            link_style = LinkStyle(ctx.attrs.link_style),
            link_weight = 1,
            linker = RunInfo(args = cmd_args(clang_wrapper, dist.directory, "clang++", hidden = [dist.directory])),
            linker_flags = cmd_args(ctx.attrs.linker_flags),
            object_file_extension = "o",
            shlib_interfaces = ShlibInterfacesMode("disabled"),
            shared_dep_runtime_ld_flags = ctx.attrs.shared_dep_runtime_ld_flags,
            shared_library_name_default_prefix = "lib",
            shared_library_name_format = "lib{}.so",
            shared_library_versioned_name_format = "lib{}.{}.so",
            static_dep_runtime_ld_flags = ctx.attrs.static_dep_runtime_ld_flags,
            static_library_extension = "a",
            static_pic_dep_runtime_ld_flags = ctx.attrs.static_pic_dep_runtime_ld_flags,
            independent_shlib_interface_linker_flags = [],
            type = _get_linker_type(dist.target),
            use_archiver_flags = True,
            is_pdb_generated = is_pdb_generated(_get_linker_type(dist.target), ctx.attrs.linker_flags),
        ),
        binary_utilities_info = BinaryUtilitiesInfo(
            bolt_msdk = None,
            dwp = None,
            nm = RunInfo(args = cmd_args(clang_wrapper, dist.directory, "llvm-nm", hidden = [dist.directory])),
            objcopy = RunInfo(args = cmd_args(clang_wrapper, dist.directory, "llvm-objcopy", hidden = [dist.directory])),
            ranlib = RunInfo(args = cmd_args(clang_wrapper, dist.directory, "llvm-ranlib", hidden = [dist.directory])),
            strip = RunInfo(args = cmd_args(clang_wrapper, dist.directory, "llvm-strip", hidden = [dist.directory])),
        ),
        header_mode = HeaderMode("symlink_tree_only"),
        strip_flags_info = StripFlagsInfo(
            strip_debug_flags = [],
            strip_non_global_flags = [],
            strip_all_flags = [],
        ),
    )

hermetic_clang_toolchain = rule(
    impl = _hermetic_clang_toolchain_impl,
    attrs = {
        "distribution": attrs.exec_dep(providers = [SimpleClangDistributionInfo]),
        "binary_linker_flags": attrs.list(attrs.arg(), default = []),
        "c_compiler_flags": attrs.list(attrs.arg(), default = []),
        "c_preprocessor_flags": attrs.list(attrs.arg(), default = []),
        "cxx_compiler_flags": attrs.list(attrs.arg(), default = []),
        "cxx_preprocessor_flags": attrs.list(attrs.arg(), default = []),
        "link_style": attrs.enum(
            LinkStyle.values(),
            default = "static",
        ),
        "linker_flags": attrs.list(attrs.arg(), default = []),
        "shared_dep_runtime_ld_flags": attrs.list(attrs.arg(), default = []),
        "static_dep_runtime_ld_flags": attrs.list(attrs.arg(), default = []),
        "static_pic_dep_runtime_ld_flags": attrs.list(attrs.arg(), default = []),
        "_cxx_internal_tools": attrs.default_only(attrs.dep(providers = [CxxInternalTools], default = "prelude//cxx/tools:internal_tools")),
    },
    is_toolchain_rule = True,
)