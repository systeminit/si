use std::{
    collections::{
        HashMap,
        hash_map::Entry,
    },
    ops::Deref,
};

use si_pkg::{
    ActionFuncSpec,
    AttrFuncInputSpec,
    AttrFuncInputSpecKind,
    AuthenticationFuncSpec,
    ComponentSpec,
    EdgeSpec,
    FuncArgumentSpec,
    FuncSpec,
    FuncSpecData,
    LeafFunctionSpec,
    ManagementFuncSpec,
    MapKeyFuncSpec,
    PkgSpec,
    PropSpec,
    PropSpecBuilder,
    PropSpecKind,
    RootPropFuncSpec,
    SchemaSpec,
    SchemaSpecData,
    SchemaVariantSpec,
    SchemaVariantSpecBuilder,
    SchemaVariantSpecData,
    SchemaVariantSpecPropRoot,
    SiPkg,
    SiPkgKind,
    SiPropFuncSpec,
    SiPropFuncSpecKind,
    SocketSpec,
    SocketSpecData,
    SocketSpecKind,
    SpecError,
};
use strum::IntoEnumIterator;
use telemetry::prelude::*;

use super::{
    PkgResult,
    import_pkg_from_pkg,
};
use crate::{
    AttributePrototype,
    AttributePrototypeId,
    DalContext,
    Func,
    FuncBackendKind,
    FuncId,
    InputSocket,
    OutputSocket,
    Prop,
    PropId,
    PropKind,
    Schema,
    SchemaId,
    SchemaVariant,
    SchemaVariantId,
    Workspace,
    action::prototype::ActionPrototype,
    attribute::prototype::argument::{
        AttributePrototypeArgument,
        AttributePrototypeArgumentId,
        value_source::ValueSource,
    },
    func::{
        FuncKind,
        argument::FuncArgument,
        intrinsics::IntrinsicFunc,
    },
    management::prototype::ManagementPrototype,
    pkg::PkgError,
    prop::PropPath,
    schema::variant::leaves::{
        LeafInputLocation,
        LeafKind,
    },
};

pub type FuncSpecMap = super::ChangeSetThingMap<FuncId, FuncSpec>;
type VariantSpecMap = super::ChangeSetThingMap<SchemaVariantId, SchemaVariantSpec>;

pub struct PkgExporter {
    name: String,
    version: String,
    description: Option<String>,
    kind: SiPkgKind,
    created_by: String,
    schema_ids: Option<Vec<SchemaId>>,
    func_map: FuncSpecMap,
    variant_map: VariantSpecMap,
    include_transformations: bool,
}

impl PkgExporter {
    /// Creates an unopinionated [`PkgExporter`].
    ///
    /// _Note:_ if you are unsure which constructor method to use, you likely want
    /// [`Self::new_for_module_contribution`].
    fn new(
        name: impl Into<String>,
        version: impl Into<String>,
        description: Option<impl Into<String>>,
        created_by: impl Into<String>,
        schema_ids: Vec<SchemaId>,
        include_transformations: bool,
    ) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            description: description.map(Into::into),
            kind: SiPkgKind::Module,
            created_by: created_by.into(),
            schema_ids: Some(schema_ids),
            func_map: FuncSpecMap::new(),
            variant_map: VariantSpecMap::new(),
            include_transformations,
        }
    }

    /// Creates a new [`PkgExporter`] for contributing an individual module.
    pub fn new_for_module_contribution(
        name: impl Into<String>,
        version: impl Into<String>,
        created_by: impl Into<String>,
        schema_id: SchemaId,
        include_transformations: bool,
    ) -> Self {
        Self::new(
            name,
            version,
            None::<String>,
            created_by,
            vec![schema_id],
            include_transformations,
        )
    }

    fn new_standalone_variant_exporter(schema_name: &str) -> Self {
        Self::new(schema_name, "", None::<String>, "", vec![], false)
    }

    pub async fn export_as_bytes(&mut self, ctx: &DalContext) -> PkgResult<Vec<u8>> {
        match self.kind {
            SiPkgKind::Module => info!("Building module package"),
            SiPkgKind::WorkspaceBackup => return Err(PkgError::WorkspaceExportNotSupported()),
        }

        let pkg = self.export(ctx).await?;

        info!("Exporting as bytes");

        Ok(pkg.write_to_bytes()?)
    }

    async fn export_schema(
        &mut self,
        ctx: &DalContext,
        schema: &Schema,
    ) -> PkgResult<(SchemaSpec, Vec<FuncSpec>)> {
        let mut funcs = vec![];
        let schema_is_builtin = schema.is_builtin();

        let default_variant = SchemaVariant::default_for_schema(ctx, schema.id()).await?;

        let mut schema_spec_builder = SchemaSpec::builder();
        schema_spec_builder.name(schema.name());
        schema_spec_builder.unique_id(schema.id().to_string());

        let mut default_variant_unique_id = None;

        let variant_is_builtin = default_variant.is_builtin();
        let variant_category = default_variant.clone().category().to_owned();

        let variant_funcs = self
            .export_funcs_for_variant(ctx, default_variant.id(), None)
            .await?;
        funcs.extend(variant_funcs);

        let variant_spec = self
            .export_variant(ctx, &default_variant, variant_is_builtin, None)
            .await?;
        self.variant_map
            .insert(default_variant.id(), variant_spec.to_owned());
        if variant_spec.unique_id.is_some() {
            variant_spec
                .unique_id
                .clone_into(&mut default_variant_unique_id);

            schema_spec_builder.variant(variant_spec);
        }

        let mut data_builder = SchemaSpecData::builder();
        data_builder.name(schema.name());
        data_builder.ui_hidden(schema.ui_hidden());
        data_builder.category(variant_category.clone());
        if let Some(default_unique_id) = default_variant_unique_id {
            data_builder.default_schema_variant(default_unique_id);
        }
        schema_spec_builder.data(data_builder.build()?);
        schema_spec_builder.is_builtin(schema_is_builtin);

        let schema_spec = schema_spec_builder.build()?;

        Ok((schema_spec, funcs))
    }

    /// Exports just a single schema variant and the functions connected to it.
    pub async fn export_variant_standalone(
        ctx: &DalContext,
        variant: &SchemaVariant,
        schema_name: &str,
        overridden_asset_func_id: Option<FuncId>,
    ) -> PkgResult<(SchemaVariantSpec, Vec<FuncSpec>)> {
        let mut exporter = Self::new_standalone_variant_exporter(schema_name);
        let email = ctx.history_actor().email(ctx).await?;
        exporter.created_by = email;

        exporter
            .export_funcs_for_variant(ctx, variant.id(), overridden_asset_func_id)
            .await?;

        exporter.export_intrinsics(ctx).await?;
        let variant_spec = exporter
            .export_variant(ctx, variant, false, overridden_asset_func_id)
            .await?;

        let funcs = exporter
            .func_map
            .inner
            .values()
            .map(ToOwned::to_owned)
            .collect();

        Ok((variant_spec, funcs))
    }

    async fn export_variant(
        &mut self,
        ctx: &DalContext,
        variant: &SchemaVariant,
        variant_is_builtin: bool,
        overridden_asset_func_id: Option<FuncId>,
    ) -> PkgResult<SchemaVariantSpec> {
        let mut variant_spec_builder = SchemaVariantSpec::builder();
        variant_spec_builder.version(variant.version());
        variant_spec_builder.is_builtin(variant_is_builtin);
        variant_spec_builder.unique_id(variant.id().to_string());
        let mut data_builder = SchemaVariantSpecData::builder();

        data_builder.version(variant.version());
        data_builder.color(variant.color());

        if let Some(link) = variant.link() {
            data_builder.try_link(link.to_string().deref())?;
        }
        data_builder.display_name(variant.display_name());

        data_builder.component_type(variant.component_type());
        data_builder.description(variant.description());

        if let Some(authoring_func_id) =
            overridden_asset_func_id.or_else(|| variant.asset_func_id())
        {
            let asset_func_unique_id = self
                .func_map
                .get(&authoring_func_id)
                .ok_or(PkgError::MissingFuncUniqueId(
                    authoring_func_id.to_string(),
                    "error found while exporting variant",
                ))?
                .unique_id
                .to_owned();
            data_builder.func_unique_id(asset_func_unique_id);
        }

        variant_spec_builder.data(data_builder.build()?);

        self.export_prop_tree(
            ctx,
            variant,
            &mut variant_spec_builder,
            SchemaVariantSpecPropRoot::Domain,
            false,
        )
        .await?;

        self.export_prop_tree(
            ctx,
            variant,
            &mut variant_spec_builder,
            SchemaVariantSpecPropRoot::ResourceValue,
            false,
        )
        .await?;

        self.export_prop_tree(
            ctx,
            variant,
            &mut variant_spec_builder,
            SchemaVariantSpecPropRoot::Secrets,
            false,
        )
        .await?;

        self.export_prop_tree(
            ctx,
            variant,
            &mut variant_spec_builder,
            SchemaVariantSpecPropRoot::SecretDefinition,
            true,
        )
        .await?;

        self.export_leaf_funcs(ctx, variant.id())
            .await?
            .drain(..)
            .for_each(|leaf_func_spec| {
                variant_spec_builder.leaf_function(leaf_func_spec);
            });

        self.export_sockets(ctx, variant.id())
            .await?
            .drain(..)
            .for_each(|socket_spec| {
                variant_spec_builder.socket(socket_spec);
            });

        self.export_action_funcs(ctx, variant.id())
            .await?
            .drain(..)
            .for_each(|action_func_spec| {
                variant_spec_builder.action_func(action_func_spec);
            });

        self.export_management_funcs(ctx, variant.id())
            .await?
            .drain(..)
            .for_each(|management_func_spec| {
                variant_spec_builder.management_func(management_func_spec);
            });

        self.export_auth_funcs(ctx, variant.id())
            .await?
            .drain(..)
            .for_each(|spec| {
                variant_spec_builder.auth_func(spec);
            });

        self.export_si_prop_funcs(ctx, variant.id())
            .await?
            .drain(..)
            .for_each(|si_prop_func_spec| {
                variant_spec_builder.si_prop_func(si_prop_func_spec);
            });

        self.export_root_prop_funcs(ctx, variant)
            .await?
            .drain(..)
            .for_each(|root_prop_func_spec| {
                variant_spec_builder.root_prop_func(root_prop_func_spec);
            });

        let variant_spec = variant_spec_builder.build()?;

        Ok(variant_spec)
    }

    async fn export_root_prop_funcs(
        &self,
        ctx: &DalContext,
        variant: &SchemaVariant,
    ) -> PkgResult<Vec<RootPropFuncSpec>> {
        let mut specs = vec![];

        for root_prop in SchemaVariantSpecPropRoot::iter() {
            if let Some(prop_id) = Prop::find_prop_id_by_path_opt(
                ctx,
                variant.id(),
                &PropPath::new(root_prop.path_parts()),
            )
            .await?
            {
                if let Some(prototype_id) =
                    AttributePrototype::find_for_prop(ctx, prop_id, &None).await?
                {
                    if let Some((func_unique_id, mut inputs)) = self
                        .export_input_func_and_arguments(ctx, prototype_id)
                        .await?
                    {
                        let mut builder = RootPropFuncSpec::builder();
                        builder.func_unique_id(func_unique_id).prop(root_prop);

                        inputs.drain(..).for_each(|input| {
                            builder.input(input);
                        });

                        specs.push(builder.build()?);
                    }
                }
            }
        }

        Ok(specs)
    }

    async fn export_si_prop_funcs(
        &self,
        ctx: &DalContext,
        variant_id: SchemaVariantId,
    ) -> PkgResult<Vec<SiPropFuncSpec>> {
        let _variant = SchemaVariant::get_by_id(ctx, variant_id).await?;
        let mut specs = vec![];

        for kind in SiPropFuncSpecKind::iter() {
            let prop =
                Prop::find_prop_by_path(ctx, variant_id, &PropPath::new(kind.prop_path())).await?;

            if let Some(prototype_id) =
                AttributePrototype::find_for_prop(ctx, prop.id, &None).await?
            {
                if let Some((func_unique_id, mut inputs)) = self
                    .export_input_func_and_arguments(ctx, prototype_id)
                    .await?
                {
                    let mut builder = SiPropFuncSpec::builder();
                    builder.func_unique_id(func_unique_id).kind(kind);

                    builder.unique_id(prototype_id.to_string());

                    inputs.drain(..).for_each(|input| {
                        builder.input(input);
                    });

                    specs.push(builder.build()?);
                }
            }
        }

        Ok(specs)
    }

    async fn export_leaf_funcs(
        &self,
        ctx: &DalContext,
        variant_id: SchemaVariantId,
    ) -> PkgResult<Vec<LeafFunctionSpec>> {
        let mut specs = vec![];

        for leaf_kind in LeafKind::iter() {
            for leaf_func_id in
                SchemaVariant::find_leaf_item_functions(ctx, variant_id, leaf_kind).await?
            {
                let func_spec = self
                    .func_map
                    .get(&leaf_func_id)
                    .ok_or(PkgError::MissingExportedFunc(leaf_func_id))?;

                let mut inputs = vec![];
                for arg in FuncArgument::list_for_func(ctx, leaf_func_id).await? {
                    let arg_name = arg.name;
                    inputs.push(
                        LeafInputLocation::maybe_from_arg_name(arg_name.clone())
                            .ok_or(SpecError::LeafInputLocationConversionError(
                                arg_name.clone(),
                            ))?
                            .into(),
                    );
                }

                let mut builder = LeafFunctionSpec::builder();

                specs.push(
                    builder
                        .func_unique_id(&func_spec.unique_id)
                        .leaf_kind(leaf_kind)
                        .inputs(inputs)
                        .build()?,
                );
            }
        }

        Ok(specs)
    }

    async fn export_sockets(
        &self,
        ctx: &DalContext,
        variant_id: SchemaVariantId,
    ) -> PkgResult<Vec<SocketSpec>> {
        let mut specs = vec![];

        for input_socket_id in InputSocket::list_ids_for_schema_variant(ctx, variant_id).await? {
            let socket = InputSocket::get_by_id(ctx, input_socket_id).await?;

            let mut socket_spec_builder = SocketSpec::builder();
            socket_spec_builder.name(socket.name());

            let mut data_builder = SocketSpecData::builder();
            let connection_annotation_str =
                serde_json::to_string(&socket.connection_annotations())?;

            data_builder
                .name(socket.name())
                .connection_annotations(connection_annotation_str)
                .kind(SocketSpecKind::Input)
                .arity(&socket.arity())
                .ui_hidden(socket.ui_hidden());

            if let Some(attr_proto_id) =
                AttributePrototype::find_for_input_socket(ctx, input_socket_id).await?
            {
                let _proto = AttributePrototype::get_by_id(ctx, attr_proto_id).await?;

                if let Some((func_unique_id, mut inputs)) = self
                    .export_input_func_and_arguments(ctx, attr_proto_id)
                    .await?
                {
                    data_builder.func_unique_id(func_unique_id);
                    inputs.drain(..).for_each(|input| {
                        socket_spec_builder.input(input);
                    });
                }
            }

            socket_spec_builder.data(data_builder.build()?);

            specs.push(socket_spec_builder.build()?);
        }
        for output_socket_id in OutputSocket::list_ids_for_schema_variant(ctx, variant_id).await? {
            let socket = OutputSocket::get_by_id(ctx, output_socket_id).await?;
            let mut socket_spec_builder = SocketSpec::builder();
            socket_spec_builder.name(socket.name());
            let mut data_builder = SocketSpecData::builder();
            let connection_annotation_str =
                serde_json::to_string(&socket.connection_annotations())?;

            data_builder
                .name(socket.name())
                .connection_annotations(connection_annotation_str)
                .kind(SocketSpecKind::Output)
                .arity(&socket.arity())
                .ui_hidden(socket.ui_hidden());

            if let Some(attr_proto_id) =
                AttributePrototype::find_for_output_socket(ctx, output_socket_id).await?
            {
                let proto = AttributePrototype::get_by_id(ctx, attr_proto_id).await?;
                if let Some((func_unique_id, mut inputs)) = self
                    .export_input_func_and_arguments(ctx, proto.id())
                    .await?
                {
                    data_builder.func_unique_id(func_unique_id);
                    inputs.drain(..).for_each(|input| {
                        socket_spec_builder.input(input);
                    });
                }
            }

            socket_spec_builder.data(data_builder.build()?);
            specs.push(socket_spec_builder.build()?);
        }

        Ok(specs)
    }

    async fn export_management_funcs(
        &self,
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> PkgResult<Vec<ManagementFuncSpec>> {
        let mut specs = vec![];
        let management_prototypes =
            ManagementPrototype::list_for_variant_id(ctx, schema_variant_id).await?;

        for management_proto in management_prototypes {
            let key = ManagementPrototype::func_id(ctx, management_proto.id()).await?;

            let func_spec = self
                .func_map
                .get(&key)
                .ok_or(PkgError::MissingExportedFunc(key))?;

            let mut builder = ManagementFuncSpec::builder();
            if let Some(description) = management_proto.description() {
                builder.description(description.to_string());
            }

            specs.push(
                builder
                    .func_unique_id(&func_spec.unique_id)
                    .name(management_proto.name())
                    .build()?,
            )
        }

        Ok(specs)
    }

    async fn export_action_funcs(
        &self,
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> PkgResult<Vec<ActionFuncSpec>> {
        let mut specs = vec![];
        // only export 1 of the Action Func Versions if there are both
        let action_prototypes = ActionPrototype::for_variant(ctx, schema_variant_id).await?;

        for action_proto in action_prototypes {
            let key = ActionPrototype::func_id(ctx, action_proto.id()).await?;

            let func_spec = self
                .func_map
                .get(&key)
                .ok_or(PkgError::MissingExportedFunc(key))?;

            let mut builder = ActionFuncSpec::builder();

            specs.push(
                builder
                    .kind(action_proto.kind)
                    .func_unique_id(&func_spec.unique_id)
                    .build()?,
            )
        }

        Ok(specs)
    }

    async fn export_auth_funcs(
        &self,
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> PkgResult<Vec<AuthenticationFuncSpec>> {
        let mut specs = vec![];
        let auth_funcs = SchemaVariant::list_auth_func_ids_for_id(ctx, schema_variant_id).await?;

        for auth_func in auth_funcs {
            let func_spec = self
                .func_map
                .get(&auth_func)
                .ok_or(PkgError::MissingExportedFunc(auth_func))?;

            let mut builder = AuthenticationFuncSpec::builder();

            specs.push(builder.func_unique_id(&func_spec.unique_id).build()?)
        }

        Ok(specs)
    }

    async fn export_prop_tree(
        &self,
        ctx: &DalContext,
        variant: &SchemaVariant,
        variant_spec: &mut SchemaVariantSpecBuilder,
        prop_root: SchemaVariantSpecPropRoot,
        is_optional_prop: bool,
    ) -> PkgResult<()> {
        let variant_id = variant.id();
        let prop_path = PropPath::new(prop_root.path_parts());
        let root_prop: Prop;
        if let Some(root_prop_id) =
            Prop::find_prop_id_by_path_opt(ctx, variant_id, &prop_path).await?
        {
            root_prop = Prop::get_by_id(ctx, root_prop_id).await?
        } else if is_optional_prop {
            return Ok(());
        } else {
            return Err(PkgError::PropNotFoundByName(prop_root.to_string()));
        }

        #[derive(Debug)]
        struct TraversalStackEntry {
            builder: PropSpecBuilder,
            prop_id: PropId,
            parent_prop_id: Option<PropId>,
        }

        let mut stack: Vec<(PropId, Option<PropId>)> = Vec::new();
        for child_tree_node in Prop::direct_child_prop_ids_ordered(ctx, root_prop.id()).await? {
            stack.push((child_tree_node, None));
        }

        let mut traversal_stack: Vec<TraversalStackEntry> = Vec::new();

        while let Some((prop_id, parent_prop_id)) = stack.pop() {
            let child_prop = Prop::get_by_id(ctx, prop_id).await?;
            let mut builder = PropSpec::builder();

            builder.unique_id(prop_id);

            builder
                .name(child_prop.name)
                .kind(match child_prop.kind {
                    PropKind::Json => PropSpecKind::Json,
                    PropKind::Array => PropSpecKind::Array,
                    PropKind::Boolean => PropSpecKind::Boolean,
                    PropKind::Integer => PropSpecKind::Number,
                    PropKind::Float => PropSpecKind::Float,
                    PropKind::Object => PropSpecKind::Object,
                    PropKind::String => PropSpecKind::String,
                    PropKind::Map => PropSpecKind::Map,
                })
                .hidden(child_prop.hidden)
                .widget_kind(child_prop.widget_kind);

            if let Some(widget_options) = child_prop.widget_options {
                builder.widget_options(serde_json::to_value(widget_options)?);
            }

            if let Some(doc_link) = child_prop.doc_link {
                builder.try_doc_link(doc_link.as_str())?;
            }

            if let Some(documentation) = child_prop.documentation {
                builder.documentation(documentation.as_str());
            }

            if let Some(validations) = child_prop.validation_format {
                builder.validation_format(validations.as_str());
            }

            if let Some(default_val) = Prop::default_value(ctx, prop_id).await? {
                builder.default_value(default_val);
            }

            traversal_stack.push(TraversalStackEntry {
                builder,
                prop_id,
                parent_prop_id,
            });

            for child_tree_node in Prop::direct_child_prop_ids_ordered(ctx, child_prop.id).await? {
                stack.push((child_tree_node, Some(prop_id)));
            }
        }

        let mut prop_children_map: HashMap<PropId, Vec<(PropSpec, PropId)>> = HashMap::new();

        while let Some(mut entry) = traversal_stack.pop() {
            let mut maybe_type_prop_id: Option<PropId> = None;

            if let Some(mut prop_children) = prop_children_map.remove(&entry.prop_id) {
                match entry.builder.get_kind() {
                    Some(kind) => match kind {
                        PropSpecKind::Object => {
                            entry.builder.entries(
                                prop_children
                                    .iter()
                                    .map(|(prop_spec, _)| prop_spec.to_owned())
                                    .collect(),
                            );
                        }
                        PropSpecKind::Map | PropSpecKind::Array => {
                            let (type_prop, type_prop_id) =
                                prop_children.pop().ok_or_else(|| {
                                    PkgError::PropSpecChildrenInvalid(format!(
                                        "found no child for map/array for prop id {}",
                                        entry.prop_id,
                                    ))
                                })?;
                            if !prop_children.is_empty() {
                                return Err(PkgError::PropSpecChildrenInvalid(format!(
                                    "found multiple children for map/array for prop id {}",
                                    entry.prop_id,
                                )));
                            }
                            entry.builder.type_prop(type_prop);
                            maybe_type_prop_id = Some(type_prop_id);
                        }
                        PropSpecKind::Json
                        | PropSpecKind::String
                        | PropSpecKind::Number
                        | PropSpecKind::Float
                        | PropSpecKind::Boolean => {
                            return Err(PkgError::PropSpecChildrenInvalid(format!(
                                "primitve prop type should have no children for prop id {}",
                                entry.prop_id,
                            )));
                        }
                    },
                    None => {
                        return Err(SpecError::UninitializedField("kind").into());
                    }
                };
            }

            if matches!(entry.builder.get_kind(), Some(PropSpecKind::Map)) {
                if let Some(type_prop_id) = maybe_type_prop_id {
                    for (maybe_key, proto) in Prop::prototypes_by_key(ctx, type_prop_id).await? {
                        if let Some(key) = maybe_key {
                            if let Some((func_unique_id, mut inputs)) =
                                self.export_input_func_and_arguments(ctx, proto).await?
                            {
                                let mut map_key_func_builder = MapKeyFuncSpec::builder();
                                map_key_func_builder.key(key);
                                map_key_func_builder.func_unique_id(func_unique_id);
                                inputs.drain(..).for_each(|input| {
                                    map_key_func_builder.input(input);
                                });
                                entry.builder.map_key_func(map_key_func_builder.build()?);
                            }
                        }
                    }
                }
            }

            if let Some(prototype) =
                AttributePrototype::find_for_prop(ctx, entry.prop_id, &None).await?
            {
                if let Some((func_unique_id, mut inputs)) =
                    self.export_input_func_and_arguments(ctx, prototype).await?
                {
                    entry.builder.has_data(true);

                    entry.builder.func_unique_id(func_unique_id);
                    inputs.drain(..).for_each(|input| {
                        entry.builder.input(input);
                    });
                }
            }

            let prop_spec = entry.builder.build()?;

            match entry.parent_prop_id {
                None => {
                    variant_spec.prop(prop_root, prop_spec);
                }
                Some(parent_prop_id) => {
                    match prop_children_map.entry(parent_prop_id) {
                        Entry::Occupied(mut occupied) => {
                            occupied.get_mut().push((prop_spec, entry.prop_id));
                        }
                        Entry::Vacant(vacant) => {
                            vacant.insert(vec![(prop_spec, entry.prop_id)]);
                        }
                    };
                }
            };
        }

        Ok(())
    }

    async fn export_input_func_and_arguments(
        &self,
        ctx: &DalContext,
        prototype_id: AttributePrototypeId,
    ) -> PkgResult<Option<(String, Vec<AttrFuncInputSpec>)>> {
        let _proto = AttributePrototype::get_by_id(ctx, prototype_id).await?;
        let func_id = AttributePrototype::func_id(ctx, prototype_id).await?;
        let proto_func = Func::get_by_id_opt(ctx, func_id).await?.ok_or(
            PkgError::MissingAttributePrototypeFunc(prototype_id, func_id),
        )?;

        let apas: Vec<AttributePrototypeArgumentId> =
            AttributePrototypeArgument::list_ids_for_prototype(ctx, prototype_id).await?;

        // If the prototype func is intrinsic and has no arguments, it's one that is created by default
        // and we don't have to track it in the package. However, if it was formerly a special case
        // func, then we will continue to treat it as such.
        if apas.is_empty() {
            match IntrinsicFunc::maybe_from_str(proto_func.name.as_str()) {
                Some(IntrinsicFunc::ResourcePayloadToValue)
                | Some(IntrinsicFunc::NormalizeToArray)
                    if proto_func.backend_kind == FuncBackendKind::JsAttribute => {}
                Some(_) => {
                    return Ok(None);
                }
                None => {}
            }
        }

        let mut inputs = vec![];

        for &apa_id in &apas {
            let func_arg_id = AttributePrototypeArgument::func_argument_id(ctx, apa_id).await?;
            let func_arg = FuncArgument::get_by_id(ctx, func_arg_id).await?;
            let arg_name = func_arg.name;

            let mut builder = AttrFuncInputSpec::builder();
            builder.unique_id(apa_id.to_string());

            builder.name(arg_name.clone());
            if let Some(value_source) =
                AttributePrototypeArgument::value_source_opt(ctx, apa_id).await?
            {
                match value_source {
                    ValueSource::InputSocket(input_socket_id) => {
                        // get the input arg from the other end of the socket and add to the list
                        let input_socket = InputSocket::get_by_id(ctx, input_socket_id).await?;
                        inputs.push(
                            builder
                                .name(arg_name.clone())
                                .kind(AttrFuncInputSpecKind::InputSocket)
                                .socket_name(input_socket.name())
                                .build()?,
                        );
                    }
                    ValueSource::OutputSocket(_) => {
                        // We don't want to create these on import of schema variants, so we don't care if
                        // we find it or not. But we do need to ensure the input length is correct for when
                        // we do this on *component import*, so that we don't modify the inputs to the
                        // attribute function on the component.
                    }

                    ValueSource::Prop(prop_id) => {
                        // get the prop name and add to the list
                        let prop = Prop::get_by_id(ctx, prop_id).await?.path(ctx).await?;

                        inputs.push(
                            builder
                                .kind(AttrFuncInputSpecKind::Prop)
                                .prop_path(prop)
                                .build()?,
                        );
                    }
                    // NOTE(nick): do we want to skip exporting secrets? Probably not... but maybe
                    // something that the user can toggle?
                    ValueSource::Secret(_)
                    | ValueSource::StaticArgumentValue(_)
                    | ValueSource::ValueSubscription(_) => {
                        // do nothing as this is irrelevant for the schema variant!
                    }
                }
            }
        }

        let func_spec = self
            .func_map
            .get(&func_id)
            .ok_or(PkgError::MissingExportedFunc(func_id))?;

        let func_unique_id = func_spec.unique_id.to_owned();
        Ok(Some((func_unique_id, inputs)))
    }

    async fn export_func(&self, ctx: &DalContext, func: &Func) -> PkgResult<(FuncSpec, bool)> {
        let mut func_spec_builder = FuncSpec::builder();

        func_spec_builder.name(func.name.clone());
        func_spec_builder.unique_id(func.id);

        let mut data_builder = FuncSpecData::builder();

        data_builder.name(func.name.clone());

        if let Some(display_name) = &func.display_name {
            data_builder.display_name(display_name);
        }

        if let Some(description) = &func.description {
            data_builder.description(description);
        }

        if let Some(link) = &func.link {
            data_builder.try_link(link.deref())?;
        }
        // Should we package an empty func?
        data_builder.handler(func.handler.clone().unwrap_or("".to_string()));
        data_builder.code_base64(func.code_base64.clone().unwrap_or("".to_string()));

        data_builder.response_type(func.backend_response_type);
        data_builder.backend_kind(func.backend_kind);
        data_builder.is_transformation(func.is_transformation);
        data_builder.last_updated_at(Some(func.timestamp.updated_at));

        data_builder.hidden(func.hidden);

        func_spec_builder.data(data_builder.build()?);
        func_spec_builder.unique_id(func.id.to_string());
        func_spec_builder.is_from_builtin(Some(func.builtin));

        let args: Vec<FuncArgument> = FuncArgument::list_for_func(ctx, func.id).await?;

        for arg in &args {
            let mut arg_builder = FuncArgumentSpec::builder();
            arg_builder.unique_id(arg.id.to_string());

            func_spec_builder.argument(
                arg_builder
                    .name(&arg.name)
                    .kind(arg.kind)
                    .element_kind(arg.element_kind.map(|kind| kind.into()))
                    .build()?,
            );
        }

        let func_spec = func_spec_builder.build()?;
        // If we have data, or change set specific arguments, we're valid for this changeset
        let include_in_export = func_spec.data.is_some() || !args.is_empty();

        Ok((func_spec, include_in_export))
    }

    async fn add_func_to_map(
        &mut self,
        ctx: &DalContext,
        func: &Func,
    ) -> PkgResult<(FuncSpec, bool)> {
        let (spec, include) = match IntrinsicFunc::maybe_from_str(&func.name) {
            // NOTE(nick): we need to retain the behavior of how we export the old, now intrinsic
            // funcs unless we perform a graph migration.
            Some(IntrinsicFunc::ResourcePayloadToValue) | Some(IntrinsicFunc::NormalizeToArray)
                if func.backend_kind == FuncBackendKind::JsAttribute =>
            {
                self.export_func(ctx, func).await?
            }
            Some(intrinsic) => {
                let spec = intrinsic.to_spec()?;

                (spec, true)
            }
            None => self.export_func(ctx, func).await?,
        };

        self.func_map.insert(func.id, spec.clone());

        Ok((spec, include))
    }

    pub fn func_spec_map(&self) -> &FuncSpecMap {
        &self.func_map
    }

    /// If change_set_id is None, we export everything in the changeset without checking for
    /// differences from HEAD. Otherwise we attempt to only export the data specific to the
    /// requested change_set
    async fn export_change_set(
        &mut self,
        ctx: &DalContext,
    ) -> PkgResult<(
        Vec<FuncSpec>,
        Vec<FuncSpec>,
        Vec<SchemaSpec>,
        Vec<ComponentSpec>,
        Vec<EdgeSpec>,
    )> {
        let mut func_specs = vec![];
        let head_funcs = vec![];
        let mut schema_specs = vec![];
        let component_specs = vec![];
        let edge_specs = vec![];

        let new_ctx = ctx.clone();
        let ctx = &new_ctx;

        for intrinsic in IntrinsicFunc::iter() {
            match intrinsic {
                // NOTE(nick): we only want to export intrinsics that were not special case funcs. If
                // we want that to change, we may need a graph migration.
                IntrinsicFunc::ResourcePayloadToValue | IntrinsicFunc::NormalizeToArray => {
                    let intrinsic_name = intrinsic.name();

                    // Notice that we only want to find the func with the intrinsic kind (the new
                    // version). If we cannot, we just skip it because it is not guaranteed to be
                    // here without a graph migration.
                    if let Some(intrinsic_func_id) =
                        Func::find_id_by_name_and_kind(ctx, intrinsic_name, FuncKind::Intrinsic)
                            .await?
                    {
                        let intrinsic_func = Func::get_by_id(ctx, intrinsic_func_id).await?;

                        let (spec, _) = self.add_func_to_map(ctx, &intrinsic_func).await?;

                        func_specs.push(spec);
                    }
                }
                IntrinsicFunc::SetFloat => {
                    // We need to check that an intrinsic function with the name "si:setFloat" exists.
                    // If it does not exist, we import it from the intrinsic package.
                    let maybe_intrinsic_func_id =
                        Func::find_id_by_name(ctx, intrinsic.name()).await?;
                    if let Some(intrinsic_func_id) = maybe_intrinsic_func_id {
                        let intrinsic_func = Func::get_by_id(ctx, intrinsic_func_id).await?;
                        let (spec, _) = self.add_func_to_map(ctx, &intrinsic_func).await?;
                        func_specs.push(spec);
                    } else {
                        import_pkg_from_pkg(
                            ctx,
                            &SiPkg::load_from_spec(IntrinsicFunc::float_pkg_spec()?)?,
                            None,
                        )
                        .await?;

                        let intrinsic_func_id = Func::find_id_by_name(ctx, "si:setFloat")
                            .await?
                            .ok_or(PkgError::MissingIntrinsicFunc("si:setFloat".to_string()))?;

                        let intrinsic_func = Func::get_by_id(ctx, intrinsic_func_id).await?;
                        let (spec, _) = self.add_func_to_map(ctx, &intrinsic_func).await?;
                        func_specs.push(spec);
                    }
                }
                _ => {
                    let intrinsic_name = intrinsic.name();
                    // We need a unique id for intrinsic funcs to refer to them in custom bindings (for example
                    // mapping one prop to another via si:identity)
                    let intrinsic_func_id = Func::find_id_by_name(ctx, intrinsic_name)
                        .await?
                        .ok_or(PkgError::MissingIntrinsicFunc(intrinsic_name.to_string()))?;

                    let intrinsic_func = Func::get_by_id(ctx, intrinsic_func_id).await?;

                    let (spec, _) = self.add_func_to_map(ctx, &intrinsic_func).await?;

                    func_specs.push(spec);
                }
            }
        }

        // Add all transformation funcs to package
        if self.include_transformations {
            for func in Func::list_all(ctx).await? {
                if func.is_transformation && func.kind == FuncKind::Attribute {
                    let (spec, _) = self.add_func_to_map(ctx, &func).await?;
                    func_specs.push(spec);
                }
            }
        }

        let mut schemas = vec![];
        for schema in Schema::list(ctx).await? {
            if self
                .schema_ids
                .as_ref()
                .map(|schema_ids| schema_ids.contains(&schema.id()))
                .unwrap_or(true)
            {
                schemas.push(schema)
            }
        }

        for schema in &schemas {
            let (schema_spec, funcs) = self.export_schema(ctx, schema).await?;

            func_specs.extend_from_slice(&funcs);
            schema_specs.push(schema_spec);
        }

        Ok((
            func_specs,
            head_funcs,
            schema_specs,
            component_specs,
            edge_specs,
        ))
    }

    pub async fn export_as_spec(&mut self, ctx: &DalContext) -> PkgResult<PkgSpec> {
        let mut pkg_spec_builder = PkgSpec::builder();
        pkg_spec_builder
            .name(&self.name)
            .kind(self.kind)
            .version(&self.version)
            .created_by(&self.created_by);

        if let Some(workspace_pk) = ctx.tenancy().workspace_pk_opt() {
            pkg_spec_builder.workspace_pk(workspace_pk.to_string());
            let workspace = Workspace::get_by_pk(ctx, workspace_pk).await?;
            pkg_spec_builder.workspace_name(workspace.name());
        }

        if let Some(description) = &self.description {
            pkg_spec_builder.description(description);
        }

        match self.kind {
            SiPkgKind::Module => {
                let (funcs, _, schemas, _, _) = self.export_change_set(ctx).await?;
                pkg_spec_builder.funcs(funcs);
                pkg_spec_builder.schemas(schemas);
            }
            SiPkgKind::WorkspaceBackup => return Err(PkgError::WorkspaceExportNotSupported()),
        }

        Ok(pkg_spec_builder.build()?)
    }

    pub async fn export(&mut self, ctx: &DalContext) -> PkgResult<SiPkg> {
        let spec = self.export_as_spec(ctx).await?;
        let pkg = SiPkg::load_from_spec(spec)?;

        Ok(pkg)
    }

    async fn export_intrinsics(&mut self, ctx: &DalContext) -> PkgResult<Vec<FuncSpec>> {
        let mut funcs = vec![];
        for intrinsic in IntrinsicFunc::iter() {
            // NOTE(nick): we only want to export intrinsics that were not special case funcs. If
            // we want that to change, we may need a graph migration.
            match intrinsic {
                IntrinsicFunc::ResourcePayloadToValue | IntrinsicFunc::NormalizeToArray => {
                    // Notice that we only want to find the func with the intrinsic kind (the new
                    // version). If we cannot, we just skip it because it is not guaranteed to be
                    // here without a graph migration.
                    if let Some(intrinsic_func_id) =
                        Func::find_id_by_name_and_kind(ctx, intrinsic.name(), FuncKind::Intrinsic)
                            .await?
                    {
                        let spec = intrinsic.to_spec()?;
                        funcs.push(spec.clone());
                        self.func_map.insert(intrinsic_func_id, spec.clone());
                    }
                }
                IntrinsicFunc::SetFloat => {
                    // We need to check that an intrinsic function with the name "si:setFloat" exists.
                    // If it does not exist, we import it from the intrinsic package.
                    let maybe_intrinsic_func_id =
                        Func::find_id_by_name(ctx, intrinsic.name()).await?;
                    if let Some(intrinsic_func_id) = maybe_intrinsic_func_id {
                        let spec = intrinsic.to_spec()?;
                        funcs.push(spec.clone());
                        self.func_map.insert(intrinsic_func_id, spec.clone());
                    } else {
                        import_pkg_from_pkg(
                            ctx,
                            &SiPkg::load_from_spec(IntrinsicFunc::float_pkg_spec()?)?,
                            None,
                        )
                        .await?;

                        let intrinsic_func_id = Func::find_id_by_name(ctx, "si:setFloat")
                            .await?
                            .ok_or(PkgError::MissingIntrinsicFunc("si:setFloat".to_string()))?;

                        let spec = intrinsic.to_spec()?;
                        funcs.push(spec.clone());
                        self.func_map.insert(intrinsic_func_id, spec.clone());
                    }
                }
                _ => {
                    let intrinsic_func_id = Func::find_id_by_name(ctx, intrinsic.name())
                        .await?
                        .ok_or(PkgError::MissingIntrinsicFunc(intrinsic.name().to_string()))?;

                    let spec = intrinsic.to_spec()?;
                    funcs.push(spec.clone());
                    self.func_map.insert(intrinsic_func_id, spec.clone());
                }
            }
        }
        Ok(funcs)
    }

    async fn export_funcs_for_variant(
        &mut self,
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
        overridden_asset_func_id: Option<FuncId>,
    ) -> PkgResult<Vec<FuncSpec>> {
        let related_funcs =
            SchemaVariant::all_funcs_without_intrinsics(ctx, schema_variant_id).await?;
        let mut funcs = vec![];

        for func in &related_funcs {
            let (func_spec, include) = self.add_func_to_map(ctx, func).await?;

            if include {
                funcs.push(func_spec);
            }
        }

        let variant = SchemaVariant::get_by_id(ctx, schema_variant_id).await?;

        // Asset Funcs are not stored in the FuncMap
        // So we need to look it up directly then store it!
        if let Some(asset_func_id) = overridden_asset_func_id.or_else(|| variant.asset_func_id()) {
            let asset_func = Func::get_by_id(ctx, asset_func_id).await?;
            let (func_spec, include) = self.add_func_to_map(ctx, &asset_func).await?;

            if include {
                funcs.push(func_spec);
            }
        }

        Ok(funcs)
    }
}
