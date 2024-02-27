use std::{collections::HashMap, path::Path, str::FromStr};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use si_pkg::{
    AttributeValuePath, ComponentSpec, ComponentSpecVariant, EdgeSpec, EdgeSpecKind,
    FuncArgumentSpec, FuncSpec, FuncSpecData, SchemaVariantSpecPropRoot, SiPkg, SiPkgActionFunc,
    SiPkgAttrFuncInputView, SiPkgAuthFunc, SiPkgComponent, SiPkgEdge, SiPkgError, SiPkgFunc,
    SiPkgKind, SiPkgLeafFunction, SiPkgMetadata, SiPkgProp, SiPkgPropData, SiPkgSchema,
    SiPkgSchemaData, SiPkgSchemaVariant, SiPkgSocket, SiPkgSocketData, SocketSpecKind,
};
use telemetry::prelude::*;

use crate::{
    authentication_prototype::{AuthenticationPrototype, AuthenticationPrototypeContext},
    component::{self},
    property_editor::values_summary::PropertyEditorValuesSummary,
};
use crate::{
    component::ComponentKind,
    edge::EdgeKind,
    func::{
        self,
        argument::{FuncArgumentError, FuncArgumentKind},
        binding::FuncBinding,
        binding_return_value::FuncBindingReturnValue,
    },
    installed_pkg::{
        InstalledPkg, InstalledPkgAsset, InstalledPkgAssetKind, InstalledPkgAssetTyped,
        InstalledPkgId,
    },
    prop::PropPath,
    schema::{
        variant::{
            definition::{SchemaVariantDefinition, SchemaVariantDefinitionJson},
            leaves::LeafInputLocation,
        },
        SchemaUiMenu,
    },
    socket::SocketEdgeKind,
    ActionKind, ActionPrototype, ActionPrototypeContext, AttributeContextBuilder,
    AttributePrototype, AttributePrototypeArgument, AttributePrototypeId, AttributeReadContext,
    AttributeValue, AttributeValueError, ChangeSet, ChangeSetPk, Component, DalContext, Edge,
    ExternalProvider, ExternalProviderId, Func, FuncArgument, FuncError, FuncId, InternalProvider,
    InternalProviderError, LeafKind, Node, Prop, PropId, PropKind, Schema, SchemaId, SchemaVariant,
    SchemaVariantError, SchemaVariantId, Socket, StandardModel, Tenancy, UserPk, Workspace,
    WorkspacePk,
};

use super::{PkgError, PkgResult};

#[derive(Clone, Debug)]
pub enum Thing {
    ActionPrototype(ActionPrototype),
    AuthPrototype(AuthenticationPrototype),
    AttributePrototypeArgument(AttributePrototypeArgument),
    Component((Component, Node)),
    Edge(Edge),
    Func(Func),
    FuncArgument(FuncArgument),
    Schema(Schema),
    SchemaVariant(SchemaVariant),
    Socket(Box<(Socket, Option<InternalProvider>, Option<ExternalProvider>)>),
}

pub type ThingMap = super::ChangeSetThingMap<String, Thing>;

#[derive(Clone, Debug, Default)]
pub struct ImportOptions {
    pub schemas: Option<Vec<String>>,
    pub skip_import_funcs: Option<HashMap<String, Func>>,
    /// If set to `true`, the importer will install the assets from the module
    /// but will not make a record of the install as an "installed module".
    pub no_record: bool,
    /// If set to `true` then we will set the functions to a builtin
    /// in the UI. They will be marked as such.
    pub is_builtin: bool,
}

const SPECIAL_CASE_FUNCS: [&str; 2] = ["si:resourcePayloadToValue", "si:normalizeToArray"];

#[allow(clippy::too_many_arguments)]
async fn import_change_set(
    ctx: &DalContext,
    change_set_pk: ChangeSetPk,
    metadata: &SiPkgMetadata,
    funcs: &[SiPkgFunc<'_>],
    schemas: &[SiPkgSchema<'_>],
    components: &[SiPkgComponent<'_>],
    edges: &[SiPkgEdge<'_>],
    installed_pkg_id: Option<InstalledPkgId>,
    thing_map: &mut ThingMap,
    options: &ImportOptions,
) -> PkgResult<(
    Vec<SchemaVariantId>,
    Vec<(String, Vec<ImportAttributeSkip>)>,
    Vec<ImportEdgeSkip>,
)> {
    for func_spec in funcs {
        // This is a hack because the hash of the intrinsics has changed from the version in the
        // packages. We also apply this to si:resourcePayloadToValue since it should be an
        // intrinsic but is only in our packages
        if func::is_intrinsic(func_spec.name())
            || SPECIAL_CASE_FUNCS.contains(&func_spec.name())
            || func_spec.is_from_builtin().unwrap_or(false)
        {
            let hash = func_spec.hash();
            let func_spec: SiPkgFunc<'_> = func_spec.clone();
            let func_spec: FuncSpec = func_spec.try_into()?;
            if let (Some(mut func), Some(data)) = (
                Func::find_by_name(ctx, &func_spec.name).await?,
                &func_spec.data,
            ) {
                func.set_description(ctx, data.description.clone()).await?;
                func.set_display_name(ctx, data.display_name.clone())
                    .await?;
                func.set_handler(ctx, Some(data.handler.clone())).await?;
                func.set_link(ctx, data.link.clone()).await?;
                func.set_hidden(ctx, data.hidden).await?;
                func.set_backend_kind(ctx, data.backend_kind).await?;
                func.set_backend_response_type(ctx, data.response_type)
                    .await?;
                func.set_code_base64(ctx, Some(data.code_base64.clone()))
                    .await?;

                thing_map.insert(
                    change_set_pk,
                    func_spec.unique_id.to_owned(),
                    Thing::Func(func.to_owned()),
                );
            } else if let Some(func) = import_func(
                ctx,
                change_set_pk,
                &func_spec,
                Some(hash),
                installed_pkg_id,
                thing_map,
                options.is_builtin,
            )
            .await?
            {
                let args = func_spec.arguments;

                if !args.is_empty() {
                    import_func_arguments(ctx, change_set_pk, *func.id(), &args, thing_map).await?;
                }
            }
        } else {
            let hash = func_spec.hash();
            let func_spec: SiPkgFunc<'_> = func_spec.clone();
            let func_spec: FuncSpec = func_spec.try_into()?;

            let func = if let Some(Some(func)) = options
                .skip_import_funcs
                .as_ref()
                .map(|skip_funcs| skip_funcs.get(&func_spec.unique_id))
            {
                if let Some(installed_pkg_id) = installed_pkg_id {
                    InstalledPkgAsset::new(
                        ctx,
                        InstalledPkgAssetTyped::new_for_func(
                            *func.id(),
                            installed_pkg_id,
                            hash.to_string(),
                        ),
                    )
                    .await?;
                }

                // We're not going to import this func but we need it in the map for lookups later
                thing_map.insert(
                    change_set_pk,
                    func_spec.unique_id.to_owned(),
                    Thing::Func(func.to_owned()),
                );

                None
            } else {
                import_func(
                    ctx,
                    change_set_pk,
                    &func_spec,
                    Some(hash),
                    installed_pkg_id,
                    thing_map,
                    options.is_builtin,
                )
                .await?
            };

            if let Some(func) = func {
                let args = func_spec.arguments;

                if !args.is_empty() {
                    import_func_arguments(ctx, change_set_pk, *func.id(), &args, thing_map).await?;
                }
            }
        };
    }

    let mut installed_schema_variant_ids = vec![];

    for schema_spec in schemas {
        match &options.schemas {
            None => {}
            Some(schemas) => {
                if !schemas.contains(&schema_spec.name().to_string().to_lowercase()) {
                    continue;
                }
            }
        }

        info!(
            "installing schema '{}' from {}",
            schema_spec.name(),
            metadata.name(),
        );

        let (_, schema_variant_ids) = import_schema(
            ctx,
            change_set_pk,
            schema_spec,
            installed_pkg_id,
            thing_map,
            metadata,
        )
        .await?;

        installed_schema_variant_ids.extend(schema_variant_ids);
    }

    let mut component_attribute_skips = vec![];
    for component_spec in components {
        let component_spec: SiPkgComponent<'_> = component_spec.clone();
        let name = component_spec.name().to_owned();
        let skips = import_component(
            ctx,
            change_set_pk,
            component_spec.try_into()?,
            thing_map,
            false,
        )
        .await?;
        if !skips.is_empty() {
            component_attribute_skips.push((name, skips));
        }
    }

    let mut edge_skips = vec![];
    for edge_spec in edges {
        let edge_spec: SiPkgEdge<'_> = edge_spec.clone();
        if let Some(skip) =
            import_edge(ctx, change_set_pk, &edge_spec.try_into()?, thing_map).await?
        {
            edge_skips.push(skip);
        }
    }

    AttributeValue::remove_dependency_summaries_for_deleted_values(ctx).await?;
    for component in Component::list(ctx).await? {
        AttributeValue::update_component_dependencies(ctx, *component.id()).await?;
        PropertyEditorValuesSummary::create_or_update_component_entry(ctx, *component.id()).await?;
    }

    info!("Finished Imports: {}", Utc::now());

    Ok((
        installed_schema_variant_ids,
        component_attribute_skips,
        edge_skips,
    ))
}

async fn import_edge(
    ctx: &DalContext,
    change_set_pk: ChangeSetPk,
    edge_spec: &EdgeSpec,
    thing_map: &mut ThingMap,
) -> PkgResult<Option<ImportEdgeSkip>> {
    let edge = match thing_map.get(change_set_pk, &edge_spec.unique_id.clone()) {
        Some(Thing::Edge(edge)) => Some(edge.to_owned()),
        _ => {
            if !edge_spec.deleted {
                let head_component_unique_id = edge_spec.to_component_unique_id.clone();
                let (_, head_node) = match thing_map.get(change_set_pk, &head_component_unique_id) {
                    Some(Thing::Component((component, node))) => (component, node),
                    _ => {
                        return Err(PkgError::MissingComponentForEdge(
                            head_component_unique_id,
                            edge_spec.from_socket_name.clone(),
                            edge_spec.to_socket_name.clone(),
                        ));
                    }
                };

                let tail_component_unique_id = edge_spec.from_component_unique_id.clone();
                let (_, tail_node) = match thing_map.get(change_set_pk, &tail_component_unique_id) {
                    Some(Thing::Component((component, node))) => (component, node),
                    _ => {
                        return Err(PkgError::MissingComponentForEdge(
                            tail_component_unique_id,
                            edge_spec.from_socket_name.clone(),
                            edge_spec.to_socket_name.clone(),
                        ));
                    }
                };

                let to_socket = match Socket::find_by_name_for_edge_kind_and_node(
                    ctx,
                    &edge_spec.to_socket_name,
                    SocketEdgeKind::ConfigurationInput,
                    *head_node.id(),
                )
                .await?
                {
                    Some(socket) => socket,
                    None => {
                        return Ok(Some(ImportEdgeSkip::MissingInputSocket(
                            edge_spec.to_socket_name.clone(),
                        )));
                    }
                };

                let from_socket = match Socket::find_by_name_for_edge_kind_and_node(
                    ctx,
                    &edge_spec.from_socket_name,
                    SocketEdgeKind::ConfigurationOutput,
                    *tail_node.id(),
                )
                .await?
                {
                    Some(socket) => socket,
                    None => {
                        return Ok(Some(ImportEdgeSkip::MissingOutputSocket(
                            edge_spec.from_socket_name.clone(),
                        )));
                    }
                };

                Some(
                    Edge::new_for_connection(
                        ctx,
                        *head_node.id(),
                        *to_socket.id(),
                        *tail_node.id(),
                        *from_socket.id(),
                        match edge_spec.edge_kind {
                            EdgeSpecKind::Configuration => EdgeKind::Configuration,
                            EdgeSpecKind::Symbolic => EdgeKind::Symbolic,
                        },
                    )
                    .await?,
                )
            } else {
                None
            }
        }
    };

    if let Some(mut edge) = edge {
        let creation_user_pk = match &edge_spec.creation_user_pk {
            Some(pk_str) => Some(UserPk::from_str(pk_str)?),
            None => None,
        };
        if creation_user_pk.as_ref() != edge.creation_user_pk() {
            edge.set_creation_user_pk(ctx, creation_user_pk).await?;
        }

        let deletion_user_pk = match &edge_spec.deletion_user_pk {
            Some(pk_str) => Some(UserPk::from_str(pk_str)?),
            None => None,
        };

        if deletion_user_pk.as_ref() != edge.deletion_user_pk() {
            edge.set_deletion_user_pk(ctx, deletion_user_pk).await?;
        }

        if edge.deleted_implicitly() != edge_spec.deleted_implicitly {
            edge.set_deleted_implicitly(ctx, edge_spec.deleted_implicitly)
                .await?;
        }

        if edge.visibility().is_deleted() && !edge_spec.deleted {
            Edge::restore_by_id(ctx, *edge.id()).await?;
        } else if !edge.visibility().is_deleted() && edge_spec.deleted {
            // ignore errors here, since they mostly occur if we've already deleted a node that
            // this is an edge to
            let _ = edge.delete_and_propagate(ctx).await;
        }

        thing_map.insert(
            change_set_pk,
            edge_spec.unique_id.clone(),
            Thing::Edge(edge),
        );
    }

    Ok(None)
}

async fn import_component(
    ctx: &DalContext,
    change_set_pk: ChangeSetPk,
    component_spec: ComponentSpec,
    thing_map: &mut ThingMap,
    force_resource_patch: bool,
) -> PkgResult<Vec<ImportAttributeSkip>> {
    let variant = match &component_spec.variant {
        ComponentSpecVariant::BuiltinVariant {
            schema_name,
            variant_name,
        } => {
            let schema = Schema::find_by_name_builtin(ctx, schema_name.as_str())
                .await?
                .ok_or(PkgError::ComponentMissingBuiltinSchema(
                    schema_name.to_owned(),
                    component_spec.name.clone(),
                ))?;

            schema
                .find_variant_by_name(ctx, variant_name.as_str())
                .await?
                .ok_or(PkgError::ComponentMissingBuiltinSchemaVariant(
                    schema_name.to_owned(),
                    variant_name.to_owned(),
                    component_spec.name.clone(),
                ))?
        }
        ComponentSpecVariant::UpdateVariant {
            schema_name,
            variant_name,
        } => {
            let schema = Schema::find_by_name(ctx, schema_name.as_str()).await?;

            schema
                .find_variant_by_name(ctx, variant_name.as_str())
                .await?
                .ok_or(PkgError::ComponentMissingUpdateSchemaVariant(
                    schema_name.to_owned(),
                    variant_name.to_owned(),
                    component_spec.name.clone(),
                ))?
        }
        ComponentSpecVariant::WorkspaceVariant { variant_unique_id } => {
            match thing_map.get(change_set_pk, variant_unique_id) {
                Some(Thing::SchemaVariant(variant)) => variant.to_owned(),
                _ => {
                    return Err(PkgError::ComponentMissingSchemaVariant(
                        variant_unique_id.to_owned(),
                        component_spec.name.clone(),
                    ));
                }
            }
        }
    };

    let (mut component, mut node) = match thing_map
        .get(change_set_pk, &component_spec.unique_id.clone())
    {
        Some(Thing::Component((existing_component, node))) => {
            if Component::schema_variant_id(ctx, *existing_component.id()).await? != *variant.id() {
                // If the component exists already, but the schema variant is
                // different, we need to respin the component to the change-set
                // specific schema variant
                (
                    Component::respin(ctx, *existing_component.id(), *variant.id()).await?,
                    node.to_owned(),
                )
            } else {
                (existing_component.to_owned(), node.to_owned())
            }
        }
        _ => {
            let (component, node) =
                Component::new(ctx, component_spec.name.clone(), *variant.id()).await?;

            thing_map.insert(
                change_set_pk,
                component_spec.unique_id.clone(),
                Thing::Component((component.to_owned(), node.to_owned())),
            );

            (component, node)
        }
    };

    if component.name(ctx).await? != component_spec.name {
        component
            .set_name(ctx, Some(component_spec.name.clone()))
            .await?;
    }

    let position = component_spec.position;
    if node.x() != position.x
        || node.y() != position.y
        || node.height() != position.height.as_deref()
        || node.width() != position.width.as_deref()
    {
        // Use set_geometry to ensure summary diagram gets updated positioning
        node.set_geometry(ctx, position.x, position.y, position.width, position.height)
            .await?;
    }

    let mut component_root_implicit_value =
        component::migrate::build_empty_json_for_prop_tree(ctx, *variant.id()).await?;
    let imported_json = component_spec.attributes[0]
        .implicit_value
        .as_ref()
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    component::migrate::serde_value_merge_in_place_recursive(
        &mut component_root_implicit_value,
        imported_json,
    );

    if component_root_implicit_value != serde_json::Value::Null {
        let root_attribute_value = component.root_attribute_value(ctx).await?;
        AttributeValue::update_for_context_without_propagating_dependent_values(
            ctx,
            *root_attribute_value.id(),
            None,
            root_attribute_value.context,
            Some(component_root_implicit_value),
            None,
        )
        .await?;

        component::migrate::restore_prototypes_and_implicit_values(ctx, *component.id()).await?;
    }

    let mut resource_value = None;
    for attribute in &component_spec.attributes {
        if let AttributeValuePath::Prop { path, .. } = &attribute.path {
            if path == &PropPath::new(["root", "resource"]).to_string() {
                resource_value = attribute.implicit_value.clone();
                break;
            }
        }
    }

    if component_spec.needs_destroy {
        component.set_needs_destroy(ctx, true).await?;
    }

    if let Some(resource_value) = resource_value {
        if force_resource_patch || change_set_pk == ChangeSetPk::NONE {
            if let Ok(result) = serde_json::from_value(resource_value) {
                component.set_resource(ctx, result).await?;
            }
        }
    }

    if component.visibility().is_deleted() && !component_spec.deleted {
        Component::restore_and_propagate(ctx, *component.id()).await?;
    } else if !component.visibility().is_deleted() && component_spec.deleted {
        component.delete_and_propagate(ctx).await?;
    }

    Ok(vec![])
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportSkips {
    pub change_set_pk: ChangeSetPk,
    pub edge_skips: Vec<ImportEdgeSkip>,
    pub attribute_skips: Vec<(String, Vec<ImportAttributeSkip>)>,
}

#[remain::sorted]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ImportAttributeSkip {
    #[serde(rename_all = "camelCase")]
    KindMismatch {
        path: PropPath,
        expected_kind: PropKind,
        variant_kind: PropKind,
    },
    MissingInputSocket(String),
    MissingOutputSocket(String),
    MissingProp(PropPath),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ImportEdgeSkip {
    MissingInputSocket(String),
    MissingOutputSocket(String),
}

pub async fn import_pkg_from_pkg(
    ctx: &DalContext,
    pkg: &SiPkg,
    options: Option<ImportOptions>,
) -> PkgResult<(
    Option<InstalledPkgId>,
    Vec<SchemaVariantId>,
    Option<Vec<ImportSkips>>,
)> {
    // We have to write the installed_pkg row first, so that we have an id, and rely on transaction
    // semantics to remove the row if anything in the installation process fails
    let root_hash = pkg.hash()?.to_string();

    let options = options.unwrap_or_default();

    if InstalledPkg::find_by_hash(ctx, &root_hash).await?.is_some() {
        return Err(PkgError::PackageAlreadyInstalled(root_hash));
    }

    let metadata = pkg.metadata()?;

    let installed_pkg_id = if options.no_record {
        None
    } else {
        Some(
            *InstalledPkg::new(ctx, metadata.name(), pkg.hash()?.to_string())
                .await?
                .id(),
        )
    };

    let mut change_set_things = ThingMap::new();

    match metadata.kind() {
        SiPkgKind::Module => {
            let (installed_schema_variant_ids, _, _) = import_change_set(
                ctx,
                ctx.visibility().change_set_pk,
                &metadata,
                &pkg.funcs()?,
                &pkg.schemas()?,
                &[],
                &[],
                installed_pkg_id,
                &mut change_set_things,
                &options,
            )
            .await?;

            Ok((installed_pkg_id, installed_schema_variant_ids, None))
        }
        SiPkgKind::WorkspaceBackup => {
            let mut ctx = ctx.clone_with_new_visibility(ctx.visibility().to_head());

            let mut import_skips = vec![];

            let workspace_pk = WorkspacePk::from_str(
                metadata
                    .workspace_pk()
                    .ok_or(PkgError::WorkspacePkNotInBackup)?,
            )?;
            let workspace_name = metadata
                .workspace_name()
                .ok_or(PkgError::WorkspaceNameNotInBackup)?;
            let default_change_set_name = metadata.default_change_set().unwrap_or("head");

            Workspace::clear_or_create_workspace(&mut ctx, workspace_pk, workspace_name).await?;

            ctx.update_tenancy(Tenancy::new(workspace_pk));

            let change_sets = pkg.change_sets()?;
            let default_change_set = change_sets
                .iter()
                .find(|cs| cs.name() == default_change_set_name)
                .ok_or(PkgError::WorkspaceBackupNoDefaultChangeSet(
                    default_change_set_name.into(),
                ))?;

            let (_, attribute_skips, edge_skips) = import_change_set(
                &ctx,
                ChangeSetPk::NONE,
                &metadata,
                &default_change_set.funcs()?,
                &default_change_set.schemas()?,
                &default_change_set.components()?,
                &default_change_set.edges()?,
                installed_pkg_id,
                &mut change_set_things,
                &options,
            )
            .await?;

            import_skips.push(ImportSkips {
                change_set_pk: ChangeSetPk::NONE,
                attribute_skips,
                edge_skips,
            });

            for change_set in change_sets {
                if change_set.name() == default_change_set_name {
                    continue;
                }

                // Revert to head to create new change set
                let ctx = ctx.clone_with_new_visibility(ctx.visibility().to_head());
                let new_cs = ChangeSet::new(&ctx, change_set.name(), None).await?;
                // Switch to new change set visibility
                let ctx = ctx.clone_with_new_visibility(ctx.visibility().to_change_set(new_cs.pk));

                let (_, attribute_skips, edge_skips) = import_change_set(
                    &ctx,
                    new_cs.pk,
                    &metadata,
                    &change_set.funcs()?,
                    &change_set.schemas()?,
                    &change_set.components()?,
                    &change_set.edges()?,
                    installed_pkg_id,
                    &mut change_set_things,
                    &options,
                )
                .await?;

                import_skips.push(ImportSkips {
                    change_set_pk: new_cs.pk,
                    attribute_skips,
                    edge_skips,
                });
            }

            Ok((
                None,
                vec![],
                if import_skips.is_empty() {
                    None
                } else {
                    Some(import_skips)
                },
            ))
        }
    }
}

pub async fn import_pkg(ctx: &DalContext, pkg_file_path: impl AsRef<Path>) -> PkgResult<SiPkg> {
    println!("Importing package from {:?}", pkg_file_path.as_ref());
    let pkg = SiPkg::load_from_file(&pkg_file_path).await?;

    import_pkg_from_pkg(ctx, &pkg, None).await?;

    Ok(pkg)
}

async fn create_func(ctx: &DalContext, func_spec: &FuncSpec) -> PkgResult<Func> {
    let name = func_spec.name.clone();

    let func_spec_data = func_spec
        .data
        .clone()
        .ok_or_else(|| PkgError::DataNotFound(name.clone()))?;

    // How to handle name conflicts?
    let mut func = Func::new(
        ctx,
        name,
        func_spec_data.backend_kind.into(),
        func_spec_data.response_type.into(),
    )
    .await?;

    func.set_display_name(ctx, func_spec_data.display_name.clone())
        .await?;
    func.set_code_base64(ctx, Some(func_spec_data.code_base64.clone()))
        .await?;
    func.set_description(ctx, func_spec_data.description.clone())
        .await?;
    func.set_handler(ctx, Some(func_spec_data.handler.clone()))
        .await?;
    func.set_hidden(ctx, func_spec_data.hidden).await?;
    func.set_link(ctx, func_spec_data.link.map(|l| l.to_string()))
        .await?;

    Ok(func)
}

async fn update_func(
    ctx: &DalContext,
    func: &mut Func,
    func_spec_data: &FuncSpecData,
) -> PkgResult<()> {
    func.set_name(ctx, func_spec_data.name.clone()).await?;
    func.set_backend_kind(ctx, func_spec_data.backend_kind)
        .await?;
    func.set_backend_response_type(ctx, func_spec_data.response_type)
        .await?;
    func.set_display_name(ctx, func_spec_data.display_name.clone())
        .await?;
    func.set_code_base64(ctx, Some(func_spec_data.code_base64.clone()))
        .await?;
    func.set_description(ctx, func_spec_data.description.clone())
        .await?;
    func.set_handler(ctx, Some(func_spec_data.handler.clone()))
        .await?;
    func.set_hidden(ctx, func_spec_data.hidden).await?;
    func.set_link(ctx, func_spec_data.link.clone()).await?;

    Ok(())
}

pub async fn import_func(
    ctx: &DalContext,
    change_set_pk: ChangeSetPk,
    func_spec: &FuncSpec,
    hash: Option<object_tree::Hash>,
    installed_pkg_id: Option<InstalledPkgId>,
    thing_map: &mut ThingMap,
    is_builtin: bool,
) -> PkgResult<Option<Func>> {
    let mut func = {
        let existing_func = InstalledPkgAsset::list_for_kind_and_hash(
            ctx,
            InstalledPkgAssetKind::Func,
            &hash.map_or_else(String::new, |h| h.to_string()),
        )
        .await?
        .pop();

        if let Some(installed_func_record) = existing_func {
            match installed_func_record.as_installed_func()? {
                InstalledPkgAssetTyped::Func { id, .. } => match Func::get_by_id(ctx, &id).await? {
                    Some(mut func) => {
                        if is_builtin {
                            func.set_builtin(ctx, true).await?
                        }

                        if let (Some(installed_pkg_id), Some(hash)) = (installed_pkg_id, hash) {
                            InstalledPkgAsset::new(
                                ctx,
                                InstalledPkgAssetTyped::new_for_func(
                                    *func.id(),
                                    installed_pkg_id,
                                    hash.to_string(),
                                ),
                            )
                            .await?;
                        }

                        thing_map.insert(
                            change_set_pk,
                            func_spec.unique_id.clone(),
                            Thing::Func(func.to_owned()),
                        );
                        None
                    }
                    None => return Err(PkgError::InstalledFuncMissing(id)),
                },
                _ => unreachable!(),
            }
        } else {
            let existing_func = thing_map.get(change_set_pk, &func_spec.unique_id.clone());

            match existing_func {
                Some(Thing::Func(existing_func)) => {
                    let mut existing_func = existing_func.to_owned();

                    if func_spec.deleted {
                        existing_func.delete_by_id(ctx).await?;

                        None
                    } else {
                        if let Some(data) = &func_spec.data {
                            update_func(ctx, &mut existing_func, data).await?;
                        }

                        Some(existing_func)
                    }
                }
                _ => {
                    if func_spec.deleted {
                        // If we're "deleted" but there is no existing function, this means we're
                        // deleted only in a change set. Do nothing
                        None
                    } else {
                        Some(create_func(ctx, func_spec).await?)
                    }
                }
            }
        }
    };

    if let Some(func) = func.as_mut() {
        if is_builtin {
            func.set_builtin(ctx, true).await?
        }

        if let (Some(installed_pkg_id), Some(hash)) = (installed_pkg_id, hash) {
            InstalledPkgAsset::new(
                ctx,
                InstalledPkgAssetTyped::new_for_func(
                    *func.id(),
                    installed_pkg_id,
                    hash.to_string(),
                ),
            )
            .await?;
        }

        thing_map.insert(
            change_set_pk,
            func_spec.unique_id.clone(),
            Thing::Func(func.to_owned()),
        );
    }

    Ok(func)
}

async fn create_func_argument(
    ctx: &DalContext,
    func_id: FuncId,
    func_arg: &FuncArgumentSpec,
) -> PkgResult<FuncArgument> {
    Ok(FuncArgument::new(
        ctx,
        func_arg.name.clone(),
        func_arg.kind.into(),
        func_arg.element_kind.as_ref().map(|&kind| kind.into()),
        func_id,
    )
    .await?)
}

async fn update_func_argument(
    ctx: &DalContext,
    existing_arg: &mut FuncArgument,
    func_id: FuncId,
    func_arg: &FuncArgumentSpec,
) -> PkgResult<()> {
    existing_arg.set_name(ctx, &func_arg.name).await?;
    existing_arg.set_kind(ctx, func_arg.kind).await?;
    let element_kind: Option<FuncArgumentKind> =
        func_arg.element_kind.as_ref().map(|&kind| kind.into());
    existing_arg.set_element_kind(ctx, element_kind).await?;
    existing_arg.set_func_id(ctx, func_id).await?;

    Ok(())
}

async fn import_func_arguments(
    ctx: &DalContext,
    change_set_pk: ChangeSetPk,
    func_id: FuncId,
    func_arguments: &[FuncArgumentSpec],
    thing_map: &mut ThingMap,
) -> PkgResult<()> {
    for arg in func_arguments {
        match arg.unique_id.as_deref().map(|unique_id| {
            (
                unique_id,
                thing_map.get(change_set_pk, &unique_id.to_owned()),
            )
        }) {
            Some((unique_id, Some(Thing::FuncArgument(existing_arg)))) => {
                let mut existing_arg = existing_arg.to_owned();

                if arg.deleted {
                    existing_arg.delete_by_id(ctx).await?;
                } else {
                    update_func_argument(ctx, &mut existing_arg, func_id, arg).await?;
                    thing_map.insert(
                        change_set_pk,
                        unique_id.to_owned(),
                        Thing::FuncArgument(existing_arg.to_owned()),
                    );
                }
            }
            Some((unique_id, _)) => {
                if !arg.deleted {
                    let new_arg = create_func_argument(ctx, func_id, arg).await?;
                    thing_map.insert(
                        change_set_pk,
                        unique_id.to_owned(),
                        Thing::FuncArgument(new_arg),
                    );
                }
            }
            None => {
                create_func_argument(ctx, func_id, arg).await?;
            }
        }
    }

    Ok(())
}

async fn create_schema(ctx: &DalContext, schema_spec_data: &SiPkgSchemaData) -> PkgResult<Schema> {
    let mut schema = Schema::new(ctx, schema_spec_data.name(), &ComponentKind::Standard).await?;
    schema
        .set_ui_hidden(ctx, schema_spec_data.ui_hidden())
        .await?;

    let ui_menu = SchemaUiMenu::new(
        ctx,
        schema_spec_data
            .category_name()
            .unwrap_or_else(|| schema_spec_data.name()),
        schema_spec_data.category(),
    )
    .await?;
    ui_menu.set_schema(ctx, schema.id()).await?;

    Ok(schema)
}

async fn update_schema(
    ctx: &DalContext,
    schema: &mut Schema,
    schema_spec_data: &SiPkgSchemaData,
) -> PkgResult<()> {
    if schema_spec_data.name() != schema.name() {
        schema.set_name(ctx, schema_spec_data.name()).await?;
    }

    if schema_spec_data.ui_hidden() != schema.ui_hidden() {
        schema
            .set_ui_hidden(ctx, schema_spec_data.ui_hidden())
            .await?;
    }

    if let Some(mut ui_menu) = schema.ui_menus(ctx).await?.pop() {
        if let Some(category_name) = schema_spec_data.category_name() {
            if category_name != ui_menu.name() {
                ui_menu.set_name(ctx, category_name).await?;
            }
            if schema_spec_data.category() != ui_menu.category() {
                ui_menu.set_name(ctx, schema_spec_data.category()).await?;
            }
        }
    }

    Ok(())
}

async fn import_schema(
    ctx: &DalContext,
    change_set_pk: ChangeSetPk,
    schema_spec: &SiPkgSchema<'_>,
    installed_pkg_id: Option<InstalledPkgId>,
    thing_map: &mut ThingMap,
    metadata: &SiPkgMetadata,
) -> PkgResult<(Option<SchemaId>, Vec<SchemaVariantId>)> {
    let hash = schema_spec.hash().to_string();
    let schema = {
        // If this is a workspace backup and the schema is a builtin, then it's
        // in the backup *only* because it has a custom schema variant.  We
        // expect that the builtin schema is already imported, so we use that.
        // If it's missing, we're erroring out, but what we should do is try and
        // find the missing builtin schema on the remote module index, and then
        // import that.
        if metadata.kind() == SiPkgKind::WorkspaceBackup && schema_spec.is_builtin() {
            let existing_schema = Schema::find_by_name_builtin(ctx, schema_spec.name())
                .await?
                .ok_or(PkgError::MissingBuiltinSchema(
                    schema_spec.name().to_owned(),
                ))?;

            Some(existing_schema)
        } else {
            let existing_schema = InstalledPkgAsset::list_for_kind_and_hash(
                ctx,
                InstalledPkgAssetKind::Schema,
                &hash,
            )
            .await?
            .pop();

            if let Some(installed_schema_record) = existing_schema {
                match installed_schema_record.as_installed_schema()? {
                    InstalledPkgAssetTyped::Schema { id, .. } => {
                        match Schema::get_by_id(ctx, &id).await? {
                            Some(schema) => Some(schema),
                            None => return Err(PkgError::InstalledSchemaMissing(id)),
                        }
                    }
                    _ => unreachable!(),
                }
            } else {
                match schema_spec
                    .unique_id()
                    .and_then(|unique_id| thing_map.get(change_set_pk, &unique_id.to_owned()))
                {
                    Some(Thing::Schema(schema)) => {
                        let mut schema = schema.to_owned();

                        if schema_spec.deleted() {
                            schema.delete_by_id(ctx).await?;
                            // delete all schema children?

                            None
                        } else {
                            if let Some(data) = schema_spec.data() {
                                update_schema(ctx, &mut schema, data).await?;
                            }

                            Some(schema)
                        }
                    }
                    _ => {
                        if schema_spec.deleted() {
                            None
                        } else {
                            Some(
                                create_schema(
                                    ctx,
                                    schema_spec
                                        .data()
                                        .ok_or(PkgError::DataNotFound("schema".into()))?,
                                )
                                .await?,
                            )
                        }
                    }
                }
            }
        }
    };

    if let Some(mut schema) = schema {
        // Even if the asset is already installed, we write a record of the asset
        // installation so that we can track the installed packages that share schemas.
        if let Some(installed_pkg_id) = installed_pkg_id {
            InstalledPkgAsset::new(
                ctx,
                InstalledPkgAssetTyped::new_for_schema(*schema.id(), installed_pkg_id, hash),
            )
            .await?;
        }

        if let Some(unique_id) = schema_spec.unique_id() {
            thing_map.insert(
                change_set_pk,
                unique_id.to_owned(),
                Thing::Schema(schema.to_owned()),
            );
        }

        let mut installed_schema_variant_ids = vec![];
        for variant_spec in &schema_spec.variants()? {
            let variant = import_schema_variant(
                ctx,
                change_set_pk,
                &mut schema,
                variant_spec,
                installed_pkg_id,
                thing_map,
                metadata,
            )
            .await?;

            if let Some(variant) = variant {
                installed_schema_variant_ids.push(*variant.id());

                let schema_default_variant_id = schema_spec
                    .data()
                    .as_ref()
                    .and_then(|data| data.default_schema_variant());

                set_default_schema_variant_id(
                    ctx,
                    &mut schema,
                    schema_default_variant_id,
                    variant_spec.unique_id(),
                    *variant.id(),
                )
                .await?;

                if let Some(variant_spec_data) = variant_spec.data() {
                    let func_unique_id = variant_spec_data.func_unique_id().to_owned();

                    if let Thing::Func(asset_func) =
                        thing_map
                            .get(change_set_pk, &func_unique_id)
                            .ok_or(PkgError::MissingFuncUniqueId(func_unique_id.to_string()))?
                    {
                        create_schema_variant_definition(
                            ctx,
                            schema_spec.clone(),
                            installed_pkg_id,
                            *variant.id(),
                            asset_func,
                        )
                        .await?;
                    }
                }
            }
        }

        Ok((Some(*schema.id()), installed_schema_variant_ids))
    } else {
        Ok((None, vec![]))
    }
}

async fn set_default_schema_variant_id(
    ctx: &DalContext,
    schema: &mut Schema,
    spec_default_unique_id: Option<&str>,
    variant_unique_id: Option<&str>,
    variant_id: SchemaVariantId,
) -> PkgResult<()> {
    match (variant_unique_id, spec_default_unique_id) {
        (None, _) | (Some(_), None) => {
            if schema.default_schema_variant_id().is_none() {
                schema
                    .set_default_schema_variant_id(ctx, Some(variant_id))
                    .await?;
            }
        }
        (Some(variant_unique_id), Some(spec_default_unique_id)) => {
            if variant_unique_id == spec_default_unique_id {
                let current_default_variant_id = schema
                    .default_schema_variant_id()
                    .copied()
                    .unwrap_or(SchemaVariantId::NONE);

                if variant_id != current_default_variant_id {
                    schema
                        .set_default_schema_variant_id(ctx, Some(variant_id))
                        .await?;
                }
            }
        }
    }

    Ok(())
}

async fn create_schema_variant_definition(
    ctx: &DalContext,
    schema_spec: SiPkgSchema<'_>,
    installed_pkg_id: Option<InstalledPkgId>,
    schema_variant_id: SchemaVariantId,
    asset_func: &Func,
) -> PkgResult<()> {
    let hash = schema_spec.hash().to_string();
    let existing_definition = InstalledPkgAsset::list_for_kind_and_hash(
        ctx,
        InstalledPkgAssetKind::SchemaVariantDefinition,
        &hash,
    )
    .await?
    .pop();

    let definition = match existing_definition {
        None => {
            let maybe_schema_variant_definition =
                SchemaVariantDefinition::get_by_func_id(ctx, *asset_func.id()).await?;
            let mut schema_variant_definition = match maybe_schema_variant_definition {
                None => {
                    let spec = schema_spec.to_spec().await?;
                    let metadata = SchemaVariantDefinitionJson::metadata_from_spec(spec)?;

                    let mut svd = SchemaVariantDefinition::new(
                        ctx,
                        metadata.name,
                        metadata.menu_name,
                        metadata.category,
                        metadata.link,
                        metadata.color,
                        metadata.component_kind,
                        metadata.description,
                        *asset_func.id(),
                    )
                    .await?;

                    svd.set_component_type(ctx, metadata.component_type).await?;

                    svd
                }
                Some(schema_variant_definition) => schema_variant_definition,
            };

            schema_variant_definition
                .set_schema_variant_id(ctx, Some(schema_variant_id))
                .await?;

            schema_variant_definition
        }
        Some(existing_definition) => {
            match existing_definition.as_installed_schema_variant_definition()? {
                InstalledPkgAssetTyped::SchemaVariantDefinition { id, .. } => {
                    match SchemaVariantDefinition::get_by_id(ctx, &id).await? {
                        Some(definition) => definition,
                        None => return Err(PkgError::InstalledSchemaVariantDefinitionMissing(id)),
                    }
                }
                _ => unreachable!(
                    "we are protected by the as_installed_schema_variant_definition method"
                ),
            }
        }
    };

    if let Some(installed_pkg_id) = installed_pkg_id {
        InstalledPkgAsset::new(
            ctx,
            InstalledPkgAssetTyped::new_for_schema_variant_definition(
                *definition.id(),
                installed_pkg_id,
                hash,
            ),
        )
        .await?;
    }

    Ok(())
}

#[derive(Clone, Debug)]
struct AttrFuncInfo {
    func_unique_id: String,
    prop_id: PropId,
    inputs: Vec<SiPkgAttrFuncInputView>,
}

#[remain::sorted]
#[derive(Clone, Debug)]
enum DefaultValueInfo {
    Boolean {
        prop_id: PropId,
        default_value: bool,
    },
    Number {
        prop_id: PropId,
        default_value: i64,
    },
    String {
        prop_id: PropId,
        default_value: String,
    },
}

struct PropVisitContext<'a> {
    pub ctx: &'a DalContext,
    pub schema_variant_id: SchemaVariantId,
    pub attr_funcs: Mutex<Vec<AttrFuncInfo>>,
    pub default_values: Mutex<Vec<DefaultValueInfo>>,
    pub map_key_funcs: Mutex<Vec<(String, AttrFuncInfo)>>,
}

async fn import_leaf_function(
    ctx: &DalContext,
    change_set_pk: ChangeSetPk,
    leaf_func: SiPkgLeafFunction<'_>,
    schema_variant_id: SchemaVariantId,
    thing_map: &mut ThingMap,
) -> PkgResult<()> {
    let inputs: Vec<LeafInputLocation> = leaf_func
        .inputs()
        .iter()
        .map(|input| input.into())
        .collect();

    let kind: LeafKind = leaf_func.leaf_kind().into();

    match thing_map.get(change_set_pk, &leaf_func.func_unique_id().to_owned()) {
        Some(Thing::Func(func)) => {
            SchemaVariant::upsert_leaf_function(ctx, schema_variant_id, None, kind, &inputs, func)
                .await?;
        }
        _ => {
            return Err(PkgError::MissingFuncUniqueId(
                leaf_func.func_unique_id().to_string(),
            ));
        }
    }

    Ok(())
}

// TODO: cache this so we don't fetch it for every socket
async fn get_identity_func(
    ctx: &DalContext,
) -> PkgResult<(Func, FuncBinding, FuncBindingReturnValue, FuncArgument)> {
    let func_name = "si:identity";
    let func_argument_name = "identity";
    let func: Func = Func::find_by_name(ctx, func_name)
        .await?
        .ok_or_else(|| FuncError::NotFoundByName(func_name.to_string()))?;

    let func_id = *func.id();
    let (func_binding, func_binding_return_value) = FuncBinding::create_and_execute(
        ctx,
        serde_json::json![{ "identity": null }],
        func_id,
        vec![],
    )
    .await?;
    let func_argument = FuncArgument::find_by_name_for_func(ctx, func_argument_name, func_id)
        .await?
        .ok_or_else(|| {
            PkgError::MissingIntrinsicFuncArgument(
                func_name.to_string(),
                func_argument_name.to_string(),
            )
        })?;

    Ok((func, func_binding, func_binding_return_value, func_argument))
}

async fn create_socket(
    ctx: &DalContext,
    data: &SiPkgSocketData,
    schema_id: SchemaId,
    schema_variant_id: SchemaVariantId,
) -> PkgResult<(Socket, Option<InternalProvider>, Option<ExternalProvider>)> {
    let (identity_func, identity_func_binding, identity_fbrv, _) = get_identity_func(ctx).await?;

    let (mut socket, ip, ep) = match data.kind() {
        SocketSpecKind::Input => {
            let (ip, socket) = InternalProvider::new_explicit_with_socket(
                ctx,
                schema_variant_id,
                data.name(),
                *identity_func.id(),
                *identity_func_binding.id(),
                *identity_fbrv.id(),
                data.connection_annotations(),
                data.arity().into(),
                false,
            )
            .await?;

            (socket, Some(ip), None)
        }
        SocketSpecKind::Output => {
            let (ep, socket) = ExternalProvider::new_with_socket(
                ctx,
                schema_id,
                schema_variant_id,
                data.name(),
                None,
                *identity_func.id(),
                *identity_func_binding.id(),
                *identity_fbrv.id(),
                data.connection_annotations(),
                data.arity().into(),
                false,
            )
            .await?;

            (socket, None, Some(ep))
        }
    };

    socket.set_ui_hidden(ctx, data.ui_hidden()).await?;

    Ok((socket, ip, ep))
}

async fn import_socket(
    ctx: &DalContext,
    change_set_pk: ChangeSetPk,
    socket_spec: SiPkgSocket<'_>,
    schema_id: SchemaId,
    schema_variant_id: SchemaVariantId,
    thing_map: &mut ThingMap,
) -> PkgResult<()> {
    let (socket, ip, ep) = {
        match socket_spec
            .unique_id()
            .and_then(|unique_id| thing_map.get(change_set_pk, &unique_id.to_owned()))
        {
            Some(Thing::Socket(socket_box)) => {
                (
                    socket_box.0.to_owned(),
                    socket_box.1.to_owned(),
                    socket_box.2.to_owned(),
                )
                // prop trees, including sockets and providers, are created whole cloth, so
                // should not have differences in change sets (currently)
            }
            _ => {
                let data = socket_spec
                    .data()
                    .ok_or(PkgError::DataNotFound(socket_spec.name().into()))?;

                create_socket(ctx, data, schema_id, schema_variant_id).await?
            }
        }
    };

    if let Some(unique_id) = socket_spec.unique_id() {
        thing_map.insert(
            change_set_pk,
            unique_id.to_owned(),
            Thing::Socket(Box::new((socket, ip.to_owned(), ep.to_owned()))),
        );
    }

    match (
        socket_spec.data().and_then(|data| data.func_unique_id()),
        ep,
        ip,
    ) {
        (Some(func_unique_id), Some(ep), None) => {
            import_attr_func_for_output_socket(
                ctx,
                change_set_pk,
                schema_variant_id,
                *ep.id(),
                func_unique_id,
                socket_spec.inputs()?.drain(..).map(Into::into).collect(),
                thing_map,
            )
            .await?;
        }
        (Some(_), _, Some(_)) => {}
        _ => {}
    }

    Ok(())
}

async fn create_action_prototype(
    ctx: &DalContext,
    action_func_spec: &SiPkgActionFunc<'_>,
    func_id: FuncId,
    schema_variant_id: SchemaVariantId,
) -> PkgResult<ActionPrototype> {
    let mut proto = ActionPrototype::new(
        ctx,
        func_id,
        action_func_spec.kind().into(),
        ActionPrototypeContext { schema_variant_id },
    )
    .await?;

    if let Some(name) = action_func_spec.name() {
        proto.set_name(ctx, Some(name)).await?;
    }

    Ok(proto)
}

async fn create_authentication_prototype(
    ctx: &DalContext,
    func_id: FuncId,
    schema_variant_id: SchemaVariantId,
) -> PkgResult<AuthenticationPrototype> {
    Ok(AuthenticationPrototype::new(
        ctx,
        func_id,
        AuthenticationPrototypeContext { schema_variant_id },
    )
    .await?)
}

async fn update_action_prototype(
    ctx: &DalContext,
    prototype: &mut ActionPrototype,
    action_func_spec: &SiPkgActionFunc<'_>,
    func_id: FuncId,
    schema_variant_id: SchemaVariantId,
) -> PkgResult<()> {
    if prototype.schema_variant_id() != schema_variant_id {
        prototype
            .set_schema_variant_id(ctx, schema_variant_id)
            .await?;
    }

    if prototype.name() != action_func_spec.name() {
        prototype.set_name(ctx, action_func_spec.name()).await?;
    }

    if prototype.func_id() != func_id {
        prototype.set_func_id(ctx, func_id).await?;
    }

    let kind: ActionKind = action_func_spec.kind().into();
    if *prototype.kind() != kind {
        prototype.set_kind(ctx, kind).await?;
    }

    Ok(())
}

async fn update_authentication_prototype(
    ctx: &DalContext,
    prototype: &mut AuthenticationPrototype,
    func_id: FuncId,
    schema_variant_id: SchemaVariantId,
) -> PkgResult<()> {
    if prototype.schema_variant_id() != schema_variant_id {
        prototype
            .set_schema_variant_id(ctx, schema_variant_id)
            .await?;
    }

    if prototype.func_id() != func_id {
        prototype.set_func_id(ctx, func_id).await?;
    }

    Ok(())
}

async fn import_action_func(
    ctx: &DalContext,
    change_set_pk: ChangeSetPk,
    action_func_spec: &SiPkgActionFunc<'_>,
    schema_variant_id: SchemaVariantId,
    thing_map: &ThingMap,
) -> PkgResult<Option<ActionPrototype>> {
    let prototype =
        match thing_map.get(change_set_pk, &action_func_spec.func_unique_id().to_owned()) {
            Some(Thing::Func(func)) => {
                let func_id = *func.id();

                if let Some(unique_id) = action_func_spec.unique_id() {
                    match thing_map.get(change_set_pk, &unique_id.to_owned()) {
                        Some(Thing::ActionPrototype(prototype)) => {
                            let mut prototype = prototype.to_owned();

                            if action_func_spec.deleted() {
                                prototype.delete_by_id(ctx).await?;
                            } else {
                                update_action_prototype(
                                    ctx,
                                    &mut prototype,
                                    action_func_spec,
                                    func_id,
                                    schema_variant_id,
                                )
                                .await?;
                            }

                            Some(prototype)
                        }
                        _ => {
                            if action_func_spec.deleted() {
                                None
                            } else {
                                Some(
                                    create_action_prototype(
                                        ctx,
                                        action_func_spec,
                                        func_id,
                                        schema_variant_id,
                                    )
                                    .await?,
                                )
                            }
                        }
                    }
                } else {
                    Some(
                        create_action_prototype(ctx, action_func_spec, func_id, schema_variant_id)
                            .await?,
                    )
                }
            }
            _ => {
                return Err(PkgError::MissingFuncUniqueId(
                    action_func_spec.func_unique_id().into(),
                ));
            }
        };

    Ok(prototype)
}

async fn import_auth_func(
    ctx: &DalContext,
    change_set_pk: ChangeSetPk,
    func_spec: &SiPkgAuthFunc<'_>,
    schema_variant_id: SchemaVariantId,
    thing_map: &ThingMap,
) -> PkgResult<Option<AuthenticationPrototype>> {
    let prototype = match thing_map.get(change_set_pk, &func_spec.func_unique_id().to_owned()) {
        Some(Thing::Func(func)) => {
            let func_id = *func.id();

            if let Some(unique_id) = func_spec.unique_id() {
                match thing_map.get(change_set_pk, &unique_id.to_owned()) {
                    Some(Thing::AuthPrototype(prototype)) => {
                        let mut prototype = prototype.to_owned();

                        if func_spec.deleted() {
                            prototype.delete_by_id(ctx).await?;
                        } else {
                            update_authentication_prototype(
                                ctx,
                                &mut prototype,
                                func_id,
                                schema_variant_id,
                            )
                            .await?;
                        }

                        Some(prototype)
                    }
                    _ => {
                        if func_spec.deleted() {
                            None
                        } else {
                            Some(
                                create_authentication_prototype(ctx, func_id, schema_variant_id)
                                    .await?,
                            )
                        }
                    }
                }
            } else {
                Some(create_authentication_prototype(ctx, func_id, schema_variant_id).await?)
            }
        }
        _ => {
            return Err(PkgError::MissingFuncUniqueId(
                func_spec.func_unique_id().into(),
            ));
        }
    };

    Ok(prototype)
}

#[derive(Default, Clone, Debug)]
struct CreatePropsSideEffects {
    attr_funcs: Vec<AttrFuncInfo>,
    default_values: Vec<DefaultValueInfo>,
    map_key_funcs: Vec<(String, AttrFuncInfo)>,
}

impl IntoIterator for CreatePropsSideEffects {
    type Item = CreatePropsSideEffects;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        vec![self].into_iter()
    }
}

impl Extend<CreatePropsSideEffects> for CreatePropsSideEffects {
    fn extend<T: IntoIterator<Item = CreatePropsSideEffects>>(&mut self, iter: T) {
        for element in iter {
            self.attr_funcs.extend(element.attr_funcs);
            self.default_values.extend(element.default_values);
            self.map_key_funcs.extend(element.map_key_funcs);
        }
    }
}

async fn create_props(
    ctx: &DalContext,
    variant_spec: &SiPkgSchemaVariant<'_>,
    prop_root: SchemaVariantSpecPropRoot,
    prop_root_prop_id: PropId,
    schema_variant_id: SchemaVariantId,
) -> PkgResult<CreatePropsSideEffects> {
    let context = PropVisitContext {
        ctx,
        schema_variant_id,
        attr_funcs: Mutex::new(vec![]),
        default_values: Mutex::new(vec![]),
        map_key_funcs: Mutex::new(vec![]),
    };

    let parent_info = (prop_root_prop_id, PropPath::new(prop_root.path_parts()));

    variant_spec
        .visit_prop_tree(prop_root, create_prop, Some(parent_info), &context)
        .await?;

    Ok(CreatePropsSideEffects {
        attr_funcs: context.attr_funcs.into_inner(),
        default_values: context.default_values.into_inner(),
        map_key_funcs: context.map_key_funcs.into_inner(),
    })
}

async fn update_schema_variant(
    ctx: &DalContext,
    schema_variant: &mut SchemaVariant,
    name: &str,
    schema_id: SchemaId,
) -> PkgResult<()> {
    let current_schema_id = schema_variant
        .schema(ctx)
        .await?
        .map(|schema| *schema.id())
        .ok_or(SchemaVariantError::MissingSchema(*schema_variant.id()))?;

    if schema_id != current_schema_id {
        schema_variant.set_schema(ctx, &schema_id).await?;
    }

    if schema_variant.name() != name {
        schema_variant.set_name(ctx, name).await?;
    }

    Ok(())
}

/// Duplicate all the functions, and return a thing_map with them included, so
/// that we can import a standalone schema variant.
pub async fn clone_and_import_funcs(ctx: &DalContext, funcs: Vec<FuncSpec>) -> PkgResult<ThingMap> {
    let mut thing_map = ThingMap::new();

    for func_spec in funcs {
        let func = if func::is_intrinsic(&func_spec.name)
            || SPECIAL_CASE_FUNCS.contains(&func_spec.name.as_str())
        {
            Func::find_by_name(ctx, &func_spec.name)
                .await?
                .ok_or(PkgError::MissingIntrinsicFunc(func_spec.name.to_owned()))?
        } else {
            let func = create_func(ctx, &func_spec).await?;

            if !func_spec.arguments.is_empty() {
                import_func_arguments(
                    ctx,
                    ChangeSetPk::NONE,
                    *func.id(),
                    &func_spec.arguments,
                    &mut thing_map,
                )
                .await?;
            }

            func
        };

        thing_map.insert(ChangeSetPk::NONE, func_spec.unique_id, Thing::Func(func));
    }

    Ok(thing_map)
}

pub async fn import_schema_variant(
    ctx: &DalContext,
    change_set_pk: ChangeSetPk,
    schema: &mut Schema,
    variant_spec: &SiPkgSchemaVariant<'_>,
    installed_pkg_id: Option<InstalledPkgId>,
    thing_map: &mut ThingMap,
    metadata: &SiPkgMetadata,
) -> PkgResult<Option<SchemaVariant>> {
    // Builtin variants should already be imported, so we can just find it and
    // return it.  However, what we *should* do is check to see if the installed
    // variant is the one we want, and if not, find the missing builtin variant
    // on the remote module index, and then import that.
    if metadata.kind() == SiPkgKind::WorkspaceBackup && variant_spec.is_builtin() {
        let variant = schema
            .find_variant_by_name(ctx, variant_spec.name())
            .await?
            .ok_or(PkgError::MissingBuiltinSchemaVariant(
                variant_spec.name().to_owned(),
            ))?;

        return Ok(Some(variant));
    }

    let hash = variant_spec.hash().to_string();
    let mut schema_variant = {
        let existing_schema_variant = InstalledPkgAsset::list_for_kind_and_hash(
            ctx,
            InstalledPkgAssetKind::SchemaVariant,
            &hash,
        )
        .await?
        .pop();

        if let Some(installed_sv_record) = existing_schema_variant {
            match installed_sv_record.as_installed_schema_variant()? {
                InstalledPkgAssetTyped::SchemaVariant { id, .. } => {
                    SchemaVariant::get_by_id(ctx, &id).await?
                }
                _ => unreachable!(
                    "the as_installed_schema_variant method ensures we cannot hit this branch"
                ),
            }
        } else {
            match variant_spec
                .unique_id()
                .and_then(|unique_id| thing_map.get(change_set_pk, &unique_id.to_owned()))
            {
                Some(Thing::SchemaVariant(variant)) => {
                    let mut variant = variant.to_owned();
                    update_schema_variant(ctx, &mut variant, variant_spec.name(), *schema.id())
                        .await?;

                    if variant_spec.deleted() {
                        variant.delete_by_id(ctx).await?;

                        None
                    } else {
                        Some(variant)
                    }
                }
                _ => {
                    if variant_spec.deleted() {
                        None
                    } else {
                        let mut variant =
                            SchemaVariant::new(ctx, *schema.id(), variant_spec.name())
                                .await?
                                .0;

                        if matches!(metadata.kind(), SiPkgKind::Module) {
                            variant
                                .set_pkg_created_at(ctx, Some(metadata.created_at()))
                                .await?;
                        }

                        Some(variant)
                    }
                }
            }
        }
    };

    if let Some(schema_variant) = schema_variant.as_mut() {
        if let Some(installed_pkg_id) = installed_pkg_id {
            InstalledPkgAsset::new(
                ctx,
                InstalledPkgAssetTyped::new_for_schema_variant(
                    *schema_variant.id(),
                    installed_pkg_id,
                    hash,
                ),
            )
            .await?;
        }

        if let Some(unique_id) = variant_spec.unique_id() {
            thing_map.insert(
                change_set_pk,
                unique_id.to_owned(),
                Thing::SchemaVariant(schema_variant.to_owned()),
            );
        }

        if let Some(data) = variant_spec.data() {
            if let (Some(spec_color), current_color) =
                (data.color(), schema_variant.color(ctx).await?)
            {
                if current_color.is_none()
                    || spec_color
                        != current_color.expect("is none condition ensures this won't panic")
                {
                    schema_variant.set_color(ctx, spec_color.to_owned()).await?;
                }
            }
        }

        let mut side_effects = CreatePropsSideEffects::default();

        let domain_prop_id = schema_variant
            .find_prop(ctx, &["root", "domain"])
            .await?
            .id()
            .to_owned();

        side_effects.extend(
            create_props(
                ctx,
                variant_spec,
                SchemaVariantSpecPropRoot::Domain,
                domain_prop_id,
                *schema_variant.id(),
            )
            .await?,
        );

        let secrets_prop_id = schema_variant
            .find_prop(ctx, &["root", "secrets"])
            .await?
            .id()
            .to_owned();

        side_effects.extend(
            create_props(
                ctx,
                variant_spec,
                SchemaVariantSpecPropRoot::Secrets,
                secrets_prop_id,
                *schema_variant.id(),
            )
            .await?,
        );

        if !variant_spec.secret_definitions()?.is_empty() {
            let secret_definition_prop_id = *Prop::new_without_ui_optionals(
                ctx,
                "secret_definition",
                PropKind::Object,
                *schema_variant.id(),
                Some(*schema_variant.find_prop(ctx, &["root"]).await?.id()),
            )
            .await?
            .id();

            side_effects.extend(
                create_props(
                    ctx,
                    variant_spec,
                    SchemaVariantSpecPropRoot::SecretDefinition,
                    secret_definition_prop_id,
                    *schema_variant.id(),
                )
                .await?,
            );
        }

        match schema_variant
            .find_prop(ctx, &["root", "resource_value"])
            .await
        {
            Ok(resource_value_prop) => {
                side_effects.extend(
                    create_props(
                        ctx,
                        variant_spec,
                        SchemaVariantSpecPropRoot::ResourceValue,
                        *resource_value_prop.id(),
                        *schema_variant.id(),
                    )
                    .await?,
                );
            }
            Err(SchemaVariantError::PropNotFoundAtPath(_, _, _)) => {
                warn!("Cannot find /root/resource_value prop, so skipping creating props under the resource value. If the /root/resource_value pr has been merged, this should be an error!");
            }
            Err(err) => Err(err)?,
        };

        if let Some(data) = variant_spec.data() {
            schema_variant
                .finalize(ctx, Some(data.component_type().into()))
                .await?;
        }

        for action_func in &variant_spec.action_funcs()? {
            let prototype = import_action_func(
                ctx,
                change_set_pk,
                action_func,
                *schema_variant.id(),
                thing_map,
            )
            .await?;

            if let (Some(prototype), Some(unique_id)) = (prototype, action_func.unique_id()) {
                thing_map.insert(
                    change_set_pk,
                    unique_id.to_owned(),
                    Thing::ActionPrototype(prototype),
                );
            }
        }

        for func in &variant_spec.auth_funcs()? {
            let prototype =
                import_auth_func(ctx, change_set_pk, func, *schema_variant.id(), thing_map).await?;

            if let (Some(prototype), Some(unique_id)) = (prototype, func.unique_id()) {
                thing_map.insert(
                    change_set_pk,
                    unique_id.to_owned(),
                    Thing::AuthPrototype(prototype),
                );
            }
        }

        for leaf_func in variant_spec.leaf_functions()? {
            import_leaf_function(
                ctx,
                change_set_pk,
                leaf_func,
                *schema_variant.id(),
                thing_map,
            )
            .await?;
        }

        for socket in variant_spec.sockets()? {
            import_socket(
                ctx,
                change_set_pk,
                socket,
                *schema.id(),
                *schema_variant.id(),
                thing_map,
            )
            .await?;
        }

        // Default values must be set before attribute functions are configured so they don't
        // override the prototypes set there
        for default_value_info in side_effects.default_values {
            set_default_value(ctx, default_value_info).await?;
        }

        // Set a default name value for all name props, this ensures region has a name before
        // the function is executed
        {
            let name_prop = schema_variant
                .find_prop(ctx, &["root", "si", "name"])
                .await?;
            let name_default_value_info = DefaultValueInfo::String {
                prop_id: *name_prop.id(),
                default_value: schema.name().to_lowercase(),
            };

            set_default_value(ctx, name_default_value_info).await?;
        }

        for si_prop_func in variant_spec.si_prop_funcs()? {
            let prop = schema_variant
                .find_prop(ctx, &si_prop_func.kind().prop_path())
                .await?;
            import_attr_func_for_prop(
                ctx,
                change_set_pk,
                *schema_variant.id(),
                AttrFuncInfo {
                    func_unique_id: si_prop_func.func_unique_id().to_owned(),
                    prop_id: *prop.id(),
                    inputs: si_prop_func
                        .inputs()?
                        .iter()
                        .map(|input| input.to_owned().into())
                        .collect(),
                },
                None,
                thing_map,
            )
            .await?;
        }

        let mut has_resource_value_func = false;
        for root_prop_func in variant_spec.root_prop_funcs()? {
            if root_prop_func.prop() == SchemaVariantSpecPropRoot::ResourceValue {
                has_resource_value_func = true;
            }

            let prop = schema_variant
                .find_prop(ctx, root_prop_func.prop().path_parts())
                .await?;
            import_attr_func_for_prop(
                ctx,
                change_set_pk,
                *schema_variant.id(),
                AttrFuncInfo {
                    func_unique_id: root_prop_func.func_unique_id().to_owned(),
                    prop_id: *prop.id(),
                    inputs: root_prop_func
                        .inputs()?
                        .iter()
                        .map(|input| input.to_owned().into())
                        .collect(),
                },
                None,
                thing_map,
            )
            .await?;
        }
        if !has_resource_value_func {
            attach_resource_payload_to_value(ctx, *schema_variant.id()).await?;
        }

        for attr_func in side_effects.attr_funcs {
            import_attr_func_for_prop(
                ctx,
                change_set_pk,
                *schema_variant.id(),
                attr_func,
                None,
                thing_map,
            )
            .await?;
        }

        for (key, map_key_func) in side_effects.map_key_funcs {
            import_attr_func_for_prop(
                ctx,
                change_set_pk,
                *schema_variant.id(),
                map_key_func,
                Some(key),
                thing_map,
            )
            .await?;
        }
    }

    Ok(schema_variant)
}

pub async fn attach_resource_payload_to_value(
    ctx: &DalContext,
    schema_variant_id: SchemaVariantId,
) -> PkgResult<()> {
    let func_id = *Func::find_by_name(ctx, "si:resourcePayloadToValue")
        .await?
        .ok_or(FuncError::NotFoundByName(
            "si:resourcePayloadToValue".into(),
        ))?
        .id();

    let func_argument_id = *FuncArgument::find_by_name_for_func(ctx, "payload", func_id)
        .await?
        .ok_or(FuncArgumentError::NotFoundByNameForFunc(
            "payload".into(),
            func_id,
        ))?
        .id();

    let source = {
        let prop = SchemaVariant::find_prop_in_tree(
            ctx,
            schema_variant_id,
            &["root", "resource", "payload"],
        )
        .await?;

        InternalProvider::find_for_prop(ctx, *prop.id())
            .await?
            .ok_or(InternalProviderError::NotFoundForProp(*prop.id()))?
    };

    let target = {
        let resource_value_prop =
            SchemaVariant::find_prop_in_tree(ctx, schema_variant_id, &["root", "resource_value"])
                .await?;

        let mut prototype = AttributeValue::find_for_context(
            ctx,
            AttributeReadContext::default_with_prop(*resource_value_prop.id()),
        )
        .await?
        .ok_or(AttributeValueError::Missing)?
        .attribute_prototype(ctx)
        .await?
        .ok_or(AttributeValueError::MissingAttributePrototype)?;

        prototype.set_func_id(ctx, func_id).await?;

        prototype
    };

    match AttributePrototypeArgument::list_for_attribute_prototype(ctx, *target.id())
        .await?
        .iter()
        .find(|apa| apa.func_argument_id() == func_argument_id)
    {
        Some(apa) => {
            if apa.internal_provider_id() != *source.id() {
                let mut apa = apa.to_owned();
                apa.set_internal_provider_id(ctx, *source.id()).await?;
            }
        }
        None => {
            AttributePrototypeArgument::new_for_intra_component(
                ctx,
                *target.id(),
                func_argument_id,
                *source.id(),
            )
            .await?;
        }
    }
    Ok(())
}

async fn set_default_value(
    ctx: &DalContext,
    default_value_info: DefaultValueInfo,
) -> PkgResult<()> {
    let prop = match &default_value_info {
        DefaultValueInfo::Number { prop_id, .. }
        | DefaultValueInfo::String { prop_id, .. }
        | DefaultValueInfo::Boolean { prop_id, .. } => Prop::get_by_id(ctx, prop_id)
            .await?
            .ok_or(PkgError::MissingProp(*prop_id))?,
    };

    match default_value_info {
        DefaultValueInfo::Boolean { default_value, .. } => {
            prop.set_default_value(ctx, default_value).await?
        }
        DefaultValueInfo::Number { default_value, .. } => {
            prop.set_default_value(ctx, default_value).await?
        }
        DefaultValueInfo::String { default_value, .. } => {
            prop.set_default_value(ctx, default_value).await?
        }
    }

    Ok(())
}

async fn import_attr_func_for_prop(
    ctx: &DalContext,
    change_set_pk: ChangeSetPk,
    schema_variant_id: SchemaVariantId,
    AttrFuncInfo {
        func_unique_id,
        prop_id,
        inputs,
    }: AttrFuncInfo,
    key: Option<String>,
    thing_map: &mut ThingMap,
) -> PkgResult<()> {
    match thing_map.get(change_set_pk, &func_unique_id.to_owned()) {
        Some(Thing::Func(func)) => {
            import_attr_func(
                ctx,
                change_set_pk,
                AttributeReadContext {
                    prop_id: Some(prop_id),
                    ..Default::default()
                },
                key,
                schema_variant_id,
                *func.id(),
                inputs,
                thing_map,
            )
            .await?;
        }
        _ => return Err(PkgError::MissingFuncUniqueId(func_unique_id.to_string())),
    }

    Ok(())
}

async fn import_attr_func_for_output_socket(
    ctx: &DalContext,
    change_set_pk: ChangeSetPk,
    schema_variant_id: SchemaVariantId,
    external_provider_id: ExternalProviderId,
    func_unique_id: &str,
    inputs: Vec<SiPkgAttrFuncInputView>,
    thing_map: &mut ThingMap,
) -> PkgResult<()> {
    match thing_map.get(change_set_pk, &func_unique_id.to_owned()) {
        Some(Thing::Func(func)) => {
            import_attr_func(
                ctx,
                change_set_pk,
                AttributeReadContext {
                    external_provider_id: Some(external_provider_id),
                    ..Default::default()
                },
                None,
                schema_variant_id,
                *func.id(),
                inputs,
                thing_map,
            )
            .await?;
        }
        _ => return Err(PkgError::MissingFuncUniqueId(func_unique_id.to_string())),
    }

    Ok(())
}

async fn get_prototype_for_context(
    ctx: &DalContext,
    context: AttributeReadContext,
    key: Option<String>,
) -> PkgResult<AttributePrototype> {
    let value = AttributeValue::find_for_context(ctx, context)
        .await?
        .ok_or(AttributeValueError::Missing)?;

    let real_value = if let Some(key) = key {
        let parent_prop_id = context
            .prop_id()
            .ok_or(PkgError::AttributeFuncForKeyMissingProp(
                context,
                key.to_owned(),
            ))?;

        let parent_prop = Prop::get_by_id(ctx, &parent_prop_id)
            .await?
            .ok_or(PkgError::MissingProp(parent_prop_id))?;

        if *parent_prop.kind() != PropKind::Map {
            return Err(PkgError::AttributeFuncForKeySetOnWrongKind(
                parent_prop_id,
                key,
                *parent_prop.kind(),
            ));
        }

        match parent_prop.child_props(ctx).await?.pop() {
            Some(item_prop) => {
                let item_write_context = AttributeContextBuilder::new()
                    .set_prop_id(*item_prop.id())
                    .to_context()?;

                let item_read_context: AttributeReadContext = item_write_context.to_owned().into();

                match AttributeValue::find_with_parent_and_key_for_context(
                    ctx,
                    Some(*value.id()),
                    Some(key.to_owned()),
                    item_read_context,
                )
                .await?
                {
                    Some(item_av) => item_av,
                    None => {
                        let item_id = AttributeValue::insert_for_context(
                            ctx,
                            item_write_context,
                            *value.id(),
                            None,
                            Some(key),
                        )
                        .await?;

                        AttributeValue::get_by_id(ctx, &item_id)
                            .await?
                            .ok_or(AttributeValueError::MissingForId(item_id))?
                    }
                }
            }
            None => {
                return Err(PkgError::MissingItemPropForMapProp(parent_prop_id));
            }
        }
    } else {
        value
    };

    Ok(real_value
        .attribute_prototype(ctx)
        .await?
        .ok_or(AttributeValueError::MissingAttributePrototype)?)
}

async fn create_attr_proto_arg(
    ctx: &DalContext,
    prototype_id: AttributePrototypeId,
    input: &SiPkgAttrFuncInputView,
    func_id: FuncId,
    schema_variant_id: SchemaVariantId,
) -> PkgResult<AttributePrototypeArgument> {
    let arg = match &input {
        SiPkgAttrFuncInputView::Prop { name, .. }
        | SiPkgAttrFuncInputView::InputSocket { name, .. }
        | SiPkgAttrFuncInputView::OutputSocket { name, .. } => {
            FuncArgument::find_by_name_for_func(ctx, name, func_id)
                .await?
                .ok_or(PkgError::MissingFuncArgument(name.to_owned(), func_id))?
        }
    };

    Ok(match input {
        SiPkgAttrFuncInputView::Prop { prop_path, .. } => {
            let prop = Prop::find_prop_by_path(ctx, schema_variant_id, &prop_path.into()).await?;
            let prop_ip = InternalProvider::find_for_prop(ctx, *prop.id())
                .await?
                .ok_or(PkgError::MissingInternalProviderForProp(*prop.id()))?;

            AttributePrototypeArgument::new_for_intra_component(
                ctx,
                prototype_id,
                *arg.id(),
                *prop_ip.id(),
            )
            .await?
        }
        SiPkgAttrFuncInputView::InputSocket { socket_name, .. } => {
            let explicit_ip = InternalProvider::find_explicit_for_schema_variant_and_name(
                ctx,
                schema_variant_id,
                &socket_name,
            )
            .await?
            .ok_or(PkgError::MissingInternalProviderForSocketName(
                socket_name.to_owned(),
            ))?;

            AttributePrototypeArgument::new_for_intra_component(
                ctx,
                prototype_id,
                *arg.id(),
                *explicit_ip.id(),
            )
            .await?
        }
        _ => {
            // xxx: make this an error
            panic!("unsupported taking external provider as input for prop");
        }
    })
}

async fn update_attr_proto_arg(
    ctx: &DalContext,
    apa: &mut AttributePrototypeArgument,
    _prototype_id: AttributePrototypeId,
    input: &SiPkgAttrFuncInputView,
    func_id: FuncId,
    schema_variant_id: SchemaVariantId,
) -> PkgResult<()> {
    let arg = match &input {
        SiPkgAttrFuncInputView::Prop { name, .. }
        | SiPkgAttrFuncInputView::InputSocket { name, .. }
        | SiPkgAttrFuncInputView::OutputSocket { name, .. } => {
            FuncArgument::find_by_name_for_func(ctx, name, func_id)
                .await?
                .ok_or(PkgError::MissingFuncArgument(name.to_owned(), func_id))?
        }
    };

    if apa.func_argument_id() != *arg.id() {
        apa.set_func_argument_id(ctx, arg.id()).await?;
    }

    match input {
        SiPkgAttrFuncInputView::Prop { prop_path, .. } => {
            let prop = Prop::find_prop_by_path(ctx, schema_variant_id, &prop_path.into()).await?;
            let prop_ip = InternalProvider::find_for_prop(ctx, *prop.id())
                .await?
                .ok_or(PkgError::MissingInternalProviderForProp(*prop.id()))?;

            if apa.internal_provider_id() != *prop_ip.id() {
                apa.set_internal_provider_id_safe(ctx, *prop_ip.id())
                    .await?;
            }
        }
        SiPkgAttrFuncInputView::InputSocket { socket_name, .. } => {
            let explicit_ip = InternalProvider::find_explicit_for_schema_variant_and_name(
                ctx,
                schema_variant_id,
                &socket_name,
            )
            .await?
            .ok_or(PkgError::MissingInternalProviderForSocketName(
                socket_name.to_owned(),
            ))?;

            if apa.internal_provider_id() != *explicit_ip.id() {
                apa.set_internal_provider_id_safe(ctx, *explicit_ip.id())
                    .await?;
            }
        }
        _ => {}
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn import_attr_func(
    ctx: &DalContext,
    change_set_pk: ChangeSetPk,
    context: AttributeReadContext,
    key: Option<String>,
    schema_variant_id: SchemaVariantId,
    func_id: FuncId,
    inputs: Vec<SiPkgAttrFuncInputView>,
    thing_map: &mut ThingMap,
) -> PkgResult<()> {
    let mut prototype = get_prototype_for_context(ctx, context, key).await?;

    if prototype.func_id() != func_id {
        prototype.set_func_id(ctx, &func_id).await?;
    }

    for input in &inputs {
        let (unique_id, deleted) = match input {
            SiPkgAttrFuncInputView::Prop {
                unique_id, deleted, ..
            }
            | SiPkgAttrFuncInputView::InputSocket {
                unique_id, deleted, ..
            }
            | SiPkgAttrFuncInputView::OutputSocket {
                unique_id, deleted, ..
            } => (unique_id, *deleted),
        };

        let apa = match unique_id
            .as_deref()
            .and_then(|unique_id| thing_map.get(change_set_pk, &unique_id.to_owned()))
        {
            Some(Thing::AttributePrototypeArgument(apa)) => {
                let mut apa = apa.to_owned();
                if deleted {
                    apa.delete_by_id(ctx).await?;
                } else {
                    update_attr_proto_arg(
                        ctx,
                        &mut apa,
                        *prototype.id(),
                        input,
                        func_id,
                        schema_variant_id,
                    )
                    .await?;
                }

                Some(apa)
            }
            _ => {
                if deleted {
                    None
                } else {
                    Some(
                        create_attr_proto_arg(
                            ctx,
                            *prototype.id(),
                            input,
                            func_id,
                            schema_variant_id,
                        )
                        .await?,
                    )
                }
            }
        };

        if let (Some(apa), Some(unique_id)) = (apa, unique_id) {
            thing_map.insert(
                change_set_pk,
                unique_id.to_owned(),
                Thing::AttributePrototypeArgument(apa),
            );
        }
    }

    Ok(())
}

fn prop_kind_for_pkg_prop(pkg_prop: &SiPkgProp<'_>) -> PropKind {
    match pkg_prop {
        SiPkgProp::Array { .. } => PropKind::Array,
        SiPkgProp::Boolean { .. } => PropKind::Boolean,
        SiPkgProp::Map { .. } => PropKind::Map,
        SiPkgProp::Number { .. } => PropKind::Integer,
        SiPkgProp::Object { .. } => PropKind::Object,
        SiPkgProp::String { .. } => PropKind::String,
    }
}

async fn create_dal_prop(
    ctx: &DalContext,
    data: &SiPkgPropData,
    kind: PropKind,
    schema_variant_id: SchemaVariantId,
    parent_prop_id: Option<PropId>,
) -> PkgResult<Prop> {
    let mut prop = Prop::new(
        ctx,
        &data.name,
        kind,
        schema_variant_id,
        parent_prop_id,
        Some(((&data.widget_kind).into(), data.widget_options.to_owned())),
        data.documentation.to_owned(),
        data.validation_format.to_owned(),
    )
    .await
    .map_err(SiPkgError::visit_prop)?;

    prop.set_hidden(ctx, data.hidden).await?;
    prop.set_doc_link(ctx, data.doc_link.as_ref().map(|l| l.to_string()))
        .await?;

    Ok(prop)
}

async fn create_prop(
    spec: SiPkgProp<'_>,
    parent_prop_info: Option<(PropId, PropPath)>,
    ctx: &PropVisitContext<'_>,
) -> PkgResult<Option<(PropId, PropPath)>> {
    let prop = {
        let parent_path = parent_prop_info
            .as_ref()
            .map(|info| info.1.to_owned())
            .unwrap_or(PropPath::new(["root"]));

        let path = parent_path.join(&PropPath::new([spec.name()]));

        match Prop::find_prop_by_path_opt(ctx.ctx, ctx.schema_variant_id, &path).await? {
            None => {
                let data = spec.data().ok_or(PkgError::DataNotFound("prop".into()))?;
                create_dal_prop(
                    ctx.ctx,
                    data,
                    prop_kind_for_pkg_prop(&spec),
                    ctx.schema_variant_id,
                    parent_prop_info.as_ref().map(|info| info.0.to_owned()),
                )
                .await?
            }
            Some(prop) => prop,
        }
    };

    let prop_id = *prop.id();

    // Both attribute functions and default values have to be set *after* the schema variant is
    // "finalized", so we can't do until we construct the *entire* prop tree. Hence we push work
    // queues up to the outer context via the PropVisitContext, which uses Mutexes for interior
    // mutability (maybe there's a better type for that here?)

    if let Some(data) = spec.data() {
        if let Some(default_value_info) = match &spec {
            SiPkgProp::String { .. } => {
                if let Some(serde_json::Value::String(default_value)) = &data.default_value {
                    Some(DefaultValueInfo::String {
                        prop_id,
                        default_value: default_value.to_owned(),
                    })
                } else {
                    // Raise error here for type mismatch
                    None
                }
            }
            SiPkgProp::Number { .. } => {
                if let Some(serde_json::Value::Number(default_value_number)) = &data.default_value {
                    if default_value_number.is_i64() {
                        default_value_number
                            .as_i64()
                            .map(|dv_i64| DefaultValueInfo::Number {
                                prop_id,
                                default_value: dv_i64,
                            })
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            SiPkgProp::Boolean { .. } => {
                if let Some(serde_json::Value::Bool(default_value)) = &data.default_value {
                    Some(DefaultValueInfo::Boolean {
                        prop_id,
                        default_value: *default_value,
                    })
                } else {
                    None
                }
            }
            // Default values for complex types are not yet supported in packages
            _ => None,
        } {
            ctx.default_values.lock().await.push(default_value_info);
        }
    }

    if matches!(&spec, SiPkgProp::Map { .. }) {
        for map_key_func in spec.map_key_funcs()? {
            let key = map_key_func.key();
            let mut inputs = map_key_func.inputs()?;
            let func_unique_id = map_key_func.func_unique_id();

            ctx.map_key_funcs.lock().await.push((
                key.to_owned(),
                AttrFuncInfo {
                    func_unique_id: func_unique_id.to_owned(),
                    prop_id,
                    inputs: inputs.drain(..).map(Into::into).collect(),
                },
            ));
        }
    }

    if let Some(func_unique_id) = spec.data().and_then(|data| data.func_unique_id.to_owned()) {
        let mut inputs = spec.inputs()?;
        ctx.attr_funcs.lock().await.push(AttrFuncInfo {
            func_unique_id,
            prop_id,
            inputs: inputs.drain(..).map(Into::into).collect(),
        });
    }

    Ok(Some((*prop.id(), prop.path())))
}
