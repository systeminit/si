"""Simple hermetic Clang/LLVM toolchain that uses the complete distribution."""

load("@prelude//cxx:cxx_toolchain_types.bzl",
     "BinaryUtilitiesInfo", "CCompilerInfo", "CxxCompilerInfo",
     "LinkerInfo", "LinkerType", "ShlibInterfacesMode",
     "CxxInternalTools", "StripFlagsInfo",
     "cxx_toolchain_infos")
load("@prelude//cxx:headers.bzl", "HeaderMode")
load("@prelude//cxx:linker.bzl", "is_pdb_generated")
load("@prelude//linking:link_info.bzl", "LinkStyle")

# Clang release information with checksums
_CLANG_RELEASES = {
    "20.1.0": {
        "x86_64-unknown-linux-gnu": {
            "url": "https://github.com/llvm/llvm-project/releases/download/llvmorg-20.1.0/LLVM-20.1.0-Linux-X64.tar.xz",
            "sha1": "6a78ec27bbb7f6190a27bb7bb9c72a2e53b57a8a",
        },
        "aarch64-unknown-linux-gnu": {
            "url": "https://github.com/llvm/llvm-project/releases/download/llvmorg-20.1.0/LLVM-20.1.0-Linux-ARM64.tar.xz",
            "sha1": "022891b29eef77bd7036ec89c550b5db82a0f3ef",
        },
        "x86_64-apple-darwin": {
            "url": "https://github.com/llvm/llvm-project/releases/download/llvmorg-20.1.0/LLVM-20.1.0-macOS-X64.tar.xz",
            "sha1": "",  # TODO: Add real checksum
        },
        "aarch64-apple-darwin": {
            "url": "https://github.com/llvm/llvm-project/releases/download/llvmorg-20.1.0/LLVM-20.1.0-macOS-AArch64.tar.xz",
            "sha1": "",  # TODO: Add real checksum
        },
    },
}

def _get_clang_release(version: str, target: str):
    if version not in _CLANG_RELEASES:
        fail("Unknown Clang version '{}'. Available versions: {}".format(
            version,
            ", ".join(_CLANG_RELEASES.keys()),
        ))

    clang_version = _CLANG_RELEASES[version]
    if target not in clang_version:
        fail("Unsupported target '{}' for Clang {}. Supported targets: {}".format(
            target,
            version,
            ", ".join(clang_version.keys()),
        ))

    return clang_version[target]

# Simple HTTP archive implementation
def _simple_http_archive_impl(ctx: AnalysisContext) -> list[Provider]:
    """Download and extract a tar.xz archive using bundled static xz binary."""
    url = ctx.attrs.urls[0]

    # Download archive
    archive = ctx.actions.declare_output("archive.tar.xz")
    if ctx.attrs.sha256:
        ctx.actions.download_file(archive.as_output(), url, sha256 = ctx.attrs.sha256)
    elif ctx.attrs.sha1:
        ctx.actions.download_file(archive.as_output(), url, sha1 = ctx.attrs.sha1)
    else:
        fail("Must provide either sha256 or sha1 checksum")

    # Extract archive using system xz command (available in buildpack-deps)
    output = ctx.actions.declare_output(ctx.label.name, dir = True)
    script = [
        "mkdir -p $1",
        "tar xJf $2 -C $1 --strip-components=0"
    ]
    ctx.actions.run([
        "sh", "-c", "; ".join(script), "--", output.as_output(), archive
    ], category = "extract_clang")

    return [DefaultInfo(default_output = output)]

_simple_http_archive = rule(
    impl = _simple_http_archive_impl,
    attrs = {
        "urls": attrs.list(attrs.string()),
        "sha256": attrs.string(default = ""),
        "sha1": attrs.string(default = ""),
    },
)

SimpleClangDistributionInfo = provider(
    fields = {
        "version": provider_field(typing.Any, default = None),
        "target": provider_field(typing.Any, default = None),
        "directory": provider_field(typing.Any, default = None),
    },
)

def _hermetic_clang_distribution_impl(ctx: AnalysisContext) -> list[Provider]:
    """Extract complete Clang toolchain from downloaded archive."""

    # Create single output directory containing the complete clang distribution
    output_dir = ctx.actions.declare_output(ctx.label.name, dir = True)

    # Path to the extracted archive
    archive_path = cmd_args(ctx.attrs.archive[DefaultInfo].default_outputs[0])

    # Simple setup script that copies the entire clang distribution
    setup_script = ctx.actions.declare_output("setup.sh")
    ctx.actions.write(
        setup_script,
        [
            "#!/bin/bash",
            "set -e",
            "ARCHIVE_DIR=$1",
            "OUTPUT_DIR=$2",
            "",
            "echo 'Copying complete LLVM/Clang distribution for hermetic operation'",
            "",
            "# Ensure output directory exists",
            "mkdir -p \"$OUTPUT_DIR\"",
            "",
            "# Copy the entire LLVM directory as-is",
            "cp -r $ARCHIVE_DIR/*/* $OUTPUT_DIR/ 2>/dev/null || cp -r $ARCHIVE_DIR/* $OUTPUT_DIR/",
            "",
            "echo 'Complete LLVM/Clang distribution ready'",
        ],
        is_executable = True,
    )

    # Run the setup script
    ctx.actions.run(
        [setup_script, archive_path, output_dir.as_output()],
        category = "clang_extract",
        identifier = "setup",
    )

    return [
        DefaultInfo(default_output = output_dir),
        SimpleClangDistributionInfo(
            version = ctx.attrs.version,
            target = ctx.attrs.target,
            directory = output_dir,
        ),
    ]

hermetic_clang_distribution = rule(
    impl = _hermetic_clang_distribution_impl,
    attrs = {
        "version": attrs.string(),
        "target": attrs.string(),
        "archive": attrs.dep(providers = [DefaultInfo]),
    },
)

def _host_target() -> str:
    """Determine the host target triple."""
    arch = host_info().arch
    os = host_info().os

    if arch.is_x86_64:
        arch_str = "x86_64"
    elif arch.is_aarch64:
        arch_str = "aarch64"
    else:
        fail("Unsupported host architecture")

    if os.is_linux:
        return arch_str + "-unknown-linux-gnu"
    elif os.is_macos:
        return arch_str + "-apple-darwin"
    else:
        fail("Unsupported host OS")

def download_clang_distribution(
    name: str,
    version: str,
    target: [None, str] = None,
    visibility: [None, list] = None):
    """Download a simple Clang/LLVM distribution.

    Args:
        name: Name of the target
        version: Clang version (e.g., "20.1.0")
        target: Target triple (defaults to host)
        visibility: Target visibility
    """
    if target == None:
        target = _host_target()

    release = _get_clang_release(version, target)
    archive_name = name + "-archive"

    _simple_http_archive(
        name = archive_name,
        urls = [release["url"]],
        sha1 = release.get("sha1", ""),
        sha256 = release.get("sha256", ""),
    )

    hermetic_clang_distribution(
        name = name,
        version = version,
        target = target,
        archive = ":" + archive_name,
        visibility = visibility,
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
    """Create a hermetic Clang toolchain from a simple distribution."""

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
            "# For other tools, add -fno-color-diagnostics for compilers",
            "if [[ \"$TOOL\" == \"clang\" || \"$TOOL\" == \"clang++\" ]]; then",
            "  exec \"$CLANG_DIR/bin/$TOOL\" -fno-color-diagnostics \"$@\"",
            "else",
            "  exec \"$CLANG_DIR/bin/$TOOL\" \"$@\"",
            "fi",
        ],
        is_executable = True,
    )

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
