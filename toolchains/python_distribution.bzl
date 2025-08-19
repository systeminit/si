"""Hermetic Python toolchain distribution rules.

Downloads and manages Python toolchains using pre-built binaries.
Similar to the Rust/Deno/Clang toolchain approach using python-build-standalone.
"""

load(
    "@prelude//:artifacts.bzl",
    "ArtifactGroupInfo",
)
load("@prelude//python:toolchain.bzl", "PythonPlatformInfo", "PythonToolchainInfo")
load("@prelude//python_bootstrap:python_bootstrap.bzl", "PythonBootstrapToolchainInfo")

# Python release information with checksums from python-build-standalone
# Using Python 3.13.6 from https://github.com/astral-sh/python-build-standalone
_PYTHON_RELEASES = {
    "3.13.6": {
        "x86_64-unknown-linux-gnu": {
            "url": "https://github.com/astral-sh/python-build-standalone/releases/download/20250807/cpython-3.13.6+20250807-x86_64-unknown-linux-gnu-install_only.tar.gz",
            "sha256": "e2e4acef960420df7ecfb75739ce4d5756510a04370103b3659de0f7dcd36c16",
        },
        "aarch64-unknown-linux-gnu": {
            "url": "https://github.com/astral-sh/python-build-standalone/releases/download/20250807/cpython-3.13.6+20250807-aarch64-unknown-linux-gnu-install_only.tar.gz",
            "sha256": "4cc164b8b541bd9d86457bef6a5846599345148b297e248a05ccc040ef3021c7",
        },
        "x86_64-apple-darwin": {
            "url": "https://github.com/astral-sh/python-build-standalone/releases/download/20250807/cpython-3.13.6+20250807-x86_64-apple-darwin-install_only.tar.gz",
            "sha256": "24eb900f548a9cb07a300fbe3b393fd2b515f20548c08a8ff4d31ab55bc71933",
        },
        "aarch64-apple-darwin": {
            "url": "https://github.com/astral-sh/python-build-standalone/releases/download/20250807/cpython-3.13.6+20250807-aarch64-apple-darwin-install_only.tar.gz",
            "sha256": "b4e6b5ca0f5e59b4381886339f91dc0ecaf95b423e1a9e40bbea5b55c0cdd8ff",
        },
    },
}

PythonDistributionInfo = provider(
    fields = {
        "version": provider_field(typing.Any, default = None),
        "target": provider_field(typing.Any, default = None),
        "python": provider_field(typing.Any, default = None),
        "pip": provider_field(typing.Any, default = None),
        "python_path": provider_field(typing.Any, default = None),
    },
)

def _get_python_release(version: str, target: str):
    if version not in _PYTHON_RELEASES:
        fail("Unknown Python version '{}'. Available versions: {}".format(
            version,
            ", ".join(_PYTHON_RELEASES.keys()),
        ))

    python_version = _PYTHON_RELEASES[version]
    if target not in python_version:
        fail("Unsupported target '{}' for Python {}. Supported targets: {}".format(
            target,
            version,
            ", ".join(python_version.keys()),
        ))

    return python_version[target]

def _python_distribution_impl(ctx: AnalysisContext) -> list[Provider]:
    """Extract Python from pre-built standalone archive."""

    # Path to pre-built archive
    archive_path = cmd_args(ctx.attrs.archive[DefaultInfo].default_outputs[0])
    python_version = ctx.attrs.version

    # Create output directory that contains the complete Python installation
    python_dist = ctx.actions.declare_output("python", dir=True)

    # Create setup script to extract and organize the pre-built Python
    setup_script = ctx.actions.declare_output("setup.sh")
    ctx.actions.write(
        setup_script,
        [
            "#!/bin/bash",
            "set -e",
            "ARCHIVE_DIR=$1",
            "OUTPUT_DIR=$3",
            "",
            "echo 'Extracting pre-built Python for hermetic operation'",
            "",
            "# Create output directory and copy the extracted Python distribution",
            "mkdir -p $OUTPUT_DIR",
            "cp -r $ARCHIVE_DIR/python/* $OUTPUT_DIR/",
            "",
            "# Create wrapper script for python binary",
            "cat > $OUTPUT_DIR/bin/python-wrapper << 'EOF'",
            "#!/bin/bash",
            "set -e",
            'SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"',
            'export LD_LIBRARY_PATH="$SCRIPT_DIR/../lib:$LD_LIBRARY_PATH"',
            'export PYTHONPATH="$SCRIPT_DIR/../lib/python{}/site-packages:$PYTHONPATH"'.format(python_version[:4]),  # e.g., 3.13
            'exec "$SCRIPT_DIR/python3" "$@"',
            "EOF",
            "chmod +x $OUTPUT_DIR/bin/python-wrapper",
            "",
            "# Create wrapper script for pip",
            "cat > $OUTPUT_DIR/bin/pip-wrapper << 'EOF'",
            "#!/bin/bash",
            "set -e",
            'SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"',
            'export LD_LIBRARY_PATH="$SCRIPT_DIR/../lib:$LD_LIBRARY_PATH"',
            'export PYTHONPATH="$SCRIPT_DIR/../lib/python{}/site-packages:$PYTHONPATH"'.format(python_version[:4]),
            'exec "$SCRIPT_DIR/pip3" "$@"',
            "EOF",
            "chmod +x $OUTPUT_DIR/bin/pip-wrapper",
            "",
            "echo 'Python hermetic distribution ready'",
        ],
        is_executable = True,
    )

    ctx.actions.run([
        setup_script, archive_path, "unused", python_dist.as_output()
    ], category = "python_setup")

    # Individual binary references point to files in the dist directory
    python_exe = cmd_args(python_dist, "/bin/python-wrapper", delimiter="")
    pip_exe = cmd_args(python_dist, "/bin/pip-wrapper", delimiter="")
    python_lib_path = cmd_args(python_dist, "/lib", delimiter="")

    return [
        DefaultInfo(default_output = python_dist),
        PythonDistributionInfo(
            version = ctx.attrs.version,
            target = ctx.attrs.target,
            python = python_exe,
            pip = pip_exe,
            python_path = python_lib_path,
        ),
    ]

python_distribution = rule(
    impl = _python_distribution_impl,
    attrs = {
        "version": attrs.string(),
        "target": attrs.string(),
        "archive": attrs.dep(providers = [DefaultInfo]),
    },
)

def _http_archive_impl(ctx: AnalysisContext) -> list[Provider]:
    """Download and extract a tar.gz archive."""
    url = ctx.attrs.urls[0]

    # Download archive
    archive = ctx.actions.declare_output("archive.tar.gz")
    if ctx.attrs.sha256:
        ctx.actions.download_file(archive.as_output(), url, sha256 = ctx.attrs.sha256)
    else:
        ctx.actions.download_file(archive.as_output(), url)

    # Extract archive
    output = ctx.actions.declare_output(ctx.label.name, dir = True)
    script = [
        "mkdir -p $1",
        "tar xzf $2 -C $1 --strip-components=0"
    ]
    ctx.actions.run([
        "sh", "-c", "; ".join(script), "--", output.as_output(), archive
    ], category = "extract_python")

    return [DefaultInfo(default_output = output)]

_http_archive = rule(
    impl = _http_archive_impl,
    attrs = {
        "urls": attrs.list(attrs.string()),
        "sha256": attrs.string(default = ""),
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

def download_python_distribution(
    name: str,
    version: str,
    target: [None, str] = None):
    """Download a pre-built Python distribution from python-build-standalone.

    Args:
        name: Name of the target
        version: Python version (e.g., "3.13.6")
        target: Target triple (defaults to host)
    """
    if target == None:
        target = _host_target()

    release = _get_python_release(version, target)
    archive_name = name + "-archive"

    _http_archive(
        name = archive_name,
        urls = [release["url"]],
        sha256 = release["sha256"],
    )

    python_distribution(
        name = name,
        version = version,
        target = target,
        archive = ":" + archive_name,
    )

def _hermetic_python_toolchain_impl(ctx: AnalysisContext) -> list[Provider]:
    """Create a hermetic Python toolchain from a distribution."""

    dist = ctx.attrs.distribution[PythonDistributionInfo]

    # Create command-line wrappers for the tools
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

    # Create command-line wrapper for the interpreter
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
