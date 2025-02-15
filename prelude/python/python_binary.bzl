# Copyright (c) Meta Platforms, Inc. and affiliates.
#
# This source code is licensed under both the MIT license found in the
# LICENSE-MIT file in the root directory of this source tree and the Apache
# License, Version 2.0 found in the LICENSE-APACHE file in the root directory
# of this source tree.

load(
    "@prelude//:artifacts.bzl",
    "ArtifactGroupInfo",
    "ArtifactOutputs",  # @unused Used as a type
)
load("@prelude//:paths.bzl", "paths")
load("@prelude//:resources.bzl", "gather_resources")
load("@prelude//cxx:cxx.bzl", "create_shared_lib_link_group_specs")
load("@prelude//cxx:cxx_context.bzl", "get_cxx_toolchain_info")
load("@prelude//cxx:cxx_executable.bzl", "cxx_executable")
load(
    "@prelude//cxx:cxx_library_utility.bzl",
    "cxx_is_gnu",
)
load("@prelude//cxx:cxx_sources.bzl", "CxxSrcWithFlags")
load("@prelude//cxx:cxx_toolchain_types.bzl", "CxxToolchainInfo")
load(
    "@prelude//cxx:cxx_types.bzl",
    "CxxRuleConstructorParams",
)
load("@prelude//cxx:cxx_utility.bzl", "cxx_attrs_get_allow_cache_upload")
load(
    "@prelude//cxx:groups_types.bzl",
    "Group",
    "GroupAttrs",
    "GroupMapping",
    "Traversal",
)
load("@prelude//cxx:headers.bzl", "cxx_get_regular_cxx_headers_layout")
load(
    "@prelude//cxx:link_groups.bzl",
    "LinkGroupLibSpec",
    "build_link_group_info",
    "get_link_group_info",
)
load(
    "@prelude//cxx:link_groups_types.bzl",
    "LinkGroupInfo",  # @unused Used as a type
)
load("@prelude//cxx:linker.bzl", "get_rpath_origin")
load(
    "@prelude//cxx:omnibus.bzl",
    "create_omnibus_libraries",
    "get_excluded",
    "get_omnibus_graph",
    "get_roots",
)
load(
    "@prelude//cxx:preprocessor.bzl",
    "CPreprocessor",
    "CPreprocessorArgs",
    "cxx_inherited_preprocessor_infos",
)
load(
    "@prelude//linking:link_info.bzl",
    "LinkedObject",
)
load(
    "@prelude//linking:linkable_graph.bzl",
    "LinkableGraph",
    "LinkableGraphTSet",
    "create_linkable_graph",
    _linkable_graph = "linkable_graph",
)
load(
    "@prelude//linking:linkables.bzl",
    "LinkableProviders",  # @unused Used as a type
    "linkables",
)
load(
    "@prelude//linking:shared_libraries.bzl",
    "SharedLibrary",
    "create_shlib",
    "merge_shared_libraries",
    "traverse_shared_library_info",
)
load("@prelude//linking:strip.bzl", "strip_debug_with_gnu_debuglink")
load("@prelude//linking:types.bzl", "Linkage")
load("@prelude//utils:utils.bzl", "flatten", "value_or")
load(":compile.bzl", "compile_manifests")
load(
    ":interface.bzl",
    "EntryPoint",
    "EntryPointKind",
    "PythonLibraryInterface",
)
load(":make_py_package.bzl", "PexModules", "PexProviders", "make_default_info", "make_py_package", "make_run_info")
load(
    ":manifest.bzl",
    "create_manifest_for_extensions",
    "create_manifest_for_source_map",
)
load(":native_python_util.bzl", "merge_cxx_extension_info", "reduce_cxx_extension_info")
load(":python.bzl", "info_to_interface")
load(
    ":python_library.bzl",
    "create_python_library_info",
    "gather_dep_libraries",
    "py_attr_resources",
    "py_resources",
    "qualify_srcs",
)
load(":source_db.bzl", "create_dbg_source_db", "create_python_source_db_info", "create_source_db_no_deps")
load(":toolchain.bzl", "NativeLinkStrategy", "PackageStyle", "PythonPlatformInfo", "PythonToolchainInfo", "get_package_style", "get_platform_attr")
load(":typing.bzl", "create_per_target_type_check")

OmnibusMetadataInfo = provider(
    # @unsorted-dict-items
    fields = {"omnibus_libs": provider_field(typing.Any, default = None), "omnibus_graph": provider_field(typing.Any, default = None)},
)

def _link_strategy(ctx: AnalysisContext) -> NativeLinkStrategy:
    if ctx.attrs.native_link_strategy != None:
        return NativeLinkStrategy(ctx.attrs.native_link_strategy)
    return NativeLinkStrategy(ctx.attrs._python_toolchain[PythonToolchainInfo].native_link_strategy)

# We do a lot of merging extensions, so don't use O(n) type annotations
def _merge_extensions(
        # {str: ("_a", "label")}
        extensions,
        # Label
        incoming_label,
        # {str: "_a"}
        incoming_extensions) -> None:
    """
    Merges a incoming_extensions into `extensions`. Fails if duplicate dests exist.
    """
    for extension_name, incoming_artifact in incoming_extensions.items():
        existing = extensions.get(extension_name)
        if existing != None and existing[0] != incoming_artifact:
            existing_artifact, existing_label = existing
            error = (
                "Duplicate extension: {}! Conflicting mappings:\n" +
                "{} from {}\n" +
                "{} from {}"
            )
            fail(
                error.format(
                    extension_name,
                    existing_artifact,
                    existing_label,
                    incoming_artifact,
                    incoming_label,
                ),
            )
        extensions[extension_name] = (incoming_artifact, incoming_label)

def _get_root_link_group_specs(
        libs: list[LinkableProviders],
        extensions: dict[str, LinkableProviders]) -> list[LinkGroupLibSpec]:
    """
    Walk the linkable graph finding dlopen-able C++ libs.
    """

    # TODO(agallagher): We should handle `allow_embedding = False` C++ extensions
    # here too.

    specs = []

    # Add link group specs for dlopen-able libs.
    for dep in libs:
        specs.append(
            LinkGroupLibSpec(
                name = dep.linkable_root_info.name,
                is_shared_lib = True,
                root = dep.linkable_root_info,
                label = dep.linkable_graph.nodes.value.label,
                group = Group(
                    name = dep.linkable_root_info.name,
                    mappings = [
                        GroupMapping(
                            roots = [dep.linkable_graph.nodes.value.label],
                            traversal = Traversal("node"),
                        ),
                    ],
                    # TODO(@christylee): Add attributes to python dlopen-able libs
                    attrs = GroupAttrs(
                        enable_distributed_thinlto = False,
                    ),
                ),
            ),
        )

    # Add link group specs for extensions.
    for name, extension in extensions.items():
        specs.append(
            LinkGroupLibSpec(
                name = name,
                is_shared_lib = False,
                root = extension.linkable_root_info,
                group = Group(
                    name = name,
                    mappings = [
                        GroupMapping(
                            roots = [extension.linkable_graph.nodes.value.label],
                            traversal = Traversal("node"),
                        ),
                    ],
                    # TODO(@christylee): Add attributes to extensions
                    attrs = GroupAttrs(
                        enable_distributed_thinlto = False,
                    ),
                ),
            ),
        )

    return specs

def _get_shared_only_groups(shared_only_libs: list[LinkableProviders]) -> list[Group]:
    """
    Create link group mappings for shared-only libs that'll force the link to
    link them dynamically.
    """

    groups = []

    # Add link group specs for dlopen-able libs.
    for dep in shared_only_libs:
        if dep.linkable_graph == None:
            continue
        groups.append(
            Group(
                name = str(dep.linkable_graph.nodes.value.label.raw_target()),
                mappings = [
                    GroupMapping(
                        roots = [dep.linkable_graph.nodes.value.label],
                        traversal = Traversal("node"),
                        preferred_linkage = Linkage("shared"),
                    ),
                ],
                # TODO(@christylee): Add attributes to python dlopen-able libs
                attrs = GroupAttrs(
                    enable_distributed_thinlto = False,
                ),
            ),
        )
    return groups

def _get_link_group_info(
        ctx: AnalysisContext,
        link_deps: list[LinkableProviders],
        libs: list[LinkableProviders],
        extensions: dict[str, LinkableProviders],
        shared_only_libs: list[LinkableProviders]) -> (LinkGroupInfo, list[LinkGroupLibSpec]):
    """
    Return the `LinkGroupInfo` and link group lib specs to use for this binary.
    This will handle parsing the various user-specific parameters and automatic
    link group lib spec generation for dlopen-enabled native libraries and,
    eventually, extensions.
    """

    link_group_info = get_link_group_info(ctx, [d.linkable_graph for d in link_deps])
    link_group_specs = []

    # Add link group specs from user-provided link group info.
    if link_group_info != None:
        link_group_specs.extend(create_shared_lib_link_group_specs(ctx, link_group_info))

    # Add link group specs from dlopenable C++ libraries.
    root_specs = _get_root_link_group_specs(libs, extensions)

    # Add link group specs for shared-only libs, which makes sure we link
    # against them dynamically.
    shared_groups = _get_shared_only_groups(shared_only_libs)

    # (Re-)build the link group info
    if root_specs or shared_groups or link_group_info == None:
        # We prepend the dlopen roots, so that they take precedence over
        # user-specific ones.
        link_group_specs = root_specs + link_group_specs

        # Regenerate the new `LinkGroupInfo` with the new link group lib
        # groups.
        linkable_graph = LinkableGraph(
            #label = ctx.label,
            nodes = ctx.actions.tset(
                LinkableGraphTSet,
                children = (
                    [d.linkable_graph.nodes for d in link_deps] +
                    [d.linkable_graph.nodes for d in libs] +
                    [d.linkable_graph.nodes for d in extensions.values()] +
                    [d.linkable_graph.nodes for d in shared_only_libs]
                ),
            ),
        )

        # We add user-defined mappings last, so that our auto-generated
        # ones get precedence (as we rely on this for things to work).
        link_groups = [s.group for s in root_specs] + shared_groups
        if link_group_info != None:
            link_groups += link_group_info.groups.values()

        link_group_info = build_link_group_info(
            graph = linkable_graph,
            groups = link_groups,
            min_node_count = ctx.attrs.link_group_min_binary_node_count,
        )

    return (link_group_info, link_group_specs)

def _qualify_entry_point(main: EntryPoint, base_module: str) -> EntryPoint:
    qualname = main[1]
    fqname = qualname
    if qualname.startswith("."):
        fqname = base_module + qualname
        if fqname.startswith("."):
            fqname = fqname[1:]
    return (main[0], fqname)

def python_executable(
        ctx: AnalysisContext,
        main: EntryPoint,
        srcs: dict[str, Artifact],
        default_resources: dict[str, ArtifactOutputs],
        standalone_resources: dict[str, ArtifactOutputs] | None,
        compile: bool,
        allow_cache_upload: bool) -> PexProviders:
    # Returns a three tuple: the Python binary, all its potential runtime files,
    # and a provider for its source DB.

    # TODO(nmj): See if people are actually setting cxx_platform here. Really
    #                 feels like it should be a property of the python platform
    python_platform = ctx.attrs._python_toolchain[PythonPlatformInfo]
    cxx_toolchain = ctx.attrs._cxx_toolchain

    raw_deps = ctx.attrs.deps

    raw_deps.extend(flatten(
        get_platform_attr(python_platform, cxx_toolchain, ctx.attrs.platform_deps),
    ))

    # `preload_deps` is used later to configure `LD_PRELOAD` environment variable,
    # here we make the actual libraries to appear in the distribution.
    # TODO: make fully consistent with its usage later
    raw_deps.extend(ctx.attrs.preload_deps)
    python_deps, shared_deps = gather_dep_libraries(raw_deps)

    src_manifest = None
    bytecode_manifest = None

    python_toolchain = ctx.attrs._python_toolchain[PythonToolchainInfo]
    if python_toolchain.runtime_library and ArtifactGroupInfo in python_toolchain.runtime_library:
        for artifact in python_toolchain.runtime_library[ArtifactGroupInfo].artifacts:
            srcs[artifact.short_path] = artifact

    if srcs:
        src_manifest = create_manifest_for_source_map(ctx, "srcs", srcs)
        bytecode_manifest = compile_manifests(ctx, [src_manifest])

    all_default_resources = {}
    all_standalone_resources = {}
    cxx_extra_resources = {}
    for cxx_resources in gather_resources(ctx.label, deps = raw_deps).values():
        for name, resource in cxx_resources.items():
            cxx_extra_resources[paths.join("__cxx_resources__", name)] = resource
    all_default_resources.update(cxx_extra_resources)
    all_standalone_resources.update(cxx_extra_resources)

    if default_resources:
        all_default_resources.update(default_resources)
    if standalone_resources:
        all_standalone_resources.update(standalone_resources)

    library_info = create_python_library_info(
        ctx.actions,
        ctx.label,
        srcs = src_manifest,
        src_types = src_manifest,
        default_resources = py_resources(ctx, all_default_resources) if all_default_resources else None,
        standalone_resources = py_resources(ctx, all_standalone_resources, "_standalone") if all_standalone_resources else None,
        bytecode = bytecode_manifest,
        deps = python_deps,
        shared_libraries = shared_deps,
    )

    source_db_no_deps = create_source_db_no_deps(ctx, srcs)

    dbg_source_db_output = ctx.actions.declare_output("dbg-db.json")
    dbg_source_db = create_dbg_source_db(ctx, dbg_source_db_output, src_manifest, python_deps)

    exe = _convert_python_library_to_executable(
        ctx,
        _qualify_entry_point(
            main,
            ctx.attrs.base_module if ctx.attrs.base_module != None else ctx.label.package.replace("/", "."),
        ),
        info_to_interface(library_info),
        raw_deps,
        compile,
        allow_cache_upload,
        dbg_source_db_output,
    )

    exe = PexProviders(
        default_output = exe.default_output,
        other_outputs = exe.other_outputs,
        other_outputs_prefix = exe.other_outputs_prefix,
        hidden_resources = exe.hidden_resources,
        sub_targets = exe.sub_targets,
        run_cmd = exe.run_cmd,
        dbg_source_db = dbg_source_db_output,
    )

    exe.sub_targets.update({
        "dbg-source-db": [dbg_source_db],
        "library-info": [library_info],
        "main": [DefaultInfo(default_output = ctx.actions.write_json("main.json", main))],
        "source-db-no-deps": [source_db_no_deps, create_python_source_db_info(library_info.manifests)],
    })

    # Type check
    type_checker = python_toolchain.type_checker
    if type_checker != None:
        exe.sub_targets.update({
            "typecheck": [
                create_per_target_type_check(
                    ctx,
                    type_checker,
                    src_manifest,
                    python_deps,
                    typeshed = python_toolchain.typeshed_stubs,
                    py_version = ctx.attrs.py_version_for_type_checking,
                    typing_enabled = ctx.attrs.typing,
                    sharding_enabled = ctx.attrs.shard_typing,
                ),
            ],
        })

    return exe

def _convert_python_library_to_executable(
        ctx: AnalysisContext,
        main: EntryPoint,
        library: PythonLibraryInterface,
        deps: list[Dependency],
        compile: bool,
        allow_cache_upload: bool,
        dbg_source_db: [Artifact, None]) -> PexProviders:
    extra = {}

    python_toolchain = ctx.attrs._python_toolchain[PythonToolchainInfo]
    package_style = get_package_style(ctx)

    # Convert preloaded deps to a set of their names to be loaded by.
    preload_labels = {_linkable_graph(d).label: None for d in ctx.attrs.preload_deps if _linkable_graph(d)}

    extensions = {}
    extra_artifacts = {}
    link_args = []
    for manifest in library.iter_manifests():
        if manifest.extensions:
            _merge_extensions(extensions, manifest.label, manifest.extensions)

    if ctx.attrs._cxx_toolchain.get(CxxToolchainInfo) == None:
        # In fat target platforms, there may not be a CXX toolchain available.
        shared_libs = [
            ("", shared_lib)
            for shared_lib in library.shared_libraries()
        ] + [
            ("", shared_lib)
            for shared_lib in library.extension_shared_libraries()
        ]
    elif _link_strategy(ctx) == NativeLinkStrategy("merged"):
        # If we're using omnibus linking, re-link libraries and extensions and
        # update the libraries we'll pull into the final binary.

        # Collect omnibus info from deps.
        linkable_graph = create_linkable_graph(
            ctx,
            deps = deps,
        )

        omnibus_graph = get_omnibus_graph(
            graph = linkable_graph,
            # Add in any potential native root targets from our first-order deps.
            roots = get_roots(deps),
            # Exclude preloaded deps from omnibus linking, to prevent preloading
            # the monolithic omnibus library.
            excluded = get_excluded(deps = ctx.attrs.preload_deps),
        )

        # Link omnibus libraries.
        omnibus_libs = create_omnibus_libraries(
            ctx,
            omnibus_graph,
            extra_ldflags = (
                # TODO(agallagher): Should these "binary" linker flags comes
                # from the Python toolchain instead?
                get_cxx_toolchain_info(ctx).linker_info.binary_linker_flags +
                python_toolchain.linker_flags +
                ctx.attrs.linker_flags
            ),
            prefer_stripped_objects = ctx.attrs.prefer_stripped_native_objects,
        )

        # Extract re-linked extensions.
        extensions = {
            dest: (omnibus_libs.roots[label].shared_library, label)
            for dest, (_, label) in extensions.items()
        }
        shared_libs = [("", shlib) for shlib in omnibus_libs.libraries]

        omnibus_providers = []

        if omnibus_libs.omnibus != None:
            omnibus_link_result = omnibus_libs.omnibus
            omnibus_linked_obj = omnibus_link_result.linked_object

            sub_targets = {}
            sub_targets["dwp"] = [DefaultInfo(default_output = omnibus_linked_obj.dwp if omnibus_linked_obj.dwp else None)]
            if omnibus_link_result.linker_map_data != None:
                sub_targets["linker-map"] = [DefaultInfo(default_output = omnibus_link_result.linker_map_data.map, other_outputs = [omnibus_link_result.linker_map_data.binary])]
            omnibus_info = DefaultInfo(
                default_output = omnibus_linked_obj.output,
                sub_targets = sub_targets,
            )
        else:
            omnibus_info = DefaultInfo()
        omnibus_providers.append(omnibus_info)

        if python_toolchain.emit_omnibus_metadata:
            omnibus_providers.append(
                OmnibusMetadataInfo(
                    omnibus_libs = omnibus_libs,
                    omnibus_graph = omnibus_graph,
                ),
            )

            exclusion_roots = ctx.actions.write_json("omnibus/exclusion_roots.json", omnibus_libs.exclusion_roots)
            extra["omnibus-exclusion-roots"] = [DefaultInfo(default_output = exclusion_roots)]

            roots = ctx.actions.write_json("omnibus/roots.json", omnibus_libs.roots)
            extra["omnibus-roots"] = [DefaultInfo(default_output = roots)]

            omnibus_excluded = ctx.actions.write_json("omnibus/excluded.json", omnibus_libs.excluded)
            extra["omnibus-excluded"] = [DefaultInfo(default_output = omnibus_excluded)]

            omnibus_graph_json = ctx.actions.write_json("omnibus_graph.json", omnibus_graph)
            extra["linkable-graph"] = [DefaultInfo(default_output = omnibus_graph_json)]

        extra["omnibus"] = omnibus_providers

    elif _link_strategy(ctx) == NativeLinkStrategy("native"):
        executable_deps = ctx.attrs.executable_deps
        extension_info = merge_cxx_extension_info(
            ctx.actions,
            deps + executable_deps,
            # Add in dlopen-enabled libs from first-order deps.
            shared_deps = ctx.attrs.deps + ctx.attrs.preload_deps,
        )
        extension_info_reduced = reduce_cxx_extension_info(extension_info)
        inherited_preprocessor_info = cxx_inherited_preprocessor_infos(executable_deps)

        # Generate an additional C file as input
        static_extension_info_out = ctx.actions.declare_output("static_extension_info.cpp")
        cmd = cmd_args(python_toolchain.generate_static_extension_info[RunInfo])
        cmd.add(cmd_args(static_extension_info_out.as_output(), format = "--output={}"))
        cmd.add(
            extension_info.set.project_as_args("python_module_names"),
        )

        # TODO we don't need to do this ...
        ctx.actions.run(cmd, category = "generate_static_extension_info")

        extra["static_extension_info"] = [DefaultInfo(default_output = static_extension_info_out)]

        cxx_executable_srcs = [
            CxxSrcWithFlags(file = ctx.attrs.cxx_main, flags = []),
            CxxSrcWithFlags(file = ctx.attrs.static_extension_utils, flags = []),
            CxxSrcWithFlags(file = static_extension_info_out, flags = []),
        ]
        extra_preprocessors = []
        if ctx.attrs.par_style == "native":
            extra_preprocessors.append(CPreprocessor(args = CPreprocessorArgs(args = ["-DNATIVE_PAR_STYLE=1"])))

        # All deps inolved in the link.
        link_deps = (
            linkables(executable_deps + ctx.attrs.preload_deps) +
            extension_info_reduced.linkable_providers
        )

        link_group_info, auto_link_group_specs = _get_link_group_info(
            ctx,
            link_deps,
            extension_info_reduced.dlopen_deps,
            extension_info_reduced.unembeddable_extensions,
            extension_info_reduced.shared_only_libs,
        )

        extra_binary_link_flags = []

        extra_binary_link_flags.extend(python_toolchain.binary_linker_flags)

        # Set rpaths to find 1) the shared libs dir and the 2) runtime libs dir.
        rpath_ref = get_rpath_origin(get_cxx_toolchain_info(ctx).linker_info.type)
        rpath_ldflag = "-Wl,-rpath,{}/".format(rpath_ref)
        if package_style == PackageStyle("standalone"):
            extra_binary_link_flags.append(rpath_ldflag + "../..")
            extra_binary_link_flags.append(rpath_ldflag + "../lib")
        else:
            rpath_ldflag_prefix = rpath_ldflag + "{}#link-tree".format(ctx.attrs.name)
            extra_binary_link_flags.append(rpath_ldflag_prefix + "/runtime/lib")
            extra_binary_link_flags.append(rpath_ldflag_prefix)

        impl_params = CxxRuleConstructorParams(
            rule_type = "python_binary",
            headers_layout = cxx_get_regular_cxx_headers_layout(ctx),
            srcs = cxx_executable_srcs,
            extra_binary_link_flags = extra_binary_link_flags,
            extra_link_flags = python_toolchain.linker_flags,
            extra_preprocessors = extra_preprocessors,
            extra_preprocessors_info = inherited_preprocessor_info,
            extra_link_deps = link_deps,
            exe_shared_libs_link_tree = False,
            force_full_hybrid_if_capable = True,
            prefer_stripped_objects = ctx.attrs.prefer_stripped_native_objects,
            link_group_info = link_group_info,
            auto_link_group_specs = auto_link_group_specs,
            exe_category_suffix = "python_exe",
            extra_shared_libs = traverse_shared_library_info(
                merge_shared_libraries(
                    actions = ctx.actions,
                    deps =
                        [d.shared_library_info for d in extension_info_reduced.shared_only_libs],
                ),
            ),
            extra_link_roots = (
                extension_info_reduced.unembeddable_extensions.values() +
                extension_info_reduced.dlopen_deps +
                extension_info_reduced.shared_only_libs +
                linkables(ctx.attrs.link_group_deps)
            ),
            exe_allow_cache_upload = allow_cache_upload,
            compiler_flags = ctx.attrs.compiler_flags,
            lang_compiler_flags = ctx.attrs.lang_compiler_flags,
            platform_compiler_flags = ctx.attrs.platform_compiler_flags,
            lang_platform_compiler_flags = ctx.attrs.lang_platform_compiler_flags,
            preprocessor_flags = ctx.attrs.preprocessor_flags,
            lang_preprocessor_flags = ctx.attrs.lang_preprocessor_flags,
            platform_preprocessor_flags = ctx.attrs.platform_preprocessor_flags,
            lang_platform_preprocessor_flags = ctx.attrs.lang_platform_preprocessor_flags,
        )

        executable_info = cxx_executable(ctx, impl_params)
        extra["native-executable"] = [DefaultInfo(default_output = executable_info.binary, sub_targets = executable_info.sub_targets)]

        # Add sub-targets for libs.
        for shlib in executable_info.shared_libs:
            # TODO(agallagher) There appears to be pre-existing soname conflicts
            # when building this (when using link groups), which prevents using
            # `with_unique_str_sonames`.
            if shlib.soname.is_str:
                extra[shlib.soname.ensure_str()] = [DefaultInfo(default_output = shlib.lib.output)]

        for name, group in executable_info.auto_link_groups.items():
            extra[name] = [DefaultInfo(default_output = group.output)]

        # Unembeddable extensions.
        extensions = {
            name: (
                executable_info.auto_link_groups[name],
                link.linkable_graph.nodes.value.label,
            )
            for name, link in extension_info_reduced.unembeddable_extensions.items()
        }

        # Put native libraries into the runtime location, as we need to unpack
        # potentially all of them before startup.
        shared_libs = [("runtime/lib", s) for s in executable_info.shared_libs]

        # TODO expect(len(executable_info.runtime_files) == 0, "OH NO THERE ARE RUNTIME FILES")
        extra_artifacts.update(extension_info_reduced.artifacts)
        shared_libs.append((
            "runtime/bin",
            create_shlib(
                soname = ctx.attrs.executable_name,
                label = ctx.label,
                lib = LinkedObject(
                    output = executable_info.binary,
                    unstripped_output = executable_info.binary,
                    dwp = executable_info.dwp,
                ),
            ),
        ))

        link_args = executable_info.link_args
        extra_artifacts["static_extension_finder.py"] = ctx.attrs.static_extension_finder
    else:
        shared_libs = [
            ("", shared_lib)
            for shared_lib in library.shared_libraries()
        ]

        if (not ctx.attrs.standalone_extensions) or ctx.attrs.link_style == "shared":
            # darwin and windows expect self-contained dynamically linked
            # python extensions without additional transitive shared libraries
            shared_libs += [
                ("", extension_shared_lib)
                for extension_shared_lib in library.extension_shared_libraries()
            ]

    if dbg_source_db:
        extra_artifacts["dbg-db.json"] = dbg_source_db

    if python_toolchain.default_sitecustomize != None:
        extra_artifacts["sitecustomize.py"] = python_toolchain.default_sitecustomize

    extra_manifests = create_manifest_for_source_map(ctx, "extra_manifests", extra_artifacts)

    # Create the map of native libraries to their artifacts and whether they
    # need to be preloaded.  Note that we merge preload deps into regular deps
    # above, before gathering up all native libraries, so we're guaranteed to
    # have all preload libraries (and their transitive deps) here.
    shared_libs = [
        (libdir, shlib, shlib.label in preload_labels)
        for libdir, shlib in shared_libs
    ]

    # Strip native libraries and extensions and update the .gnu_debuglink references if we are extracting
    # debug symbols from the par
    debuginfo_files = []
    debuginfos = {}
    if ctx.attrs.strip_libpar == "extract" and package_style == PackageStyle("standalone") and cxx_is_gnu(ctx):
        stripped_shlibs = []
        for libdir, shlib, preload in shared_libs:
            name = paths.join(
                libdir,
                value_or(
                    shlib.soname.as_str(),
                    shlib.lib.unstripped_output.short_path,
                ),
            )
            existing = debuginfos.get(name)
            if existing == None:
                stripped, debuginfo = strip_debug_with_gnu_debuglink(
                    ctx = ctx,
                    name = name,
                    obj = shlib.lib.unstripped_output,
                )
                debuginfos[name] = (stripped, debuginfo)
            else:
                stripped, debuginfo = existing
            shlib = SharedLibrary(
                soname = shlib.soname,
                label = shlib.label,
                lib = LinkedObject(
                    output = stripped,
                    unstripped_output = shlib.lib.unstripped_output,
                    dwp = shlib.lib.dwp,
                ),
            )
            stripped_shlibs.append((libdir, shlib, preload))
            debuginfo_files.append(((libdir, shlib, ".debuginfo"), debuginfo))
        shared_libs = stripped_shlibs
        for name, (extension, label) in extensions.items():
            stripped, debuginfo = strip_debug_with_gnu_debuglink(
                ctx = ctx,
                name = name,
                obj = extension.unstripped_output,
            )
            extensions[name] = (
                LinkedObject(
                    output = stripped,
                    unstripped_output = extension.unstripped_output,
                    dwp = extension.dwp,
                ),
                label,
            )
            debuginfo_files.append((name + ".debuginfo", debuginfo))

    # Combine sources and extensions into a map of all modules.
    pex_modules = PexModules(
        manifests = library.manifests(),
        extra_manifests = extra_manifests,
        compile = compile,
        extensions = create_manifest_for_extensions(
            ctx,
            extensions,
            dwp = ctx.attrs.package_split_dwarf_dwp,
        ) if extensions else None,
    )

    # Build the PEX.
    pex = make_py_package(
        ctx = ctx,
        python_toolchain = python_toolchain,
        make_py_package_cmd = ctx.attrs.make_py_package[RunInfo] if ctx.attrs.make_py_package != None else None,
        package_style = package_style,
        build_args = ctx.attrs.build_args,
        pex_modules = pex_modules,
        shared_libraries = shared_libs,
        main = main,
        allow_cache_upload = allow_cache_upload,
        debuginfo_files = debuginfo_files,
        link_args = link_args,
    )

    pex.sub_targets.update(extra)

    return pex

def python_binary_impl(ctx: AnalysisContext) -> list[Provider]:
    main_module = ctx.attrs.main_module
    main_function = ctx.attrs.main_function
    if main_module != None and ctx.attrs.main != None:
        fail("Only one of main_module or main may be set. Prefer main_function as main and main_module are considered deprecated")
    elif main_module != None and main_function != None:
        fail("Only one of main_module or main_function may be set. Prefer main_function.")
    elif ctx.attrs.main != None and main_function == None:
        main_module = "." + ctx.attrs.main.short_path.replace("/", ".")
        if main_module.endswith(".py"):
            main_module = main_module[:-3]

    # if "python-version=3.8" in ctx.attrs.labels:
    #     # buildifier: disable=print
    #     print((
    #         "\033[1;33m \u26A0 [Warning] " +
    #         "{0} 3.8 is EOL, and is going away by the end of H1 2024. " +
    #         "This build triggered //{1}:{2} which still uses {0} 3.8. " +
    #         "Make sure someone (you or the appropriate maintainers) upgrades it to {0} 3.10 soon to avoid breakages. " +
    #         "https://fburl.com/python-eol \033[0m"
    #     ).format(
    #         "Cinder" if "python-flavor=cinder" in ctx.attrs.labels else "Python",
    #         ctx.label.package,
    #         ctx.attrs.name,
    #     ))

    if main_module != None:
        main = (EntryPointKind("module"), main_module)
    else:
        main = (EntryPointKind("function"), main_function)

    srcs = {}
    if ctx.attrs.main != None:
        srcs[ctx.attrs.main.short_path] = ctx.attrs.main
    srcs = qualify_srcs(ctx.label, ctx.attrs.base_module, srcs)
    default_resources_map, standalone_resources_map = py_attr_resources(ctx)
    standalone_resources = qualify_srcs(ctx.label, ctx.attrs.base_module, standalone_resources_map)
    default_resources = qualify_srcs(ctx.label, ctx.attrs.base_module, default_resources_map)

    pex = python_executable(
        ctx,
        main,
        srcs,
        default_resources,
        standalone_resources,
        compile = value_or(ctx.attrs.compile, False),
        allow_cache_upload = cxx_attrs_get_allow_cache_upload(ctx.attrs),
    )
    return [
        make_default_info(pex),
        make_run_info(pex, ctx.attrs.run_with_inplace),
    ]
