"""Hermetic Python toolchain distribution rules using S3 artifacts.

Downloads pre-packaged Python toolchains from si-artifacts-prod.
Uses unified structure for easier extraction and setup.
"""

load("@prelude//:artifacts.bzl", "ArtifactGroupInfo")
load("@prelude//python:toolchain.bzl", "PythonPlatformInfo", "PythonToolchainInfo")
load("@prelude//python_bootstrap:python_bootstrap.bzl", "PythonBootstrapToolchainInfo")
load("@prelude-si//toolchains:common.bzl", "create_download_distribution_function", "create_distribution_provider")
load("@prelude-si//toolchains:extraction.bzl", "ToolchainExtractionInfo")

# Python version checksums for our S3 artifacts
_PYTHON_S3_CHECKSUMS = {
    "3.13.6": {
        "linux": {
            "x86_64": "2831557346ce5be01856455ad1fe3a5c4438e84d12ef9e8860b8891c5fed6d33",
            "aarch64": "81e04c6c473952170081132dc3bed85b1c214943d245042790eeb10334980d13",
        },
        "darwin": {
            "x86_64": "cecdfe4e9628e708e5e887a4831695fb24aa6d21965070c7453acfd8bbb5bc55",
            "aarch64": "648603b4c7a7b098b7be355bf29fe4bf99ba3b5d07513b30cdb25a0ffba07406",
        },
    },
}

# Create provider using shared utility
PythonDistributionInfo = create_distribution_provider({
    "python": provider_field(typing.Any, default = None),
    "pip": provider_field(typing.Any, default = None),
    "python_path": provider_field(typing.Any, default = None),
})

def _python_distribution_impl(ctx: AnalysisContext) -> list[Provider]:
    """Create Python distribution from extracted S3 toolchain."""
    extraction = ctx.attrs.extraction[ToolchainExtractionInfo]
    
    return [
        DefaultInfo(),
        PythonDistributionInfo(
            version = ctx.attrs.version,
            target = ctx.attrs.target,
            python = cmd_args(extraction.bin_dir, "/python-wrapper", delimiter=""),
            pip = cmd_args(extraction.bin_dir, "/pip-wrapper", delimiter=""),
            python_path = cmd_args(extraction.lib_dir, delimiter=""),
        ),
    ]

python_distribution = rule(
    impl = _python_distribution_impl,
    attrs = {
        "version": attrs.string(),
        "target": attrs.string(), 
        "extraction": attrs.dep(providers = [ToolchainExtractionInfo]),
    },
)

# Create download function using shared utility
download_python_distribution = create_download_distribution_function(
    family = "python",
    checksums_dict = _PYTHON_S3_CHECKSUMS,
    distribution_rule = python_distribution,
    toolchain_name = "Python"
)

def _hermetic_python_toolchain_impl(ctx: AnalysisContext) -> list[Provider]:
    """Create a hermetic Python toolchain from a distribution."""
    
    dist = ctx.attrs.distribution[PythonDistributionInfo]
    python_cmd = cmd_args([dist.python])

    return [
        DefaultInfo(),
        PythonToolchainInfo(
            binary_linker_flags = ctx.attrs.binary_linker_flags,
            linker_flags = ctx.attrs.linker_flags,
            fail_with_message = ctx.attrs.fail_with_message[RunInfo],
            generate_static_extension_info = ctx.attrs.generate_static_extension_info,
            make_source_db = ctx.attrs.make_source_db[RunInfo],
            make_source_db_no_deps = ctx.attrs.make_source_db_no_deps[RunInfo],
            host_interpreter = RunInfo(args = python_cmd),
            interpreter = RunInfo(args = python_cmd),
            make_py_package_modules = ctx.attrs.make_py_package_modules[RunInfo],
            make_py_package_inplace = ctx.attrs.make_py_package_inplace[RunInfo],
            compile = RunInfo(args = ["echo", "COMPILEINFO"]),
            package_style = "inplace",
            pex_extension = ctx.attrs.pex_extension,
            native_link_strategy = "separate",
            runtime_library = ctx.attrs.runtime_library,
        ),
        PythonPlatformInfo(name = "x86_64"),
    ]

hermetic_python_toolchain = rule(
    impl = _hermetic_python_toolchain_impl,
    attrs = {
        "distribution": attrs.exec_dep(providers = [PythonDistributionInfo]),
        "binary_linker_flags": attrs.default_only(attrs.list(attrs.arg(), default = [])),
        "fail_with_message": attrs.default_only(attrs.dep(providers = [RunInfo], default = "prelude//python/tools:fail_with_message")),
        "generate_static_extension_info": attrs.default_only(attrs.dep(providers = [RunInfo], default = "prelude//python/tools:generate_static_extension_info")),
        "linker_flags": attrs.default_only(attrs.list(attrs.arg(), default = [])),
        "make_py_package_inplace": attrs.default_only(attrs.dep(providers = [RunInfo], default = "prelude//python/tools:make_py_package_inplace")),
        "make_py_package_modules": attrs.default_only(attrs.dep(providers = [RunInfo], default = "prelude//python/tools:make_py_package_modules")),
        "make_source_db": attrs.default_only(attrs.dep(providers = [RunInfo], default = "prelude//python/tools:make_source_db")),
        "make_source_db_no_deps": attrs.default_only(attrs.dep(providers = [RunInfo], default = "prelude//python/tools:make_source_db_no_deps")),
        "pex_extension": attrs.string(default = ".pex"),
        "runtime_library": attrs.default_only(attrs.dep(providers = [ArtifactGroupInfo], default = "prelude//python/runtime:bootstrap_files")),
    },
    is_toolchain_rule = True,
)

def _hermetic_python_bootstrap_toolchain_impl(ctx: AnalysisContext) -> list[Provider]:
    """Create a hermetic Python bootstrap toolchain from a distribution."""
    
    dist = ctx.attrs.distribution[PythonDistributionInfo]
    python_cmd = cmd_args([dist.python])

    return [
        DefaultInfo(),
        PythonBootstrapToolchainInfo(interpreter = python_cmd),
    ]

hermetic_python_bootstrap_toolchain = rule(
    impl = _hermetic_python_bootstrap_toolchain_impl,
    attrs = {
        "distribution": attrs.exec_dep(providers = [PythonDistributionInfo]),
    },
    is_toolchain_rule = True,
)