# Copyright (c) Meta Platforms, Inc. and affiliates.
#
# This source code is licensed under both the MIT license found in the
# LICENSE-MIT file in the root directory of this source tree and the Apache
# License, Version 2.0 found in the LICENSE-APACHE file in the root directory
# of this source tree.

load("@prelude//:alias.bzl", "alias_impl", "configured_alias_impl", "versioned_alias_impl")
load("@prelude//:command_alias.bzl", "command_alias_impl")
load("@prelude//:export_file.bzl", "export_file_impl")
load("@prelude//:filegroup.bzl", "filegroup_impl")
load("@prelude//:genrule.bzl", "genrule_attributes", "genrule_impl")
load("@prelude//:http_file.bzl", "http_file_impl")
load("@prelude//:remote_file.bzl", "remote_file_impl")
load("@prelude//:sh_binary.bzl", "sh_binary_impl")
load("@prelude//:sh_test.bzl", "sh_test_impl")
load("@prelude//:test_suite.bzl", "test_suite_impl")
load("@prelude//android:android.bzl", _android_extra_attributes = "extra_attributes", _android_implemented_rules = "implemented_rules")
load("@prelude//android:configuration.bzl", "is_building_android_binary_attr")
load("@prelude//apple:apple_rules_impl.bzl", _apple_extra_attributes = "extra_attributes", _apple_implemented_rules = "implemented_rules")
load("@prelude//apple/user:apple_resource_transition.bzl", "apple_resource_transition")
load("@prelude//apple/user:target_sdk_version_transition.bzl", "apple_test_target_sdk_version_transition", "target_sdk_version_transition")
load("@prelude//configurations:rules.bzl", _config_extra_attributes = "extra_attributes", _config_implemented_rules = "implemented_rules")
load("@prelude//csharp:csharp.bzl", "csharp_library_impl", "prebuilt_dotnet_library_impl")
load("@prelude//cxx:bitcode.bzl", "llvm_link_bitcode_impl")
load("@prelude//cxx:cuda.bzl", "CudaCompileStyle")
load("@prelude//cxx:cxx.bzl", "cxx_binary_impl", "cxx_library_impl", "cxx_precompiled_header_impl", "cxx_test_impl", "prebuilt_cxx_library_impl")
load("@prelude//cxx:cxx_toolchain.bzl", "cxx_toolchain_extra_attributes", "cxx_toolchain_impl")
load("@prelude//cxx:cxx_toolchain_types.bzl", "CxxPlatformInfo", "CxxToolchainInfo")
load("@prelude//cxx:headers.bzl", "CPrecompiledHeaderInfo", "HeaderMode")
load("@prelude//cxx:link_groups_types.bzl", "LINK_GROUP_MAP_ATTR")
load("@prelude//cxx:prebuilt_cxx_library_group.bzl", "prebuilt_cxx_library_group_impl")
load("@prelude//cxx:windows_resource.bzl", "windows_resource_impl")
load("@prelude//decls:android_rules.bzl", "android_rules")
load("@prelude//decls:apple_rules.bzl", "ios_rules")
load("@prelude//decls:common.bzl", "IncludeType", "buck")
load("@prelude//decls:core_rules.bzl", "core_rules")
load("@prelude//decls:cxx_rules.bzl", "BUILD_INFO_ATTR", "cxx_rules")
load("@prelude//decls:d_rules.bzl", "d_rules")
load("@prelude//decls:dotnet_rules.bzl", "dotnet_rules")
load("@prelude//decls:erlang_rules.bzl", "erlang_rules")
load("@prelude//decls:git_rules.bzl", "git_rules")
load("@prelude//decls:go_rules.bzl", "go_rules")
load("@prelude//decls:groovy_rules.bzl", "groovy_rules")
load("@prelude//decls:halide_rules.bzl", "halide_rules")
load("@prelude//decls:haskell_rules.bzl", "haskell_rules")
load("@prelude//decls:java_rules.bzl", "java_rules")
load("@prelude//decls:js_rules.bzl", "js_rules")
load("@prelude//decls:kotlin_rules.bzl", "kotlin_rules")
load("@prelude//decls:lua_rules.bzl", "lua_rules")
load("@prelude//decls:ocaml_rules.bzl", "ocaml_rules")
load("@prelude//decls:python_rules.bzl", "python_rules")
load("@prelude//decls:re_test_common.bzl", "re_test_common")
load("@prelude//decls:rust_rules.bzl", "rust_rules")
load("@prelude//decls:scala_rules.bzl", "scala_rules")
load("@prelude//decls:shell_rules.bzl", "shell_rules")
load("@prelude//decls:toolchains_common.bzl", "toolchains_common")
load("@prelude//decls:uncategorized_rules.bzl", "uncategorized_rules")
load("@prelude//erlang:erlang.bzl", _erlang_implemented_rules = "implemented_rules")
load("@prelude//git:git_fetch.bzl", "git_fetch_impl")
load("@prelude//go:coverage.bzl", "GoCoverageMode")
load("@prelude//go:go_binary.bzl", "go_binary_impl")
load("@prelude//go:go_exported_library.bzl", "go_exported_library_impl")
load("@prelude//go:go_library.bzl", "go_library_impl")
load("@prelude//go:go_stdlib.bzl", "go_stdlib_impl")
load("@prelude//go:go_test.bzl", "go_test_impl")
load("@prelude//go/transitions:defs.bzl", "asan_attr", "build_tags_attr", "cgo_enabled_attr", "coverage_mode_attr", "go_binary_transition", "go_exported_library_transition", "go_library_transition", "go_stdlib_transition", "go_test_transition", "race_attr")
load("@prelude//go_bootstrap:go_bootstrap.bzl", "go_bootstrap_binary_impl")
load("@prelude//haskell:haskell.bzl", "haskell_binary_impl", "haskell_library_impl", "haskell_prebuilt_library_impl")
load("@prelude//haskell:haskell_ghci.bzl", "haskell_ghci_impl")
load("@prelude//haskell:haskell_haddock.bzl", "haskell_haddock_impl")
load("@prelude//haskell:haskell_ide.bzl", "haskell_ide_impl")
load("@prelude//haskell:library_info.bzl", "HaskellLibraryProvider")
load("@prelude//http_archive:http_archive.bzl", "http_archive_impl")
load("@prelude//java:java.bzl", _java_extra_attributes = "extra_attributes", _java_implemented_rules = "implemented_rules")
load("@prelude//js:js.bzl", _js_extra_attributes = "extra_attributes", _js_implemented_rules = "implemented_rules")
load("@prelude//js:worker_tool.bzl", "worker_tool")
load("@prelude//julia:julia.bzl", _julia_extra_attributes = "extra_attributes", _julia_implemented_rules = "implemented_rules")
load("@prelude//kotlin:kotlin.bzl", _kotlin_extra_attributes = "extra_attributes", _kotlin_implemented_rules = "implemented_rules")
load("@prelude//linking:execution_preference.bzl", "link_execution_preference_attr")
load("@prelude//linking:link_info.bzl", "LinkOrdering")
load("@prelude//linking:types.bzl", "Linkage")
load("@prelude//lua:cxx_lua_extension.bzl", "cxx_lua_extension_impl")
load("@prelude//lua:lua_binary.bzl", "lua_binary_impl")
load("@prelude//lua:lua_library.bzl", "lua_library_impl")
load("@prelude//matlab:matlab.bzl", _matlab_extra_attributes = "extra_attributes", _matlab_implemented_rules = "implemented_rules")
load("@prelude//ocaml:attrs.bzl", _ocaml_extra_attributes = "ocaml_extra_attributes")
load("@prelude//ocaml:ocaml.bzl", "ocaml_binary_impl", "ocaml_library_impl", "ocaml_object_impl", "ocaml_shared_impl", "prebuilt_ocaml_library_impl")
load("@prelude//python:cxx_python_extension.bzl", "cxx_python_extension_impl")
load("@prelude//python:prebuilt_python_library.bzl", "prebuilt_python_library_impl")
load("@prelude//python:python_binary.bzl", "python_binary_impl")
load("@prelude//python:python_library.bzl", "python_library_impl")
load("@prelude//python:python_needed_coverage_test.bzl", "python_needed_coverage_test_impl")
load("@prelude//python:python_runtime_bundle.bzl", "python_runtime_bundle_impl")
load("@prelude//python:python_test.bzl", "python_test_impl")
load("@prelude//python_bootstrap:python_bootstrap.bzl", "PythonBootstrapSources", "python_bootstrap_binary_impl", "python_bootstrap_library_impl")
load("@prelude//transitions:constraint_overrides.bzl", "constraint_overrides")
load("@prelude//zip_file:zip_file.bzl", _zip_file_extra_attributes = "extra_attributes", _zip_file_implemented_rules = "implemented_rules")

rule_decl_records = [
    android_rules,
    core_rules,
    cxx_rules,
    d_rules,
    dotnet_rules,
    erlang_rules,
    git_rules,
    go_rules,
    groovy_rules,
    halide_rules,
    haskell_rules,
    ios_rules,
    java_rules,
    kotlin_rules,
    lua_rules,
    ocaml_rules,
    python_rules,
    rust_rules,
    shell_rules,
    js_rules,
    scala_rules,
    uncategorized_rules,
]

def _merge_dictionaries(dicts):
    result = {}
    for d in dicts:
        for key, value in d.items():
            if key in result:
                fail("Duplicate key: '{}' while merging dictionaries".format(key))
            result[key] = value

    return result

extra_implemented_rules = struct(
    #common rules
    alias = alias_impl,
    command_alias = command_alias_impl,
    configured_alias = configured_alias_impl,
    export_file = export_file_impl,
    filegroup = filegroup_impl,
    genrule = genrule_impl,
    http_archive = http_archive_impl,
    http_file = http_file_impl,
    remote_file = remote_file_impl,
    sh_binary = sh_binary_impl,
    sh_test = sh_test_impl,
    test_suite = test_suite_impl,
    toolchain_alias = alias_impl,
    versioned_alias = versioned_alias_impl,
    worker_tool = worker_tool,

    #c#
    csharp_library = csharp_library_impl,
    prebuilt_dotnet_library = prebuilt_dotnet_library_impl,

    #c++
    cxx_binary = cxx_binary_impl,
    cxx_test = cxx_test_impl,
    cxx_toolchain = cxx_toolchain_impl,
    cxx_genrule = genrule_impl,
    cxx_library = cxx_library_impl,
    cxx_precompiled_header = cxx_precompiled_header_impl,
    cxx_python_extension = cxx_python_extension_impl,
    prebuilt_cxx_library = prebuilt_cxx_library_impl,
    prebuilt_cxx_library_group = prebuilt_cxx_library_group_impl,
    windows_resource = windows_resource_impl,

    # C++ / LLVM
    llvm_link_bitcode = llvm_link_bitcode_impl,

    #git
    git_fetch = git_fetch_impl,

    #go
    go_binary = go_binary_impl,
    go_bootstrap_binary = go_bootstrap_binary_impl,
    go_exported_library = go_exported_library_impl,
    go_library = go_library_impl,
    go_test = go_test_impl,
    go_stdlib = go_stdlib_impl,

    #haskell
    haskell_library = haskell_library_impl,
    haskell_binary = haskell_binary_impl,
    haskell_ghci = haskell_ghci_impl,
    haskell_haddock = haskell_haddock_impl,
    haskell_ide = haskell_ide_impl,
    haskell_prebuilt_library = haskell_prebuilt_library_impl,

    #lua
    cxx_lua_extension = cxx_lua_extension_impl,
    lua_binary = lua_binary_impl,
    lua_library = lua_library_impl,

    #ocaml
    ocaml_binary = ocaml_binary_impl,
    ocaml_object = ocaml_object_impl,
    ocaml_shared = ocaml_shared_impl,
    ocaml_library = ocaml_library_impl,
    prebuilt_ocaml_library = prebuilt_ocaml_library_impl,

    #python
    prebuilt_python_library = prebuilt_python_library_impl,
    python_binary = python_binary_impl,
    python_library = python_library_impl,
    python_runtime_bundle = python_runtime_bundle_impl,
    python_test = python_test_impl,
    python_needed_coverage_test = python_needed_coverage_test_impl,

    #python bootstrap
    python_bootstrap_binary = python_bootstrap_binary_impl,
    python_bootstrap_library = python_bootstrap_library_impl,

    #merged **kwargs
    **_merge_dictionaries([
        _android_implemented_rules,
        _apple_implemented_rules,
        _config_implemented_rules,
        _erlang_implemented_rules,
        _java_implemented_rules,
        _js_implemented_rules,
        _julia_implemented_rules,
        _kotlin_implemented_rules,
        _matlab_implemented_rules,
        _zip_file_implemented_rules,
    ])
)

def _python_runtime_bundle_attrs():
    return {
        "include": attrs.string(doc = "Header files required for linking python extensions"),
        "install_root": attrs.dep(doc = "The filegroup containing the runtime artifacts, all the paths are relative to this location"),
        "libpython": attrs.string(doc = "libpyhon.so required at runtime for the python executable and native extensions."),
        "py_bin": attrs.string(doc = "The runtime executable"),
        "py_version": attrs.string(doc = "The version of python this represents"),
        "stdlib": attrs.string(doc = "The python standard library"),
    }

inlined_extra_attributes = {

    # csharp
    "csharp_library": {
        "_csharp_toolchain": toolchains_common.csharp(),
    },

    #c++
    "cxx_genrule": genrule_attributes() | {
        "_cxx_toolchain": toolchains_common.cxx(),
        "_exec_os_type": buck.exec_os_type_arg(),
    },
    "cxx_library": {
        "auto_link_groups": attrs.bool(default = False),
        # These flags will only be used to instrument a target
        # when coverage for that target is enabled by `exported_needs_coverage_instrumentation`
        # or by any of the target's dependencies.
        "coverage_instrumentation_compiler_flags": attrs.list(attrs.string(), default = []),
        "cuda_compile_style": attrs.enum(CudaCompileStyle.values(), default = "mono"),
        "deps_query": attrs.option(attrs.query(), default = None),
        "exported_needs_coverage_instrumentation": attrs.bool(default = False),
        "extra_xcode_sources": attrs.list(attrs.source(allow_directory = True), default = []),
        "header_mode": attrs.option(attrs.enum(HeaderMode.values()), default = None),
        "link_deps_query_whole": attrs.bool(default = False),
        "link_execution_preference": link_execution_preference_attr(),
        "link_group_map": LINK_GROUP_MAP_ATTR,
        "link_ordering": attrs.option(attrs.enum(LinkOrdering.values()), default = None),
        "precompiled_header": attrs.option(attrs.dep(providers = [CPrecompiledHeaderInfo]), default = None),
        "prefer_stripped_objects": attrs.bool(default = False),
        "preferred_linkage": attrs.enum(
            Linkage.values(),
            default = "any",
            doc = """
            Determines what linkage is used when the library is depended on by another target. To
            control how the dependencies of this library are linked, use `link_style` instead.
            """,
        ),
        "resources": attrs.named_set(attrs.one_of(attrs.dep(), attrs.source(allow_directory = True)), sorted = True, default = []),
        "separate_debug_info": attrs.bool(default = False),
        "stub": attrs.bool(default = False),
        "supports_header_symlink_subtarget": attrs.bool(default = False),
        "supports_python_dlopen": attrs.option(attrs.bool(), default = None),
        "supports_shlib_interfaces": attrs.bool(default = True),
        "_create_third_party_build_root": attrs.default_only(attrs.exec_dep(default = "prelude//third-party/tools:create_build")),
        "_cxx_hacks": attrs.default_only(attrs.dep(default = "prelude//cxx/tools:cxx_hacks")),
        "_cxx_toolchain": toolchains_common.cxx(),
        "_is_building_android_binary": is_building_android_binary_attr(),
    },
    "cxx_test": re_test_common.test_args(),
    "cxx_toolchain": cxx_toolchain_extra_attributes(is_toolchain_rule = False),
    "export_file": constraint_overrides.attributes,
    "filegroup": constraint_overrides.attributes,

    # Generic rule to build from a command
    "genrule": genrule_attributes() | constraint_overrides.attributes,

    # Go
    "go_binary": {
        "embedcfg": attrs.option(attrs.source(allow_directory = False), default = None),
        "resources": attrs.list(attrs.one_of(attrs.dep(), attrs.source(allow_directory = True)), default = []),
        "_asan": asan_attr,
        "_build_info": BUILD_INFO_ATTR,
        "_build_tags": build_tags_attr,
        "_cxx_toolchain": toolchains_common.cxx(),
        "_exec_os_type": buck.exec_os_type_arg(),
        "_go_stdlib": attrs.default_only(attrs.dep(default = "prelude//go/tools:stdlib")),
        "_go_toolchain": toolchains_common.go(),
        "_race": race_attr,
    },
    "go_bootstrap_binary": {
        "_exec_os_type": buck.exec_os_type_arg(),
        "_go_bootstrap_toolchain": toolchains_common.go_bootstrap(),
    },
    "go_exported_library": {
        "embedcfg": attrs.option(attrs.source(allow_directory = False), default = None),
        "_asan": asan_attr,
        "_build_info": BUILD_INFO_ATTR,
        "_build_tags": build_tags_attr,
        "_cxx_toolchain": toolchains_common.cxx(),
        "_exec_os_type": buck.exec_os_type_arg(),
        "_go_stdlib": attrs.default_only(attrs.dep(default = "prelude//go/tools:stdlib")),
        "_go_toolchain": toolchains_common.go(),
        "_race": race_attr,
    },
    "go_library": {
        "embedcfg": attrs.option(attrs.source(allow_directory = False), default = None),
        "_asan": asan_attr,
        "_build_tags": build_tags_attr,
        "_cgo_enabled": cgo_enabled_attr,
        "_coverage_mode": coverage_mode_attr,
        "_cxx_toolchain": toolchains_common.cxx(),
        "_exec_os_type": buck.exec_os_type_arg(),
        "_go_stdlib": attrs.default_only(attrs.dep(default = "prelude//go/tools:stdlib")),
        "_go_toolchain": toolchains_common.go(),
        "_race": race_attr,
    },
    "go_stdlib": {
        "_asan": asan_attr,
        "_build_tags": build_tags_attr,
        "_cgo_enabled": cgo_enabled_attr,
        "_cxx_toolchain": toolchains_common.cxx(),
        "_exec_os_type": buck.exec_os_type_arg(),
        "_go_toolchain": toolchains_common.go(),
        "_race": race_attr,
    },
    "go_test": {
        "coverage_mode": attrs.option(attrs.enum(GoCoverageMode.values()), default = None),
        "embedcfg": attrs.option(attrs.source(allow_directory = False), default = None),
        "resources": attrs.list(attrs.source(allow_directory = True), default = []),
        "_asan": asan_attr,
        "_build_info": BUILD_INFO_ATTR,
        "_build_tags": build_tags_attr,
        "_coverage_mode": coverage_mode_attr,
        "_cxx_toolchain": toolchains_common.cxx(),
        "_exec_os_type": buck.exec_os_type_arg(),
        "_go_stdlib": attrs.default_only(attrs.dep(default = "prelude//go/tools:stdlib")),
        "_go_toolchain": toolchains_common.go(),
        "_race": race_attr,
        "_testmaingen": attrs.default_only(attrs.exec_dep(providers = [RunInfo], default = "prelude//go_bootstrap/tools:go_testmaingen")),
    },

    # groovy
    "groovy_library": {
        "resources_root": attrs.option(attrs.string(), default = None),
    },
    "groovy_test": {
        "resources_root": attrs.option(attrs.string(), default = None),
    },
    "haskell_binary": {
        "auto_link_groups": attrs.bool(default = False),
        "link_group_map": LINK_GROUP_MAP_ATTR,
        "template_deps": attrs.list(attrs.exec_dep(providers = [HaskellLibraryProvider]), default = []),
        "_cxx_toolchain": toolchains_common.cxx(),
        "_haskell_toolchain": toolchains_common.haskell(),
    },
    "haskell_ghci": {
        "template_deps": attrs.list(attrs.exec_dep(providers = [HaskellLibraryProvider]), default = []),
        "_cxx_toolchain": toolchains_common.cxx(),
        "_haskell_toolchain": toolchains_common.haskell(),
    },
    "haskell_haddock": {
        "_cxx_toolchain": toolchains_common.cxx(),
        "_haskell_toolchain": toolchains_common.haskell(),
    },
    "haskell_ide": {
        "include_projects": attrs.list(attrs.dep(), default = []),
        "_haskell_toolchain": toolchains_common.haskell(),
    },
    "haskell_library": {
        "preferred_linkage": attrs.enum(Linkage.values(), default = "any"),
        "template_deps": attrs.list(attrs.exec_dep(providers = [HaskellLibraryProvider]), default = []),
        "_cxx_toolchain": toolchains_common.cxx(),
        "_haskell_toolchain": toolchains_common.haskell(),
    },
    "llvm_link_bitcode": {
        "_cxx_toolchain": toolchains_common.cxx(),
    },
    "ndk_toolchain": {
        "cxx_toolchain": attrs.toolchain_dep(providers = [CxxToolchainInfo, CxxPlatformInfo]),
    },
    "prebuilt_cxx_library": {
        "exported_header_style": attrs.enum(IncludeType, default = "system"),
        "header_dirs": attrs.option(attrs.list(attrs.source(allow_directory = True)), default = None),
        "linker_flags": attrs.list(attrs.arg(anon_target_compatible = True), default = []),
        "platform_header_dirs": attrs.option(attrs.list(attrs.tuple(attrs.regex(), attrs.list(attrs.source(allow_directory = True)))), default = None),
        "post_linker_flags": attrs.list(attrs.arg(anon_target_compatible = True), default = []),
        "preferred_linkage": attrs.enum(
            Linkage.values(),
            default = "any",
            doc = """
            Determines what linkage is used when the library is depended on by another target. To
            control how the dependencies of this library are linked, use `link_style` instead.
            """,
        ),
        "public_include_directories": attrs.set(attrs.string(), sorted = True, default = []),
        "public_system_include_directories": attrs.set(attrs.string(), sorted = True, default = []),
        "raw_headers": attrs.set(attrs.source(), sorted = True, default = []),
        "stub": attrs.bool(default = False),
        "supports_lto": attrs.bool(default = False),
        "supports_python_dlopen": attrs.bool(default = True),
        "versioned_header_dirs": attrs.option(attrs.versioned(attrs.list(attrs.source(allow_directory = True))), default = None),
        "_create_third_party_build_root": attrs.default_only(attrs.exec_dep(default = "prelude//third-party/tools:create_build")),
        "_cxx_toolchain": toolchains_common.cxx(),
        "_target_os_type": buck.target_os_type_arg(),
    },
    "prebuilt_cxx_library_group": {
        "_cxx_toolchain": toolchains_common.cxx(),
    },

    #python
    "prebuilt_python_library": {
        "_create_third_party_build_root": attrs.default_only(attrs.exec_dep(default = "prelude//third-party/tools:create_build")),
        "_extract": attrs.default_only(attrs.exec_dep(default = "prelude//python/tools:extract")),
        "_python_toolchain": toolchains_common.python(),
    },
    #python bootstrap
    "python_bootstrap_binary": {
        "copy_deps": attrs.bool(default = True),
        "deps": attrs.list(attrs.dep(providers = [PythonBootstrapSources]), default = []),
        "main": attrs.source(),
        "_python_bootstrap_toolchain": toolchains_common.python_bootstrap(),
    },
    "python_bootstrap_library": {
        "deps": attrs.list(attrs.dep(providers = [PythonBootstrapSources]), default = []),
        "srcs": attrs.list(attrs.source()),
    },
    "python_library": {
        "resources": attrs.named_set(attrs.one_of(attrs.dep(), attrs.source(allow_directory = True)), sorted = True, default = []),
        "_create_third_party_build_root": attrs.default_only(attrs.exec_dep(default = "prelude//third-party/tools:create_build")),
        "_cxx_toolchain": toolchains_common.cxx(),
        "_python_toolchain": toolchains_common.python(),
    },
    "python_needed_coverage_test": dict(
        contacts = attrs.list(attrs.string(), default = []),
        env = attrs.dict(key = attrs.string(), value = attrs.arg(), sorted = False, default = {}),
        labels = attrs.list(attrs.string(), default = []),
        needed_coverage = attrs.list(attrs.tuple(attrs.int(), attrs.dep(), attrs.option(attrs.string())), default = []),
        test = attrs.dep(providers = [ExternalRunnerTestInfo]),
        **(re_test_common.test_args() | buck.inject_test_env_arg())
    ),
    "python_runtime_bundle": _python_runtime_bundle_attrs(),
    "remote_file": {
        "sha1": attrs.option(attrs.string(), default = None),
        "sha256": attrs.option(attrs.string(), default = None),
        "_unzip_tool": attrs.default_only(attrs.exec_dep(providers = [RunInfo], default = "prelude//zip_file/tools:unzip")),
    },
    "rust_test": {},
    "sh_test": constraint_overrides.attributes,
    "windows_resource": {
        "_cxx_toolchain": toolchains_common.cxx(),
    },
}

all_extra_attributes = _merge_dictionaries([
    inlined_extra_attributes,
    _android_extra_attributes,
    _apple_extra_attributes,
    _config_extra_attributes,
    _java_extra_attributes,
    _js_extra_attributes,
    _julia_extra_attributes,
    _kotlin_extra_attributes,
    _matlab_extra_attributes,
    _ocaml_extra_attributes,
    _zip_file_extra_attributes,
])

extra_attributes = struct(**all_extra_attributes)

# Configuration transitions to pass `cfg` for builtin rules.
transitions = {
    "android_binary": constraint_overrides.transition,
    "apple_asset_catalog": apple_resource_transition,
    "apple_binary": target_sdk_version_transition,
    "apple_bundle": target_sdk_version_transition,
    "apple_library": target_sdk_version_transition,
    "apple_resource": apple_resource_transition,
    "apple_test": apple_test_target_sdk_version_transition,
    "cxx_binary": constraint_overrides.transition,
    "cxx_test": constraint_overrides.transition,
    "export_file": constraint_overrides.transition,
    "filegroup": constraint_overrides.transition,
    "genrule": constraint_overrides.transition,
    "go_binary": go_binary_transition,
    "go_exported_library": go_exported_library_transition,
    "go_library": go_library_transition,
    "go_stdlib": go_stdlib_transition,
    "go_test": go_test_transition,
    "python_binary": constraint_overrides.python_transition,
    "python_test": constraint_overrides.python_transition,
    "sh_test": constraint_overrides.transition,
}

toolchain_rule_names = [
    "apple_toolchain",
    "swift_macro_toolchain",
    "swift_toolchain",
    "toolchain_alias",
]
