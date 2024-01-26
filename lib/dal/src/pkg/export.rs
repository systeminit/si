use std::collections::{hash_map::Entry, HashMap, HashSet};

use strum::IntoEnumIterator;

use si_pkg::{
    ActionFuncSpec, AttrFuncInputSpec, AttrFuncInputSpecKind, AttributeValuePath,
    AttributeValueSpec, AuthenticationFuncSpec, ChangeSetSpec, ComponentSpec, ComponentSpecVariant,
    EdgeSpec, EdgeSpecKind, FuncArgumentSpec, FuncSpec, FuncSpecData, LeafFunctionSpec,
    MapKeyFuncSpec, PkgSpec, PositionSpec, PropSpec, PropSpecBuilder, PropSpecKind,
    RootPropFuncSpec, SchemaSpec, SchemaSpecData, SchemaVariantSpec, SchemaVariantSpecBuilder,
    SchemaVariantSpecComponentType, SchemaVariantSpecData, SchemaVariantSpecPropRoot, SiPkg,
    SiPkgKind, SiPropFuncSpec, SiPropFuncSpecKind, SocketSpec, SocketSpecData, SocketSpecKind,
    SpecError,
};
use telemetry::prelude::*;

use crate::authentication_prototype::{AuthenticationPrototype, AuthenticationPrototypeContext};
use crate::{
    component::view::{AttributeDebugView, ComponentDebugView},
    edge::EdgeKind,
    func::{argument::FuncArgument, intrinsics::IntrinsicFunc},
    prop::PropPath,
    prop_tree::{PropTree, PropTreeNode},
    schema::variant::definition::SchemaVariantDefinition,
    socket::SocketKind,
    ActionPrototype, ActionPrototypeContext, AttributeContextBuilder, AttributePrototype,
    AttributePrototypeArgument, AttributeReadContext, AttributeValue, ChangeSet, ChangeSetPk,
    Component, ComponentError, ComponentId, ComponentType, DalContext, Edge, EdgeError,
    ExternalProvider, ExternalProviderId, Func, FuncError, FuncId, InternalProvider,
    InternalProviderId, LeafInputLocation, LeafKind, Prop, PropError, PropId, PropKind, Schema,
    SchemaId, SchemaVariant, SchemaVariantError, SchemaVariantId, Socket, StandardModel, Workspace,
};

use super::{PkgError, PkgResult};

type FuncSpecMap = super::ChangeSetThingMap<FuncId, FuncSpec>;
type VariantSpecMap = super::ChangeSetThingMap<SchemaVariantId, SchemaVariantSpec>;
type ComponentMap = super::ChangeSetThingMap<ComponentId, ComponentSpec>;

pub struct PkgExporter {
    name: String,
    version: String,
    description: Option<String>,
    kind: SiPkgKind,
    created_by: String,
    schema_ids: Option<Vec<SchemaId>>,
    func_map: FuncSpecMap,
    variant_map: VariantSpecMap,
    component_map: ComponentMap,
    is_workspace_export: bool,
    include_components: bool,
}

fn change_set_matches(
    current_change_set_pk: Option<ChangeSetPk>,
    object_change_set_pk: ChangeSetPk,
) -> bool {
    match current_change_set_pk {
        None => true,
        Some(current_change_set_pk) => object_change_set_pk == current_change_set_pk,
    }
}

impl PkgExporter {
    pub fn new_module_exporter(
        name: impl Into<String>,
        version: impl Into<String>,
        description: Option<impl Into<String>>,
        created_by: impl Into<String>,
        schema_ids: Vec<SchemaId>,
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
            component_map: ComponentMap::new(),
            is_workspace_export: false,
            include_components: false,
        }
    }

    pub fn new_workspace_exporter(
        name: impl Into<String>,
        created_by: impl Into<String>,
        version: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            description: Some(description.into()),
            kind: SiPkgKind::WorkspaceBackup,
            created_by: created_by.into(),
            schema_ids: None,
            func_map: FuncSpecMap::new(),
            variant_map: VariantSpecMap::new(),
            component_map: ComponentMap::new(),
            is_workspace_export: true,
            include_components: true,
        }
    }

    pub async fn export_as_bytes(&mut self, ctx: &DalContext) -> PkgResult<Vec<u8>> {
        match self.kind {
            SiPkgKind::Module => info!("Building module package"),
            SiPkgKind::WorkspaceBackup => info!("Building workspace backup package"),
        }

        let pkg = self.export(ctx).await?;

        info!("Exporting as bytes");

        Ok(pkg.write_to_bytes()?)
    }

    async fn export_schema(
        &mut self,
        ctx: &DalContext,
        change_set_pk: Option<ChangeSetPk>,
        schema: &Schema,
    ) -> PkgResult<(SchemaSpec, Vec<FuncSpec>, Vec<FuncSpec>)> {
        let variants = schema.variants(ctx).await?;
        let mut funcs = vec![];
        let mut head_funcs = vec![];

        let mut schema_spec_builder = SchemaSpec::builder();
        schema_spec_builder.name(schema.name());
        schema_spec_builder.unique_id(schema.id().to_string());

        let is_deleted = schema.visibility().is_deleted();

        let default_variant_id = schema.default_schema_variant_id().copied();
        let mut default_variant_unique_id = None;

        for variant in &variants {
            let related_funcs = SchemaVariant::all_funcs(ctx, *variant.id()).await?;

            for func in &related_funcs {
                if change_set_pk.is_some()
                    && change_set_pk.as_ref().expect("some is ensured") != &ChangeSetPk::NONE
                    && self.func_map.get(ChangeSetPk::NONE, func.id()).is_none()
                    && func.visibility().change_set_pk == ChangeSetPk::NONE
                {
                    let (func_spec, _) =
                        self.export_func(ctx, Some(ChangeSetPk::NONE), func).await?;
                    self.func_map
                        .insert(ChangeSetPk::NONE, *func.id(), func_spec.to_owned());
                    head_funcs.push(func_spec);
                } else {
                    let (func_spec, include) = self.export_func(ctx, change_set_pk, func).await?;
                    self.func_map.insert(
                        change_set_pk.unwrap_or(ChangeSetPk::NONE),
                        *func.id(),
                        func_spec.to_owned(),
                    );

                    if include {
                        funcs.push(func_spec);
                    }
                }
            }

            let variant_spec = self.export_variant(ctx, change_set_pk, variant).await?;
            self.variant_map.insert(
                change_set_pk.unwrap_or(ChangeSetPk::NONE),
                *variant.id(),
                variant_spec.to_owned(),
            );
            if variant_spec.unique_id.is_some() {
                if let Some(default_variant_id) = default_variant_id {
                    if variant.id() == &default_variant_id {
                        default_variant_unique_id = variant_spec.unique_id.to_owned();
                    }
                }
            }
            schema_spec_builder.variant(variant_spec);
        }

        if is_deleted {
            schema_spec_builder.deleted(true);
        } else {
            let mut data_builder = SchemaSpecData::builder();
            data_builder.name(schema.name());
            data_builder.ui_hidden(schema.ui_hidden());
            let schema_ui_menu = schema.ui_menus(ctx).await?.pop().ok_or_else(|| {
                PkgError::StandardModelMissingBelongsTo(
                    "schema_ui_menu_belongs_to_schema",
                    "schema",
                    (*schema.id()).to_string(),
                )
            })?;
            data_builder.category(schema_ui_menu.category());
            data_builder.category_name(schema_ui_menu.name());
            if let Some(default_unique_id) = default_variant_unique_id {
                data_builder.default_schema_variant(default_unique_id);
            }
            schema_spec_builder.data(data_builder.build()?);
        }

        let schema_spec = schema_spec_builder.build()?;

        Ok((schema_spec, funcs, head_funcs))
    }

    pub async fn export_variant(
        &self,
        ctx: &DalContext,
        change_set_pk: Option<ChangeSetPk>,
        variant: &SchemaVariant,
    ) -> PkgResult<SchemaVariantSpec> {
        let mut variant_spec_builder = SchemaVariantSpec::builder();
        variant_spec_builder.name(variant.name());

        let schema_variant_definition =
            SchemaVariantDefinition::get_by_schema_variant_id(ctx, variant.id())
                .await?
                .ok_or(PkgError::MissingSchemaVariantDefinition(*variant.id()))?;

        variant_spec_builder.unique_id(variant.id().to_string());

        if variant.visibility().is_deleted() {
            variant_spec_builder.deleted(true);
        } else {
            let mut data_builder = SchemaVariantSpecData::builder();

            data_builder.name(variant.name());

            if let Some(color_str) = variant.color(ctx).await? {
                data_builder.color(color_str);
            };
            if let Some(link) = variant.link() {
                data_builder.try_link(link)?;
            }

            data_builder.component_type(get_component_type(ctx, variant).await?);

            let asset_func_unique_id = self
                .func_map
                .get(
                    change_set_pk.unwrap_or(ChangeSetPk::NONE),
                    &schema_variant_definition.func_id(),
                )
                .ok_or(PkgError::MissingExportedFunc(
                    schema_variant_definition.func_id(),
                ))?
                .unique_id
                .to_owned();

            data_builder.func_unique_id(asset_func_unique_id);

            variant_spec_builder.data(data_builder.build()?);
        }

        self.export_prop_tree(
            ctx,
            change_set_pk,
            variant,
            &mut variant_spec_builder,
            SchemaVariantSpecPropRoot::Domain,
        )
        .await?;

        self.export_prop_tree(
            ctx,
            change_set_pk,
            variant,
            &mut variant_spec_builder,
            SchemaVariantSpecPropRoot::ResourceValue,
        )
        .await?;

        self.export_prop_tree(
            ctx,
            change_set_pk,
            variant,
            &mut variant_spec_builder,
            SchemaVariantSpecPropRoot::Secrets,
        )
        .await?;

        self.export_prop_tree(
            ctx,
            change_set_pk,
            variant,
            &mut variant_spec_builder,
            SchemaVariantSpecPropRoot::SecretDefinition,
        )
        .await?;

        self.export_leaf_funcs(ctx, change_set_pk, *variant.id())
            .await?
            .drain(..)
            .for_each(|leaf_func_spec| {
                variant_spec_builder.leaf_function(leaf_func_spec);
            });

        self.export_sockets(ctx, change_set_pk, *variant.id())
            .await?
            .drain(..)
            .for_each(|socket_spec| {
                variant_spec_builder.socket(socket_spec);
            });

        self.export_action_funcs(ctx, change_set_pk, *variant.id())
            .await?
            .drain(..)
            .for_each(|action_func_spec| {
                variant_spec_builder.action_func(action_func_spec);
            });

        self.export_auth_funcs(ctx, change_set_pk, *variant.id())
            .await?
            .drain(..)
            .for_each(|spec| {
                variant_spec_builder.auth_func(spec);
            });

        self.export_si_prop_funcs(ctx, change_set_pk, variant)
            .await?
            .drain(..)
            .for_each(|si_prop_func_spec| {
                variant_spec_builder.si_prop_func(si_prop_func_spec);
            });

        self.export_root_prop_funcs(ctx, change_set_pk, variant)
            .await?
            .drain(..)
            .for_each(|root_prop_func_spec| {
                variant_spec_builder.root_prop_func(root_prop_func_spec);
            });

        let variant_spec = variant_spec_builder.build()?;

        Ok(variant_spec)
    }

    pub async fn export_root_prop_funcs(
        &self,
        ctx: &DalContext,
        change_set_pk: Option<ChangeSetPk>,
        variant: &SchemaVariant,
    ) -> PkgResult<Vec<RootPropFuncSpec>> {
        let mut specs = vec![];

        for root_prop in SchemaVariantSpecPropRoot::iter() {
            if let Some(prop) = Prop::find_prop_by_path_opt(
                ctx,
                *variant.id(),
                &PropPath::new(root_prop.path_parts()),
            )
            .await?
            {
                let context = AttributeContextBuilder::new()
                    .set_prop_id(*prop.id())
                    .to_context()?;

                if let Some(prototype) =
                    AttributePrototype::find_for_context_and_key(ctx, context, &None)
                        .await?
                        .pop()
                {
                    if let Some((func_unique_id, mut inputs)) = self
                        .export_input_func_and_arguments(ctx, change_set_pk, &prototype)
                        .await?
                    {
                        let mut builder = RootPropFuncSpec::builder();
                        builder
                            .deleted(prototype.visibility().is_deleted())
                            .func_unique_id(func_unique_id)
                            .unique_id(prototype.id().to_string())
                            .prop(root_prop);

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

    pub async fn export_si_prop_funcs(
        &self,
        ctx: &DalContext,
        change_set_pk: Option<ChangeSetPk>,
        variant: &SchemaVariant,
    ) -> PkgResult<Vec<SiPropFuncSpec>> {
        let mut specs = vec![];

        for kind in SiPropFuncSpecKind::iter() {
            let prop = variant.find_prop(ctx, &kind.prop_path()).await?;

            let context = AttributeContextBuilder::new()
                .set_prop_id(*prop.id())
                .to_context()?;

            if let Some(prototype) =
                AttributePrototype::find_for_context_and_key(ctx, context, &None)
                    .await?
                    .pop()
            {
                if let Some((func_unique_id, mut inputs)) = self
                    .export_input_func_and_arguments(ctx, change_set_pk, &prototype)
                    .await?
                {
                    let mut builder = SiPropFuncSpec::builder();
                    builder
                        .deleted(prototype.visibility().is_deleted())
                        .func_unique_id(func_unique_id)
                        .unique_id(prototype.id().to_string())
                        .kind(kind);

                    inputs.drain(..).for_each(|input| {
                        builder.input(input);
                    });

                    specs.push(builder.build()?);
                }
            }
        }

        Ok(specs)
    }

    pub async fn export_leaf_funcs(
        &self,
        ctx: &DalContext,
        change_set_pk: Option<ChangeSetPk>,
        variant_id: SchemaVariantId,
    ) -> PkgResult<Vec<LeafFunctionSpec>> {
        let mut specs = vec![];

        for leaf_kind in LeafKind::iter() {
            for (prototype, leaf_func) in
                SchemaVariant::find_leaf_item_functions(ctx, variant_id, leaf_kind).await?
            {
                let func_unique_id = self
                    .func_map
                    .get(change_set_pk.unwrap_or(ChangeSetPk::NONE), leaf_func.id())
                    .map_or_else(
                        || leaf_func.id().to_string(),
                        |spec| spec.unique_id.to_owned(),
                    );

                let mut inputs = vec![];
                for arg in FuncArgument::list_for_func(ctx, *leaf_func.id()).await? {
                    if arg.visibility().is_deleted() {
                        continue;
                    }

                    inputs.push(
                        LeafInputLocation::maybe_from_arg_name(arg.name())
                            .ok_or(SpecError::LeafInputLocationConversionError(
                                arg.name().into(),
                            ))?
                            .into(),
                    );
                }

                let mut builder = LeafFunctionSpec::builder();
                builder.unique_id(prototype.id().to_string());

                specs.push(
                    builder
                        .func_unique_id(&func_unique_id)
                        .leaf_kind(leaf_kind)
                        .inputs(inputs)
                        .deleted(prototype.visibility().is_deleted())
                        .build()?,
                );
            }
        }

        Ok(specs)
    }

    pub async fn export_sockets(
        &self,
        ctx: &DalContext,
        change_set_pk: Option<ChangeSetPk>,
        variant_id: SchemaVariantId,
    ) -> PkgResult<Vec<SocketSpec>> {
        let mut specs = vec![];

        for input_socket_ip in
            InternalProvider::list_explicit_for_schema_variant(ctx, variant_id).await?
        {
            let socket = Socket::find_for_internal_provider(ctx, *input_socket_ip.id())
                .await?
                .pop()
                .ok_or(PkgError::ExplicitInternalProviderMissingSocket(
                    *input_socket_ip.id(),
                ))?;

            if let SocketKind::Frame = socket.kind() {
                continue;
            }

            let mut socket_spec_builder = SocketSpec::builder();
            socket_spec_builder.name(input_socket_ip.name());
            socket_spec_builder.unique_id(input_socket_ip.id().to_string());

            let mut data_builder = SocketSpecData::builder();

            data_builder
                .name(input_socket_ip.name())
                .connection_annotations(socket.connection_annotations())
                .kind(SocketSpecKind::Input)
                .arity(socket.arity())
                .ui_hidden(socket.ui_hidden());

            // let mut has_custom_func = false;
            if let Some(attr_proto_id) = input_socket_ip.attribute_prototype_id() {
                let proto = AttributePrototype::get_by_id(ctx, attr_proto_id)
                    .await?
                    .ok_or(PkgError::MissingAttributePrototypeForInputSocket(
                        *attr_proto_id,
                        *input_socket_ip.id(),
                    ))?;

                if let Some((func_unique_id, mut inputs)) = self
                    .export_input_func_and_arguments(ctx, change_set_pk, &proto)
                    .await?
                {
                    // has_custom_func = true;
                    data_builder.func_unique_id(func_unique_id);
                    inputs.drain(..).for_each(|input| {
                        socket_spec_builder.input(input);
                    });
                }
            }

            // if has_custom_func {
            socket_spec_builder.data(data_builder.build()?);
            // }

            specs.push(socket_spec_builder.build()?);
        }

        for output_socket_ep in ExternalProvider::list_for_schema_variant(ctx, variant_id).await? {
            let socket = Socket::find_for_external_provider(ctx, *output_socket_ep.id())
                .await?
                .pop()
                .ok_or(PkgError::ExternalProviderMissingSocket(
                    *output_socket_ep.id(),
                ))?;

            if let SocketKind::Frame = socket.kind() {
                continue;
            }

            let mut socket_spec_builder = SocketSpec::builder();
            socket_spec_builder.name(output_socket_ep.name());

            socket_spec_builder.unique_id(output_socket_ep.id().to_string());

            let mut data_builder = SocketSpecData::builder();
            data_builder
                .name(output_socket_ep.name())
                .connection_annotations(socket.connection_annotations())
                .kind(SocketSpecKind::Output)
                .arity(socket.arity())
                .ui_hidden(socket.ui_hidden());

            // let mut has_custom_func = false;
            if let Some(attr_proto_id) = output_socket_ep.attribute_prototype_id() {
                let proto = AttributePrototype::get_by_id(ctx, attr_proto_id)
                    .await?
                    .ok_or(PkgError::MissingAttributePrototypeForOutputSocket(
                        *attr_proto_id,
                        *output_socket_ep.id(),
                    ))?;

                if let Some((func_unique_id, mut inputs)) = self
                    .export_input_func_and_arguments(ctx, change_set_pk, &proto)
                    .await?
                {
                    // has_custom_func = true;
                    data_builder.func_unique_id(func_unique_id);
                    inputs.drain(..).for_each(|input| {
                        socket_spec_builder.input(input);
                    });
                }
            }

            // if has_custom_func {
            socket_spec_builder.data(data_builder.build()?);
            // }

            specs.push(socket_spec_builder.build()?);
        }

        Ok(specs)
    }

    pub async fn export_action_funcs(
        &self,
        ctx: &DalContext,
        change_set_pk: Option<ChangeSetPk>,
        schema_variant_id: SchemaVariantId,
    ) -> PkgResult<Vec<ActionFuncSpec>> {
        let mut specs = vec![];

        let action_prototypes =
            ActionPrototype::find_for_context(ctx, ActionPrototypeContext { schema_variant_id })
                .await?;

        for action_proto in action_prototypes {
            let func_unique_id = self
                .func_map
                .get(
                    change_set_pk.unwrap_or(ChangeSetPk::NONE),
                    &action_proto.func_id(),
                )
                .map_or_else(
                    || action_proto.func_id().to_string(),
                    |spec| spec.unique_id.to_owned(),
                );

            let mut builder = ActionFuncSpec::builder();

            builder.unique_id(action_proto.id().to_string());

            specs.push(
                builder
                    .kind(action_proto.kind())
                    .func_unique_id(&func_unique_id)
                    .deleted(action_proto.visibility().is_deleted())
                    .build()?,
            )
        }

        Ok(specs)
    }

    pub async fn export_auth_funcs(
        &self,
        ctx: &DalContext,
        change_set_pk: Option<ChangeSetPk>,
        schema_variant_id: SchemaVariantId,
    ) -> PkgResult<Vec<AuthenticationFuncSpec>> {
        let mut specs = vec![];

        let prototypes = AuthenticationPrototype::find_for_context(
            ctx,
            AuthenticationPrototypeContext { schema_variant_id },
        )
        .await?;

        for prototype in prototypes {
            let func_unique_id = self
                .func_map
                .get(
                    change_set_pk.unwrap_or(ChangeSetPk::NONE),
                    &prototype.func_id(),
                )
                .map_or_else(
                    || prototype.func_id().to_string(),
                    |spec| spec.unique_id.to_owned(),
                );

            let mut builder = AuthenticationFuncSpec::builder();

            builder.unique_id(prototype.id().to_string());

            specs.push(
                builder
                    .func_unique_id(&func_unique_id)
                    .deleted(prototype.visibility().is_deleted())
                    .build()?,
            )
        }

        Ok(specs)
    }

    async fn export_prop_tree(
        &self,
        ctx: &DalContext,
        change_set_pk: Option<ChangeSetPk>,
        variant: &SchemaVariant,
        variant_spec: &mut SchemaVariantSpecBuilder,
        prop_root: SchemaVariantSpecPropRoot,
    ) -> PkgResult<()> {
        let mut prop_tree = PropTree::new(ctx, true, Some(vec![*variant.id()]), None).await?;
        let root_tree_node = prop_tree
            .root_props
            .pop()
            .ok_or_else(|| PkgError::prop_tree_invalid("root prop not found"))?;
        if !prop_tree.root_props.is_empty() {
            return Err(PkgError::prop_tree_invalid(
                "prop tree contained multiple root props",
            ));
        }
        let prop_root_tree_node = match root_tree_node.children.into_iter().find(|tree_node| {
            match prop_root {
                SchemaVariantSpecPropRoot::Domain => {
                    tree_node.name == "domain" && tree_node.path == "/root/"
                }
                SchemaVariantSpecPropRoot::ResourceValue => {
                    tree_node.name == "resource_value" && tree_node.path == "/root/"
                }
                SchemaVariantSpecPropRoot::Secrets => {
                    tree_node.name == "secrets" && tree_node.path == "/root/"
                }
                SchemaVariantSpecPropRoot::SecretDefinition => {
                    tree_node.name == "secret_definition" && tree_node.path == "/root/"
                }
            }
        }) {
            Some(root_tree_node) => root_tree_node,
            None => {
                if matches!(prop_root, SchemaVariantSpecPropRoot::Domain) {
                    return Err(PkgError::prop_tree_invalid("domain prop not found"));
                } else {
                    warn!("/root/resource_value prop not found, if value prop PR has merged, this should be an error not a warning.");
                    return Ok(());
                }
            }
        };

        #[derive(Debug)]
        struct TraversalStackEntry {
            builder: PropSpecBuilder,
            prop_id: PropId,
            parent_prop_id: Option<PropId>,
            inside_map_or_array: bool,
        }

        let mut stack: Vec<(PropTreeNode, Option<PropId>, bool)> = Vec::new();
        for child_tree_node in prop_root_tree_node.children {
            stack.push((child_tree_node, None, false));
        }

        let mut traversal_stack: Vec<TraversalStackEntry> = Vec::new();

        while let Some((tree_node, parent_prop_id, inside_map_or_array)) = stack.pop() {
            let prop_id = tree_node.prop_id;
            let mut builder = PropSpec::builder();

            if !change_set_matches(change_set_pk, tree_node.visibility_change_set_pk) {
                builder.has_data(false);
            }

            builder.unique_id(prop_id);

            builder
                .name(tree_node.name)
                .kind(match tree_node.kind {
                    PropKind::Array => PropSpecKind::Array,
                    PropKind::Boolean => PropSpecKind::Boolean,
                    PropKind::Integer => PropSpecKind::Number,
                    PropKind::Object => PropSpecKind::Object,
                    PropKind::String => PropSpecKind::String,
                    PropKind::Map => PropSpecKind::Map,
                })
                .hidden(tree_node.hidden)
                .widget_kind(tree_node.widget_kind)
                .widget_options(tree_node.widget_options);

            if let Some(doc_link) = tree_node.doc_link {
                builder.try_doc_link(doc_link.as_str())?;
            }

            if let Some(documentation) = tree_node.documentation {
                builder.documentation(documentation.as_str());
            }

            traversal_stack.push(TraversalStackEntry {
                builder,
                prop_id,
                parent_prop_id,
                inside_map_or_array,
            });

            for child_tree_node in tree_node.children {
                stack.push((
                    child_tree_node,
                    Some(prop_id),
                    matches!(tree_node.kind, PropKind::Array | PropKind::Map)
                        || inside_map_or_array,
                ));
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
                                    PkgError::prop_spec_children_invalid(format!(
                                        "found no child for map/array for prop id {}",
                                        entry.prop_id,
                                    ))
                                })?;
                            if !prop_children.is_empty() {
                                return Err(PkgError::prop_spec_children_invalid(format!(
                                    "found multiple children for map/array for prop id {}",
                                    entry.prop_id,
                                )));
                            }
                            entry.builder.type_prop(type_prop);
                            maybe_type_prop_id = Some(type_prop_id);
                        }
                        PropSpecKind::String | PropSpecKind::Number | PropSpecKind::Boolean => {
                            return Err(PkgError::prop_spec_children_invalid(format!(
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
                    let context = AttributeContextBuilder::new()
                        .set_prop_id(type_prop_id)
                        .to_context()?;

                    for proto in AttributePrototype::list_for_context(ctx, context).await? {
                        if let Some(key) = proto.key() {
                            if let Some((func_unique_id, mut inputs)) = self
                                .export_input_func_and_arguments(ctx, change_set_pk, &proto)
                                .await?
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

            // TODO: if we get funcs here but we also got map_key_funcs above, that's a sign of a
            // TODO: misconfigured set of attribute prototypes. check and error
            let context = AttributeContextBuilder::new()
                .set_prop_id(entry.prop_id)
                .to_context()?;

            if let Some(prototype) =
                AttributePrototype::find_for_context_and_key(ctx, context, &None)
                    .await?
                    .pop()
            {
                if let Some((func_unique_id, mut inputs)) = self
                    .export_input_func_and_arguments(ctx, change_set_pk, &prototype)
                    .await?
                {
                    entry.builder.has_data(true);

                    entry.builder.func_unique_id(func_unique_id);
                    inputs.drain(..).for_each(|input| {
                        entry.builder.input(input);
                    });
                }
            }

            // TODO: handle default values for complex types. We also cannot set default values for
            // children of arrays and maps, at any depth (currently), since that requires tracking the
            // key or index
            if matches!(
                entry.builder.get_kind(),
                Some(PropSpecKind::String)
                    | Some(PropSpecKind::Number)
                    | Some(PropSpecKind::Boolean)
            ) && !entry.inside_map_or_array
            {
                if let Some(av) = AttributeValue::find_for_context(ctx, context.into()).await? {
                    if let Some(default_value) = av.get_value(ctx).await? {
                        entry.builder.has_data(true);
                        entry.builder.default_value(default_value);
                    }
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
        change_set_pk: Option<ChangeSetPk>,
        proto: &AttributePrototype,
    ) -> PkgResult<Option<(String, Vec<AttrFuncInputSpec>)>> {
        let proto_func = Func::get_by_id(ctx, &proto.func_id()).await?.ok_or(
            PkgError::MissingAttributePrototypeFunc(*proto.id(), proto.func_id()),
        )?;

        let apas: Vec<AttributePrototypeArgument> =
            AttributePrototypeArgument::list_for_attribute_prototype(ctx, *proto.id()).await?;

        // If the prototype func is intrinsic and has no arguments, it's one that is created by default
        // and we don't have to track it in the package
        if apas.is_empty() && proto_func.is_intrinsic() {
            return Ok(None);
        }

        let mut inputs = vec![];

        for apa in &apas {
            let func_arg = FuncArgument::get_by_id(ctx, &apa.func_argument_id())
                .await?
                .ok_or(PkgError::AttributePrototypeArgumentMissingFuncArgument(
                    *apa.id(),
                    apa.func_argument_id(),
                ))?;
            let arg_name = func_arg.name();

            let mut builder = AttrFuncInputSpec::builder();
            builder
                .unique_id(apa.id().to_string())
                .name(arg_name)
                .deleted(apa.visibility().is_deleted());

            if apa.internal_provider_id() != InternalProviderId::NONE {
                let ip = InternalProvider::get_by_id(ctx, &apa.internal_provider_id())
                    .await?
                    .ok_or(PkgError::AttributePrototypeArgumentMissingInternalProvider(
                        *apa.id(),
                        apa.internal_provider_id(),
                    ))?;

                match *ip.prop_id() {
                    PropId::NONE => {
                        inputs.push(
                            builder
                                .name(arg_name)
                                .kind(AttrFuncInputSpecKind::InputSocket)
                                .socket_name(ip.name())
                                .build()?,
                        );
                    }
                    prop_id => {
                        let prop = Prop::get_by_id(ctx, &prop_id)
                            .await?
                            .ok_or(PkgError::InternalProviderMissingProp(*ip.id(), prop_id))?;

                        inputs.push(
                            builder
                                .kind(AttrFuncInputSpecKind::Prop)
                                .prop_path(prop.path())
                                .build()?,
                        );
                    }
                }
            } else if apa.external_provider_id() != ExternalProviderId::NONE {
                // We don't want to create these on import of schema variants, so we don't care if
                // we find it or not. But we do need to ensure the input length is correct for when
                // we do this on *component import*, so that we don't modify the inputs to the
                // attribute function on the component.
                let socket_name =
                    match ExternalProvider::get_by_id(ctx, &apa.external_provider_id()).await? {
                        None => "__si-dummy-output-socket__".to_owned(),
                        Some(ep) => ep.name().to_owned(),
                    };

                inputs.push(
                    builder
                        .kind(AttrFuncInputSpecKind::OutputSocket)
                        .socket_name(socket_name)
                        .build()?,
                );
            }
        }

        let func_unique_id = self
            .func_map
            .get(change_set_pk.unwrap_or(ChangeSetPk::NONE), proto_func.id())
            .map_or_else(
                || proto_func.id().to_string(),
                |spec| spec.unique_id.to_owned(),
            );

        Ok(Some((func_unique_id, inputs)))
    }

    pub async fn export_func(
        &mut self,
        ctx: &DalContext,
        change_set_pk: Option<ChangeSetPk>,
        func: &Func,
    ) -> PkgResult<(FuncSpec, bool)> {
        if self.is_workspace_export && func.is_intrinsic() {
            if let Some(intrinsic) = IntrinsicFunc::maybe_from_str(func.name()) {
                return Ok((intrinsic.to_spec()?, true));
            }
        }

        let mut func_spec_builder = FuncSpec::builder();

        func_spec_builder.name(func.name());

        func_spec_builder.unique_id(func.id().to_string());
        if func.visibility().is_deleted() {
            func_spec_builder.deleted(true);
            return Ok((func_spec_builder.build()?, true));
        }

        let mut data_builder = FuncSpecData::builder();

        data_builder.name(func.name());

        data_builder.display_name(func.display_name().map(ToOwned::to_owned));

        data_builder.description(func.description().map(ToOwned::to_owned));

        if let Some(link) = func.link() {
            data_builder.try_link(link)?;
        }
        // Should we package an empty func?
        data_builder.handler(func.handler().unwrap_or(""));
        data_builder.code_base64(func.code_base64().unwrap_or(""));

        data_builder.response_type(*func.backend_response_type());
        data_builder.backend_kind(*func.backend_kind());

        data_builder.hidden(func.hidden());

        func_spec_builder.data(data_builder.build()?);

        func_spec_builder.unique_id(func.id().to_string());
        if self.is_workspace_export {
            func_spec_builder.is_from_builtin(Some(func.is_builtin(ctx).await?));
        }

        let args: Vec<FuncArgument> = FuncArgument::list_for_func(ctx, *func.id()).await?;

        for arg in &args {
            let mut arg_builder = FuncArgumentSpec::builder();

            arg_builder.unique_id(arg.id().to_string());

            func_spec_builder.argument(
                arg_builder
                    .name(arg.name())
                    .kind(*arg.kind())
                    .element_kind(arg.element_kind().cloned().map(|kind| kind.into()))
                    .deleted(arg.visibility().is_deleted())
                    .build()?,
            );
        }

        let func_spec = func_spec_builder.build()?;
        // If we have data, or change set specific arguments, we're valid for this changeset
        let include_in_export = func_spec.data.is_some() || !args.is_empty();

        self.func_map.insert(
            change_set_pk.unwrap_or(ChangeSetPk::NONE),
            *func.id(),
            func_spec.clone(),
        );

        Ok((func_spec, include_in_export))
    }

    /// If change_set_pk is None, we export everything in the changeset without checking for
    /// differences from HEAD. Otherwise we attempt to only export the data specific to the
    /// requested change_set
    async fn export_change_set(
        &mut self,
        ctx: &DalContext,
        change_set_pk: Option<ChangeSetPk>,
    ) -> PkgResult<(
        Vec<FuncSpec>,
        Vec<FuncSpec>,
        Vec<SchemaSpec>,
        Vec<ComponentSpec>,
        Vec<EdgeSpec>,
    )> {
        let mut func_specs = vec![];
        let mut head_funcs = vec![];
        let mut schema_specs = vec![];
        let mut component_specs = vec![];
        let mut edge_specs = vec![];

        let new_ctx = match change_set_pk {
            None => ctx.clone(),
            Some(change_set_pk) => {
                ctx.clone_with_new_visibility(ctx.visibility().to_change_set_deleted(change_set_pk))
            }
        };
        let ctx = &new_ctx;

        // Intrinsic funcs should be immutable. They're not, but we don't provide any interfaces to
        // modify them via a the standard model. We only add them to the func map if the func map
        // is HEAD (or if we're doing a module export)
        if change_set_pk.unwrap_or(ChangeSetPk::NONE) == ChangeSetPk::NONE {
            for intrinsic in crate::func::intrinsics::IntrinsicFunc::iter() {
                let intrinsic_name = intrinsic.name();
                // We need a unique id for intrinsic funcs to refer to them in custom bindings (for example
                // mapping one prop to another via si:identity)
                let intrinsic_func = Func::find_by_name(ctx, intrinsic_name)
                    .await?
                    .ok_or(PkgError::MissingIntrinsicFunc(intrinsic_name.to_string()))?;

                let intrinsic_spec = intrinsic.to_spec()?;
                self.func_map.insert(
                    change_set_pk.unwrap_or(ChangeSetPk::NONE),
                    *intrinsic_func.id(),
                    intrinsic_spec.clone(),
                );

                func_specs.push(intrinsic_spec);
            }
        }

        // XXX: make this SQL query
        let mut schemas = vec![];
        for schema in Schema::list(ctx).await? {
            let add_schema = if let Some(schema_ids) = &self.schema_ids {
                schema_ids.contains(schema.id())
            } else if self.is_workspace_export {
                !schema.is_builtin(ctx).await? || schema.visibility().deleted_at.is_some()
            } else {
                true
            };

            if add_schema {
                schemas.push(schema);
            }
        }

        for schema in &schemas {
            let (schema_spec, funcs, referenced_head_funcs) =
                self.export_schema(ctx, change_set_pk, schema).await?;

            head_funcs.extend_from_slice(&referenced_head_funcs);
            func_specs.extend_from_slice(&funcs);
            schema_specs.push(schema_spec);
        }

        if self.is_workspace_export && self.include_components {
            for component in Component::list(ctx).await? {
                let variant = component
                    .schema_variant(ctx)
                    .await?
                    .ok_or(ComponentError::NoSchemaVariant(*component.id()))?;

                let component_variant = match self
                    .variant_map
                    .get(change_set_pk.unwrap_or(ChangeSetPk::NONE), variant.id())
                {
                    Some(variant_spec) => ComponentSpecVariant::WorkspaceVariant {
                        variant_unique_id: variant_spec
                            .unique_id
                            .as_ref()
                            .unwrap_or(&variant.id().to_string())
                            .to_owned(),
                    },
                    None => {
                        let schema = component
                            .schema(ctx)
                            .await?
                            .ok_or(ComponentError::NoSchema(*component.id()))?;

                        ComponentSpecVariant::BuiltinVariant {
                            schema_name: schema.name().to_owned(),
                            variant_name: variant.name().to_owned(),
                        }
                    }
                };

                if let Some((component_spec, component_funcs, component_head_funcs)) = self
                    .export_component(ctx, change_set_pk, &component, component_variant)
                    .await?
                {
                    self.component_map.insert(
                        change_set_pk.unwrap_or(ChangeSetPk::NONE),
                        *component.id(),
                        component_spec.to_owned(),
                    );

                    component_specs.push(component_spec);
                    func_specs.extend_from_slice(&component_funcs);
                    head_funcs.extend_from_slice(&component_head_funcs);
                }
            }

            for edge in Edge::list(ctx).await? {
                let to_component_spec = self
                    .component_map
                    .get(
                        change_set_pk.unwrap_or(ChangeSetPk::NONE),
                        &edge.head_component_id(),
                    )
                    .ok_or(PkgError::EdgeRefersToMissingComponent(
                        edge.head_component_id(),
                    ))?
                    .clone();
                let from_component_spec = self
                    .component_map
                    .get(
                        change_set_pk.unwrap_or(ChangeSetPk::NONE),
                        &edge.tail_component_id(),
                    )
                    .ok_or(PkgError::EdgeRefersToMissingComponent(
                        edge.tail_component_id(),
                    ))?
                    .clone();
                edge_specs.push(
                    self.export_edge(ctx, &edge, &to_component_spec, &from_component_spec)
                        .await?,
                );
            }
        }

        Ok((
            func_specs,
            head_funcs,
            schema_specs,
            component_specs,
            edge_specs,
        ))
    }

    pub async fn export_edge(
        &mut self,
        ctx: &DalContext,
        edge: &Edge,
        to_component_spec: &ComponentSpec,
        from_component_spec: &ComponentSpec,
    ) -> PkgResult<EdgeSpec> {
        // head = to, tail = from
        let head_explicit_internal_provider =
            InternalProvider::find_explicit_for_socket(ctx, edge.head_socket_id())
                .await?
                .ok_or(EdgeError::InternalProviderNotFoundForSocket(
                    edge.head_socket_id(),
                ))?;
        let tail_external_provider = ExternalProvider::find_for_socket(ctx, edge.tail_socket_id())
            .await?
            .ok_or(EdgeError::ExternalProviderNotFoundForSocket(
                edge.tail_socket_id(),
            ))?;

        let mut edge_builder = EdgeSpec::builder();

        let to_socket_name = head_explicit_internal_provider.name().to_owned();

        let from_socket_name = tail_external_provider.name().to_owned();

        edge_builder
            .edge_kind(match edge.kind() {
                EdgeKind::Configuration => EdgeSpecKind::Configuration,
                EdgeKind::Symbolic => EdgeSpecKind::Symbolic,
            })
            .to_component_unique_id(&to_component_spec.unique_id)
            .to_socket_name(to_socket_name)
            .from_component_unique_id(&from_component_spec.unique_id)
            .from_socket_name(from_socket_name)
            .deleted(edge.visibility().is_deleted())
            .creation_user_pk(edge.creation_user_pk().map(|pk| pk.to_string()))
            .deletion_user_pk(edge.deletion_user_pk().map(|pk| pk.to_string()))
            .deleted_implicitly(edge.deleted_implicitly())
            .unique_id(*edge.id());

        Ok(edge_builder.build()?)
    }

    pub async fn export_component(
        &mut self,
        ctx: &DalContext,
        change_set_pk: Option<ChangeSetPk>,
        component: &Component,
        component_variant: ComponentSpecVariant,
    ) -> PkgResult<Option<(ComponentSpec, Vec<FuncSpec>, Vec<FuncSpec>)>> {
        let mut component_spec_builder = ComponentSpec::builder();
        component_spec_builder
            .name(component.name(ctx).await?)
            .unique_id(*component.id());
        let mut funcs = vec![];
        let mut head_funcs = vec![];

        let variant = component
            .schema_variant(ctx)
            .await?
            .ok_or(ComponentError::NoSchemaVariant(*component.id()))?;

        if variant.visibility().is_deleted() && component.visibility().is_deleted() {
            return Ok(None);
        }

        let node = component
            .node(ctx)
            .await?
            .pop()
            .ok_or(PkgError::ComponentMissingNode(*component.id()))?;
        component_spec_builder.variant(component_variant);

        let mut position_spec_builder = PositionSpec::builder();
        position_spec_builder.x(node.x());
        position_spec_builder.y(node.y());
        position_spec_builder.height(node.height().map(Into::into));
        position_spec_builder.width(node.width().map(Into::into));
        component_spec_builder.position(position_spec_builder.build()?);

        component_spec_builder.needs_destroy(component.needs_destroy());

        component_spec_builder
            .deletion_user_pk(component.deletion_user_pk().map(ToString::to_string));

        component_spec_builder.deleted(component.visibility().is_deleted());
        component_spec_builder.hidden(component.hidden());

        // ensure we are not in a deleted visibility here
        let new_ctx = ctx.clone_without_deleted_visibility();

        let debug_view = ComponentDebugView::new(&new_ctx, component).await?;
        for attribute in debug_view.attributes {
            let (attr_spec, attr_funcs, attr_head_funcs) = self
                .export_attribute_value(ctx, change_set_pk, attribute)
                .await?;
            funcs.extend_from_slice(&attr_funcs);
            head_funcs.extend_from_slice(&attr_head_funcs);
            component_spec_builder.attribute(attr_spec);
        }

        for attribute in debug_view.input_sockets {
            let (attr_spec, attr_funcs, attr_head_funcs) = self
                .export_attribute_value(ctx, change_set_pk, attribute)
                .await?;
            funcs.extend_from_slice(&attr_funcs);
            head_funcs.extend_from_slice(&attr_head_funcs);
            component_spec_builder.input_socket(attr_spec);
        }

        for attribute in debug_view.output_sockets {
            let (attr_spec, attr_funcs, attr_head_funcs) = self
                .export_attribute_value(ctx, change_set_pk, attribute)
                .await?;
            component_spec_builder.output_socket(attr_spec);
            funcs.extend_from_slice(&attr_funcs);
            head_funcs.extend_from_slice(&attr_head_funcs);
        }

        Ok(Some((component_spec_builder.build()?, funcs, head_funcs)))
    }

    pub async fn export_attribute_value(
        &mut self,
        ctx: &DalContext,
        change_set_pk: Option<ChangeSetPk>,
        view: AttributeDebugView,
    ) -> PkgResult<(AttributeValueSpec, Vec<FuncSpec>, Vec<FuncSpec>)> {
        let mut builder = AttributeValueSpec::builder();
        let mut funcs = vec![];
        let mut head_funcs = vec![];

        if let Some(parent_info) = view.parent_info {
            let parent_av = parent_info.value;
            let prop_id = parent_av.context.prop_id();
            if prop_id.is_some() {
                let parent_prop = Prop::get_by_id(ctx, &prop_id)
                    .await?
                    .ok_or(PropError::NotFound(prop_id, *ctx.visibility()))?;

                builder.parent_path(AttributeValuePath::Prop {
                    path: parent_prop.path().to_string(),
                    key: parent_info.key,
                    index: parent_info.array_index,
                });
            }
        }

        if let Some(prop) = &view.prop {
            let (key, index) = match &view.array_index {
                Some(index) => (None, Some(*index)),
                None => (view.attribute_value.key.to_owned(), None),
            };

            builder.path(AttributeValuePath::Prop {
                path: prop.path().to_string(),
                key,
                index,
            });
        } else if let Some(ip) = &view.internal_provider {
            builder.path(AttributeValuePath::InputSocket(ip.name().into()));
        } else if let Some(ep) = &view.external_provider {
            builder.path(AttributeValuePath::OutputSocket(ep.name().into()));
        }

        let func_id = *view.func.id();

        let func_unique_id = match self
            .func_map
            .get(change_set_pk.unwrap_or(ChangeSetPk::NONE), &func_id)
        {
            Some(func_spec) => {
                let func = Func::get_by_id(ctx, &func_id)
                    .await?
                    .ok_or(FuncError::NotFound(func_id))?;

                if func.visibility().change_set_pk == ChangeSetPk::NONE {
                    head_funcs.push(func_spec.to_owned());
                } else {
                    funcs.push(func_spec.to_owned());
                }

                func_spec.unique_id.to_owned()
            }
            None => {
                let func = Func::get_by_id(ctx, &func_id)
                    .await?
                    .ok_or(FuncError::NotFound(func_id))?;

                if func.visibility().change_set_pk == ChangeSetPk::NONE {
                    let (func_spec, _) = self
                        .export_func(ctx, Some(ChangeSetPk::NONE), &func)
                        .await?;
                    let unique_id = func_spec.unique_id.to_owned();
                    self.func_map
                        .insert(ChangeSetPk::NONE, func_id, func_spec.to_owned());
                    head_funcs.push(func_spec);

                    unique_id
                } else {
                    let (func_spec, _) = self.export_func(ctx, change_set_pk, &func).await?;
                    let unique_id = func_spec.unique_id.to_owned();
                    self.func_map.insert(
                        change_set_pk.unwrap_or(ChangeSetPk::NONE),
                        func_id,
                        func_spec.to_owned(),
                    );
                    funcs.push(func_spec);

                    unique_id
                }
            }
        };
        builder.func_unique_id(func_unique_id);
        builder.func_binding_args(view.func_execution.func_binding_args().to_owned());

        if let Some(handler) = view.func_execution.handler().as_deref() {
            builder.handler(handler);
        }

        builder.backend_kind(*view.func_execution.backend_kind());
        builder.response_type(*view.func_execution.backend_response_type());

        if let Some(code) = view.func_execution.code_base64().as_deref() {
            builder.code_base64(code);
        }

        if let Some(unprocessed_value) = view.func_binding_return_value.unprocessed_value() {
            builder.unprocessed_value(unprocessed_value.to_owned());
        }
        if let Some(value) = view.func_binding_return_value.value() {
            builder.value(value.to_owned());
        }
        if let Some(implicit_value) = view.implicit_attribute_value {
            if let Some(value) = implicit_value.get_value(ctx).await? {
                builder.implicit_value(value);
            }
        }
        if let Some(output_stream) = view.func_execution.output_stream() {
            builder.output_stream(serde_json::to_value(output_stream)?);
        }
        builder.is_proxy(
            view.attribute_value
                .proxy_for_attribute_value_id()
                .is_some(),
        );
        builder.sealed_proxy(view.attribute_value.sealed_proxy());

        if view.prototype.context.component_id().is_some() {
            builder.component_specific(true);
        }

        let inputs = self
            .export_input_func_and_arguments(ctx, change_set_pk, &view.prototype)
            .await?;

        if let Some((_, inputs)) = inputs {
            builder.inputs(inputs);
        }

        Ok((builder.build()?, funcs, head_funcs))
    }

    pub async fn export(&mut self, ctx: &DalContext) -> PkgResult<SiPkg> {
        let mut pkg_spec_builder = PkgSpec::builder();
        pkg_spec_builder
            .name(&self.name)
            .kind(self.kind)
            .version(&self.version)
            .created_by(&self.created_by);

        if let Some(workspace_pk) = ctx.tenancy().workspace_pk() {
            pkg_spec_builder.workspace_pk(workspace_pk.to_string());
            let workspace = Workspace::get_by_pk(ctx, &workspace_pk)
                .await?
                .ok_or(PkgError::WorkspaceNotFound(workspace_pk))?;
            pkg_spec_builder.workspace_name(workspace.name());
        }

        if let Some(description) = &self.description {
            pkg_spec_builder.description(description);
        }

        match self.kind {
            SiPkgKind::Module => {
                let (funcs, _, schemas, _, _) = self.export_change_set(ctx, None).await?;
                pkg_spec_builder.funcs(funcs);
                pkg_spec_builder.schemas(schemas);
            }
            SiPkgKind::WorkspaceBackup => {
                let (mut head_funcs, funcs, schemas, components, edges) =
                    self.export_change_set(ctx, Some(ChangeSetPk::NONE)).await?;

                head_funcs.extend_from_slice(&funcs);

                let mut head_builder = ChangeSetSpec::builder();
                head_builder
                    .name("head")
                    .schemas(schemas)
                    .components(components)
                    .edges(edges);

                pkg_spec_builder.default_change_set("head");

                for change_set in ChangeSet::list_open(ctx).await? {
                    let (funcs, referenced_head_funcs, schemas, components, edges) =
                        self.export_change_set(ctx, Some(change_set.pk)).await?;
                    head_funcs.extend_from_slice(&referenced_head_funcs);

                    pkg_spec_builder.change_set(
                        ChangeSetSpec::builder()
                            .name(&change_set.name)
                            .based_on_change_set("head")
                            .funcs(remove_duplicate_func_specs(&funcs))
                            .schemas(schemas)
                            .components(components)
                            .edges(edges)
                            .build()?,
                    );
                }

                pkg_spec_builder.change_set(
                    head_builder
                        .funcs(remove_duplicate_func_specs(&head_funcs))
                        .build()?,
                );
            }
        }

        let spec = pkg_spec_builder.build()?;
        let pkg = SiPkg::load_from_spec(spec)?;

        Ok(pkg)
    }
}

fn remove_duplicate_func_specs(func_specs: &[FuncSpec]) -> Vec<FuncSpec> {
    let mut unique_id_set = HashSet::new();

    func_specs
        .iter()
        .filter_map(|spec| {
            if unique_id_set.contains(&spec.unique_id) {
                None
            } else {
                unique_id_set.insert(spec.unique_id.to_owned());
                Some(spec.to_owned())
            }
        })
        .collect()
}

pub async fn get_component_type(
    ctx: &DalContext,
    variant: &SchemaVariant,
) -> Result<SchemaVariantSpecComponentType, PkgError> {
    let type_prop = variant.find_prop(ctx, &["root", "si", "type"]).await?;
    let type_context = AttributeReadContext {
        prop_id: Some(*type_prop.id()),
        ..Default::default()
    };

    let type_av = AttributeValue::find_for_context(ctx, type_context)
        .await?
        .ok_or_else(|| SchemaVariantError::AttributeValueNotFoundForContext(type_context.into()))?;

    Ok(match type_av.get_value(ctx).await? {
        Some(type_value) => {
            let component_type: ComponentType = serde_json::from_value(type_value)?;
            component_type.into()
        }
        None => SchemaVariantSpecComponentType::default(),
    })
}
