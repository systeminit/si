use si_pkg::{
    SchemaVariantSpecPropRoot, SiPkg, SiPkgActionFunc, SiPkgAttrFuncInputView, SiPkgAuthFunc,
    SiPkgComponent, SiPkgEdge, SiPkgError, SiPkgFunc, SiPkgFuncArgument, SiPkgFuncData, SiPkgKind,
    SiPkgLeafFunction, SiPkgMetadata, SiPkgProp, SiPkgPropData, SiPkgSchema, SiPkgSchemaData,
    SiPkgSchemaVariant, SiPkgSocket, SiPkgSocketData, SocketSpecKind,
};
use std::collections::HashSet;
use std::{collections::HashMap, path::Path};
use telemetry::prelude::*;
use tokio::sync::Mutex;

use crate::attribute::prototype::argument::{
    value_source::ValueSource, AttributePrototypeArgument, AttributePrototypeArgumentId,
};
use crate::authentication_prototype::{AuthenticationPrototype, AuthenticationPrototypeId};
use crate::func::intrinsics::IntrinsicFunc;
use crate::module::{Module, ModuleId};
use crate::schema::variant::SchemaVariantJson;
use crate::socket::connection_annotation::ConnectionAnnotation;
use crate::SocketKind;
use crate::{func, ChangeSetId};
use crate::{
    func::argument::FuncArgument,
    prop::PropPath,
    schema::variant::leaves::{LeafInputLocation, LeafKind},
    DalContext, DeprecatedActionPrototype, EdgeWeightKind, Func, FuncId, InputSocket, OutputSocket,
    OutputSocketId, Prop, PropId, PropKind, Schema, SchemaId, SchemaVariant, SchemaVariantId,
};
use crate::{AttributePrototype, AttributePrototypeId};

use super::{PkgError, PkgResult};

#[derive(Clone, Debug)]
pub enum Thing {
    ActionPrototype(DeprecatedActionPrototype),
    AuthPrototype(AuthenticationPrototype),
    // AttributePrototypeArgument(AttributePrototypeArgument),
    // Component((Component, Node)),
    // Edge(Edge),
    Func(Func),
    #[allow(dead_code)]
    FuncArgument(FuncArgument),
    Schema(Schema),
    SchemaVariant(SchemaVariant),
    Socket(Box<(Option<InputSocket>, Option<OutputSocket>)>),
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
    change_set_id: Option<ChangeSetId>,
    metadata: &SiPkgMetadata,
    funcs: &[SiPkgFunc<'_>],
    schemas: &[SiPkgSchema<'_>],
    _components: &[SiPkgComponent<'_>],
    _edges: &[SiPkgEdge<'_>],
    installed_pkg: Option<Module>,
    thing_map: &mut ThingMap,
    options: &ImportOptions,
) -> PkgResult<(
    Vec<SchemaVariantId>,
    Vec<(String, Vec<bool /*ImportAttributeSkip*/>)>,
    Vec<bool /*ImportEdgeSkip*/>,
)> {
    // let default_change_set_id = ctx.get_workspace_default_change_set_id().await?;
    for func_spec in funcs {
        let unique_id = func_spec.unique_id().to_string();

        // This is a hack because the hash of the intrinsics has changed from the version in the
        // packages. We also apply this to si:resourcePayloadToValue since it should be an
        // intrinsic but is only in our packages
        if func::is_intrinsic(func_spec.name())
            || SPECIAL_CASE_FUNCS.contains(&func_spec.name())
            || func_spec.is_from_builtin().unwrap_or(false)
        {
            if let Some(func_id) = Func::find_by_name(ctx, func_spec.name()).await? {
                let func = Func::get_by_id_or_error(ctx, func_id).await?;

                thing_map.insert(
                    change_set_id,
                    unique_id.to_owned(),
                    Thing::Func(func.to_owned()),
                );
            } else if let Some(func) = import_func(
                ctx,
                None,
                func_spec,
                installed_pkg.clone(),
                thing_map,
                options.is_builtin,
            )
            .await?
            {
                let args = func_spec.arguments()?;

                if !args.is_empty() {
                    import_func_arguments(ctx, None, func.id, &args, thing_map).await?;
                }
            }
        } else {
            let func = if let Some(Some(func)) = options
                .skip_import_funcs
                .as_ref()
                .map(|skip_funcs| skip_funcs.get(&unique_id))
            {
                if let Some(installed_pkg) = installed_pkg.clone() {
                    installed_pkg
                        .create_association(ctx, func.id.into())
                        .await?;
                }

                // We're not going to import this func but we need it in the map for lookups later
                thing_map.insert(
                    change_set_id,
                    func_spec.unique_id().to_owned(),
                    Thing::Func(func.to_owned()),
                );

                None
            } else {
                import_func(
                    ctx,
                    change_set_id,
                    func_spec,
                    installed_pkg.clone(),
                    thing_map,
                    options.is_builtin,
                )
                .await?
            };

            if let Some(func) = func {
                thing_map.insert(
                    change_set_id,
                    unique_id.to_owned(),
                    Thing::Func(func.to_owned()),
                );

                if let Some(module) = installed_pkg.clone() {
                    module.create_association(ctx, func.id.into()).await?;
                }

                let args = func_spec.arguments()?;

                if !args.is_empty() {
                    import_func_arguments(ctx, change_set_id, func.id, &args, thing_map).await?;
                }
            }
        };
    }

    let mut installed_schema_variant_ids = vec![];

    let mut unseen: HashSet<String> = options
        .schemas
        .clone()
        .unwrap_or_default()
        .iter()
        .cloned()
        .collect();
    for schema_spec in schemas {
        let normalized_name = &schema_spec.name().to_string().to_lowercase();

        match &options.schemas {
            None => {}
            Some(schemas) => {
                if !schemas.contains(normalized_name) {
                    continue;
                }
            }
        }

        unseen.remove(normalized_name);

        info!(
            "installing schema '{}' from {}",
            schema_spec.name(),
            metadata.name(),
        );

        let (_, schema_variant_ids) = import_schema(
            ctx,
            change_set_id,
            schema_spec,
            installed_pkg.clone(),
            thing_map,
        )
        .await?;

        installed_schema_variant_ids.extend(schema_variant_ids);
    }

    for schema_name in unseen {
        error!(
            "options specified schema '{}', but it's not present on {}",
            schema_name,
            metadata.name(),
        );
    }

    Ok((
        installed_schema_variant_ids,
        vec![], // component_attribute_skips,
        vec![], // edge_skips,
    ))
}

pub async fn import_pkg_from_pkg(
    ctx: &DalContext,
    pkg: &SiPkg,
    options: Option<ImportOptions>,
) -> PkgResult<(
    Option<ModuleId>,
    Vec<SchemaVariantId>,
    Option<Vec<bool /*ImportSkips*/>>,
)> {
    let root_hash = pkg.hash()?.to_string();

    let options = options.unwrap_or_default();

    if Module::find_by_root_hash(ctx, &root_hash).await?.is_some() {
        return Err(PkgError::PackageAlreadyInstalled(root_hash));
    }

    let metadata = pkg.metadata()?;

    let installed_module: Option<Module> = if options.no_record {
        None
    } else {
        Some(
            Module::new(
                ctx,
                metadata.name(),
                pkg.hash()?.to_string(),
                metadata.version(),
                metadata.description(),
                metadata.created_by(),
                metadata.created_at(),
            )
            .await?,
        )
    };
    let default_change_set_id = ctx.get_workspace_default_change_set_id().await?;
    let mut change_set_things = ThingMap::new(default_change_set_id);

    match metadata.kind() {
        SiPkgKind::Module => {
            let (installed_schema_variant_ids, _, _) = import_change_set(
                ctx,
                None,
                &metadata,
                &pkg.funcs()?,
                &pkg.schemas()?,
                &[],
                &[],
                installed_module,
                &mut change_set_things,
                &options,
            )
            .await?;

            Ok((None, installed_schema_variant_ids, None))
        }
        SiPkgKind::WorkspaceBackup => Err(PkgError::WorkspaceExportNotSupported()),
    }
}

pub async fn import_pkg(ctx: &DalContext, pkg_file_path: impl AsRef<Path>) -> PkgResult<SiPkg> {
    println!("Importing package from {:?}", pkg_file_path.as_ref());
    let pkg = SiPkg::load_from_file(&pkg_file_path).await?;

    import_pkg_from_pkg(ctx, &pkg, None).await?;

    Ok(pkg)
}

async fn create_func(
    ctx: &DalContext,
    func_spec: &SiPkgFunc<'_>,
    is_builtin: bool,
) -> PkgResult<Func> {
    let name = func_spec.name();

    let func_spec_data = func_spec
        .data()
        .ok_or(PkgError::DataNotFound(name.into()))?;

    let func = Func::new(
        ctx,
        name,
        func_spec_data
            .display_name()
            .map(|display_name| display_name.to_owned()),
        func_spec_data.description().map(|desc| desc.to_owned()),
        func_spec_data.link().map(|l| l.to_string()),
        func_spec_data.hidden(),
        is_builtin,
        func_spec_data.backend_kind().into(),
        func_spec_data.response_type().into(),
        Some(func_spec_data.handler().to_owned()),
        Some(func_spec_data.code_base64().to_owned()),
    )
    .await?;

    Ok(func)
}

#[allow(dead_code)]
async fn update_func(
    ctx: &DalContext,
    func: Func,
    func_spec_data: &SiPkgFuncData,
) -> PkgResult<Func> {
    let func = func
        .modify(ctx, |func| {
            func.name = func_spec_data.name().to_owned();
            func.backend_kind = func_spec_data.backend_kind().into();
            func.backend_response_type = func_spec_data.response_type().into();
            func.display_name = func_spec_data
                .display_name()
                .map(|display_name| display_name.to_owned());
            func.code_base64 = Some(func_spec_data.code_base64().to_owned());
            func.description = func_spec_data.description().map(|desc| desc.to_owned());
            func.handler = Some(func_spec_data.handler().to_owned());
            func.hidden = func_spec_data.hidden();
            func.link = func_spec_data.link().map(|l| l.to_string());

            Ok(())
        })
        .await?;

    Ok(func)
}

pub async fn import_func(
    ctx: &DalContext,
    change_set_id: Option<ChangeSetId>,
    func_spec: &SiPkgFunc<'_>,
    installed_module: Option<Module>,
    thing_map: &mut ThingMap,
    is_builtin: bool,
) -> PkgResult<Option<Func>> {
    let func = match change_set_id {
        None => {
            let mut existing_func: Option<Func> = None;
            if let Some(installed_pkg) = installed_module.clone() {
                let associated_funcs = installed_pkg.list_associated_funcs(ctx).await?;
                let mut maybe_matching_func: Vec<Func> = associated_funcs
                    .into_iter()
                    .filter(|f| f.name.clone() == func_spec.name())
                    .collect();
                if let Some(matching_func) = maybe_matching_func.pop() {
                    existing_func = Some(matching_func);
                }
            }

            let (func, created) = match existing_func {
                None => (create_func(ctx, func_spec, is_builtin).await?, true),
                Some(installed_func_record) => (installed_func_record, true),
            };

            if let Some(installed_pkg) = installed_module {
                installed_pkg
                    .create_association(ctx, func.id.into())
                    .await?;
            }

            thing_map.insert(
                change_set_id,
                func_spec.unique_id().to_owned(),
                Thing::Func(func.to_owned()),
            );

            if created {
                Some(func)
            } else {
                None
            }
        }
        Some(_) => return Err(PkgError::WorkspaceExportNotSupported()),
    };

    if let Some(func) = func.as_ref() {
        thing_map.insert(
            change_set_id,
            func_spec.unique_id().to_owned(),
            Thing::Func(func.to_owned()),
        );
    }

    Ok(func)
}

async fn create_func_argument(
    ctx: &DalContext,
    func_id: FuncId,
    func_arg: &SiPkgFuncArgument<'_>,
) -> PkgResult<FuncArgument> {
    Ok(FuncArgument::new(
        ctx,
        func_arg.name(),
        func_arg.kind().into(),
        func_arg.element_kind().to_owned().map(|&kind| kind.into()),
        func_id,
    )
    .await?)
}

async fn import_func_arguments(
    ctx: &DalContext,
    change_set_id: Option<ChangeSetId>,
    func_id: FuncId,
    func_arguments: &[SiPkgFuncArgument<'_>],
    _thing_map: &mut ThingMap,
) -> PkgResult<()> {
    match change_set_id {
        None => {
            for arg in func_arguments {
                create_func_argument(ctx, func_id, arg).await?;
            }
        }
        Some(_) => return Err(PkgError::WorkspaceExportNotSupported()),
    }

    Ok(())
}

async fn create_schema(ctx: &DalContext, schema_spec_data: &SiPkgSchemaData) -> PkgResult<Schema> {
    let schema = Schema::new(ctx, schema_spec_data.name())
        .await?
        .modify(ctx, |schema| {
            schema.ui_hidden = schema_spec_data.ui_hidden();
            Ok(())
        })
        .await?;
    Ok(schema)
}

async fn import_schema(
    ctx: &DalContext,
    change_set_id: Option<ChangeSetId>,
    schema_spec: &SiPkgSchema<'_>,
    installed_module: Option<Module>,
    thing_map: &mut ThingMap,
) -> PkgResult<(Option<SchemaId>, Vec<SchemaVariantId>)> {
    let schema_and_category = match change_set_id {
        None => {
            let mut existing_schema: Option<Schema> = None;
            if let Some(installed_pkg) = installed_module.clone() {
                let associated_schemas = installed_pkg.list_associated_schemas(ctx).await?;
                let mut maybe_matching_schema: Vec<Schema> = associated_schemas
                    .into_iter()
                    .filter(|s| s.name.clone() == schema_spec.name())
                    .collect();
                if let Some(matching_schema) = maybe_matching_schema.pop() {
                    existing_schema = Some(matching_schema);
                }
            }
            let data = schema_spec
                .data()
                .ok_or(PkgError::DataNotFound("schema".into()))?;

            // NOTE(nick): with the new engine, the category moves to the schema variant, so we need
            // to pull it off here, even if we find an existing schema.
            let category = data.category.clone();

            let schema = match existing_schema {
                None => create_schema(ctx, data).await?,
                Some(installed_schema_record) => installed_schema_record,
            };

            // Even if the asset is already installed, we write a record of the asset installation so that
            // we can track the installed packages that share schemas.
            if let Some(module) = installed_module.clone() {
                module.create_association(ctx, schema.id().into()).await?;
            }

            Some((schema, category))
        }
        Some(_) => return Err(PkgError::WorkspaceExportNotSupported()),
    };

    if let Some((mut schema, _category)) = schema_and_category {
        if let Some(unique_id) = schema_spec.unique_id() {
            thing_map.insert(
                change_set_id,
                unique_id.to_owned(),
                Thing::Schema(schema.to_owned()),
            );
        }

        let mut installed_schema_variant_ids = vec![];
        for variant_spec in &schema_spec.variants()? {
            let variant = import_schema_variant(
                ctx,
                change_set_id,
                &mut schema,
                schema_spec.clone(),
                variant_spec,
                installed_module.clone(),
                thing_map,
            )
            .await?;

            if let Some(variant) = variant {
                installed_schema_variant_ids.push(variant.id());

                set_default_schema_variant_id(
                    ctx,
                    change_set_id,
                    &mut schema,
                    schema_spec
                        .data()
                        .as_ref()
                        .and_then(|data| data.default_schema_variant()),
                    variant_spec.unique_id(),
                    variant.id(),
                )
                .await?;
            }
        }

        Ok((Some(schema.id()), installed_schema_variant_ids))
    } else {
        Ok((None, vec![]))
    }
}

async fn set_default_schema_variant_id(
    ctx: &DalContext,
    change_set_id: Option<ChangeSetId>,
    schema: &mut Schema,
    spec_default_unique_id: Option<&str>,
    variant_unique_id: Option<&str>,
    variant_id: SchemaVariantId,
) -> PkgResult<()> {
    match (change_set_id, variant_unique_id, spec_default_unique_id) {
        (None, _, _) | (Some(_), None, _) | (_, Some(_), None) => {
            if schema.get_default_schema_variant_id(ctx).await?.is_none() {
                schema.set_default_schema_variant(ctx, variant_id).await?;
            }
        }
        (Some(_), Some(variant_unique_id), Some(spec_default_unique_id)) => {
            if variant_unique_id == spec_default_unique_id {
                let current_default_variant_id = schema
                    .get_default_schema_variant_id(ctx)
                    .await?
                    .unwrap_or(SchemaVariantId::NONE);

                if variant_id != current_default_variant_id {
                    schema.set_default_schema_variant(ctx, variant_id).await?;
                }
            }
        }
    }

    Ok(())
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
struct AttrFuncInfo {
    func_unique_id: String,
    prop_id: PropId,
    inputs: Vec<SiPkgAttrFuncInputView>,
}

#[allow(dead_code)]
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
    pub change_set_id: Option<ChangeSetId>,
}

async fn import_leaf_function(
    ctx: &DalContext,
    change_set_id: Option<ChangeSetId>,
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

    match thing_map.get(change_set_id, &leaf_func.func_unique_id().to_owned()) {
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

async fn get_identity_func(ctx: &DalContext) -> PkgResult<FuncId> {
    Ok(Func::find_intrinsic(ctx, IntrinsicFunc::Identity).await?)
}

async fn create_socket(
    ctx: &DalContext,
    data: &SiPkgSocketData,
    schema_variant_id: SchemaVariantId,
) -> PkgResult<(Option<InputSocket>, Option<OutputSocket>)> {
    let identity_func_id = get_identity_func(ctx).await?;

    let connection_annotations: Vec<ConnectionAnnotation> =
        if data.connection_annotations().contains("tokens") {
            // This is the new format of connection annotations and will be
            // in the format [{\"tokens\":[\"region\"]}]
            // So we can deserialize this directly to Vec<ConnectionAnnotations>
            let mut stash = vec![];

            let raw_cas_values =
                serde_json::from_str::<Vec<ConnectionAnnotation>>(data.connection_annotations())?;

            for raw_ca in raw_cas_values {
                stash.push(raw_ca);
            }

            stash
        } else {
            // This is now the old format and we need to change how we
            // deserialize
            // The old format is a Vec of strings
            // "[\"text area\"]"
            let mut stash = vec![];
            let raw_cas_values =
                serde_json::from_str::<Vec<String>>(data.connection_annotations())?;
            for raw_cas_value in raw_cas_values {
                let cas_value = ConnectionAnnotation::from_tokens_array(vec![raw_cas_value]);
                stash.push(cas_value);
            }

            stash
        };

    let (input_socket, output_socket) = match data.kind() {
        SocketSpecKind::Input => {
            let input_socket = InputSocket::new(
                ctx,
                schema_variant_id,
                data.name(),
                identity_func_id,
                data.arity().into(),
                SocketKind::Standard,
                Some(connection_annotations),
            )
            .await?;

            (Some(input_socket), None)
        }
        SocketSpecKind::Output => {
            let output_socket = OutputSocket::new(
                ctx,
                schema_variant_id,
                data.name(),
                None,
                identity_func_id,
                data.arity().into(),
                SocketKind::Standard,
                Some(connection_annotations),
            )
            .await?;

            (None, Some(output_socket))
        }
    };

    // TODO: add modify_by_id to socket, ui hide frames
    // socket.set_ui_hidden(ctx, data.ui_hidden()).await?;

    Ok((input_socket, output_socket))
}

async fn import_socket(
    ctx: &DalContext,
    change_set_id: Option<ChangeSetId>,
    socket_spec: SiPkgSocket<'_>,
    schema_variant_id: SchemaVariantId,
    thing_map: &mut ThingMap,
) -> PkgResult<()> {
    let (input_socket, output_socket) = match change_set_id {
        None => {
            let data = socket_spec
                .data()
                .ok_or(PkgError::DataNotFound(socket_spec.name().into()))?;

            create_socket(ctx, data, schema_variant_id).await?
        }
        Some(_) => return Err(PkgError::WorkspaceExportNotSupported()),
    };

    if let Some(unique_id) = socket_spec.unique_id() {
        thing_map.insert(
            change_set_id,
            unique_id.to_owned(),
            Thing::Socket(Box::new((
                input_socket.to_owned(),
                output_socket.to_owned(),
            ))),
        );
    }

    match (
        socket_spec.data().and_then(|data| data.func_unique_id()),
        output_socket,
        input_socket,
    ) {
        (Some(func_unique_id), Some(output_socket), None) => {
            import_attr_func_for_output_socket(
                ctx,
                change_set_id,
                schema_variant_id,
                output_socket.id(),
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

async fn create_action_protoype(
    ctx: &DalContext,
    action_func_spec: &SiPkgActionFunc<'_>,
    func_id: FuncId,
    schema_variant_id: SchemaVariantId,
) -> PkgResult<DeprecatedActionPrototype> {
    let proto = DeprecatedActionPrototype::new(
        ctx,
        action_func_spec.name(),
        action_func_spec.kind().into(),
        schema_variant_id,
        func_id,
    )
    .await?;

    Ok(proto)
}

async fn import_action_func(
    ctx: &DalContext,
    change_set_id: Option<ChangeSetId>,
    action_func_spec: &SiPkgActionFunc<'_>,
    schema_variant_id: SchemaVariantId,
    thing_map: &ThingMap,
) -> PkgResult<Option<DeprecatedActionPrototype>> {
    let prototype =
        match thing_map.get(change_set_id, &action_func_spec.func_unique_id().to_owned()) {
            Some(Thing::Func(func)) => {
                let func_id = func.id;

                if let Some(unique_id) = action_func_spec.unique_id() {
                    match thing_map.get(change_set_id, &unique_id.to_owned()) {
                        Some(Thing::ActionPrototype(_prototype)) => {
                            return Err(PkgError::WorkspaceExportNotSupported())
                        }
                        _ => {
                            if action_func_spec.deleted() {
                                None
                            } else {
                                Some(
                                    create_action_protoype(
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
                        create_action_protoype(ctx, action_func_spec, func_id, schema_variant_id)
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
    change_set_id: Option<ChangeSetId>,
    func_spec: &SiPkgAuthFunc<'_>,
    schema_variant_id: SchemaVariantId,
    thing_map: &ThingMap,
) -> PkgResult<Option<AuthenticationPrototype>> {
    let prototype = match thing_map.get(change_set_id, &func_spec.func_unique_id().to_owned()) {
        Some(Thing::Func(func)) => {
            let func_id = func.id;

            if let Some(unique_id) = func_spec.unique_id() {
                match thing_map.get(change_set_id, &unique_id.to_owned()) {
                    Some(Thing::AuthPrototype(_prototype)) => {
                        return Err(PkgError::WorkspaceExportNotSupported())
                    }
                    _ => {
                        if func_spec.deleted() {
                            None
                        } else {
                            SchemaVariant::new_authentication_prototype(
                                ctx,
                                func_id,
                                schema_variant_id,
                            )
                            .await?;

                            Some(AuthenticationPrototype {
                                id: AuthenticationPrototypeId::generate(),
                                func_id,
                                schema_variant_id,
                            })
                        }
                    }
                }
            } else {
                SchemaVariant::new_authentication_prototype(ctx, func_id, schema_variant_id)
                    .await?;
                Some(AuthenticationPrototype {
                    id: AuthenticationPrototypeId::generate(),
                    func_id,
                    schema_variant_id,
                })
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
    change_set_id: Option<ChangeSetId>,
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
        change_set_id,
    };

    let parent_info = ParentPropInfo {
        prop_id: prop_root_prop_id,
        path: PropPath::new(prop_root.path_parts()),
    };

    variant_spec
        .visit_prop_tree(prop_root, create_prop, Some(parent_info), &context)
        .await?;

    Ok(CreatePropsSideEffects {
        attr_funcs: context.attr_funcs.into_inner(),
        default_values: context.default_values.into_inner(),
        map_key_funcs: context.map_key_funcs.into_inner(),
    })
}

/// Duplicate all the functions, and return a thing_map with them included, so
/// that we can import a standalone schema variant.
pub async fn clone_and_import_funcs(
    ctx: &DalContext,
    funcs: Vec<SiPkgFunc<'_>>,
) -> PkgResult<ThingMap> {
    let mut thing_map = ThingMap::new(ctx.get_workspace_default_change_set_id().await?);

    for func_spec in funcs {
        let func = if func::is_intrinsic(func_spec.name())
            || SPECIAL_CASE_FUNCS.contains(&func_spec.name())
        {
            let func_id = Func::find_by_name(ctx, &func_spec.name())
                .await?
                .ok_or(PkgError::MissingIntrinsicFunc(func_spec.name().to_owned()))?;

            Func::get_by_id_or_error(ctx, func_id).await?
        } else {
            let func = create_func(ctx, &func_spec, false).await?;

            if !func_spec.arguments()?.is_empty() {
                import_func_arguments(ctx, None, func.id, &func_spec.arguments()?, &mut thing_map)
                    .await?;
            }

            func
        };

        thing_map.insert(None, func_spec.unique_id().into(), Thing::Func(func));
    }

    Ok(thing_map)
}

pub(crate) async fn import_schema_variant(
    ctx: &DalContext,
    change_set_id: Option<ChangeSetId>,
    schema: &mut Schema,
    schema_spec: SiPkgSchema<'_>,
    variant_spec: &SiPkgSchemaVariant<'_>,
    installed_module: Option<Module>,
    thing_map: &mut ThingMap,
) -> PkgResult<Option<SchemaVariant>> {
    let schema_variant = match change_set_id {
        None => {
            let mut existing_schema_variant: Option<SchemaVariant> = None;
            if let Some(installed_pkg) = installed_module.clone() {
                let associated_schema_variants =
                    installed_pkg.list_associated_schema_variants(ctx).await?;
                let mut maybe_matching_schema_variant: Vec<SchemaVariant> =
                    associated_schema_variants
                        .into_iter()
                        .filter(|s| s.name() == schema_spec.name())
                        .collect();
                if let Some(matching_schema_variant) = maybe_matching_schema_variant.pop() {
                    existing_schema_variant = Some(matching_schema_variant);
                }
            }

            let (variant, created) = match existing_schema_variant {
                None => {
                    let spec = schema_spec.to_spec().await?;
                    let metadata = SchemaVariantJson::metadata_from_spec(spec)?;

                    let mut asset_func_id: Option<FuncId> = None;
                    if let Some(variant_spec_data) = variant_spec.data() {
                        let func_unique_id = variant_spec_data.func_unique_id().to_owned();
                        if let Thing::Func(asset_func) = thing_map
                            .get(change_set_id, &func_unique_id)
                            .ok_or(PkgError::MissingFuncUniqueId(func_unique_id.to_string()))?
                        {
                            asset_func_id = Some(asset_func.id)
                        }
                    }
                    (
                        SchemaVariant::new(
                            ctx,
                            schema.id(),
                            variant_spec.name(),
                            metadata.menu_name,
                            metadata.category,
                            metadata.color,
                            metadata.component_type,
                            metadata.link,
                            metadata.description,
                            asset_func_id,
                            variant_spec.is_builtin(),
                        )
                        .await?
                        .0,
                        true,
                    )
                }
                Some(installed_variant) => (installed_variant, true),
            };

            if let Some(module) = installed_module.clone() {
                module.create_association(ctx, variant.id().into()).await?;
            }

            if created {
                Some(variant)
            } else {
                None
            }
        }
        Some(_) => return Err(PkgError::WorkspaceExportNotSupported()),
    };

    let schema_variant = match schema_variant {
        None => None,
        Some(schema_variant) => {
            if let Some(unique_id) = variant_spec.unique_id() {
                thing_map.insert(
                    change_set_id,
                    unique_id.to_owned(),
                    Thing::SchemaVariant(schema_variant.to_owned()),
                );
            }

            if let Some(data) = variant_spec.data() {
                if let Some(color) = data.color() {
                    let current_color = schema_variant.get_color(ctx).await?;
                    if current_color != color {
                        schema_variant.set_color(ctx, color).await?
                    }
                }

                schema_variant
                    .set_type(ctx, data.component_type().to_string())
                    .await?;
            }

            let mut side_effects = CreatePropsSideEffects::default();

            let domain_prop_id = Prop::find_prop_id_by_path(
                ctx,
                schema_variant.id(),
                &PropPath::new(["root", "domain"]),
            )
            .await?;

            side_effects.extend(
                create_props(
                    ctx,
                    change_set_id,
                    variant_spec,
                    SchemaVariantSpecPropRoot::Domain,
                    domain_prop_id,
                    schema_variant.id(),
                )
                .await?,
            );

            let resource_value_prop_id = Prop::find_prop_id_by_path(
                ctx,
                schema_variant.id(),
                &PropPath::new(["root", "resource_value"]),
            )
            .await?;

            side_effects.extend(
                create_props(
                    ctx,
                    change_set_id,
                    variant_spec,
                    SchemaVariantSpecPropRoot::ResourceValue,
                    resource_value_prop_id,
                    schema_variant.id(),
                )
                .await?,
            );

            let secrets_prop_id = Prop::find_prop_id_by_path(
                ctx,
                schema_variant.id(),
                &PropPath::new(["root", "secrets"]),
            )
            .await?;

            side_effects.extend(
                create_props(
                    ctx,
                    change_set_id,
                    variant_spec,
                    SchemaVariantSpecPropRoot::Secrets,
                    secrets_prop_id,
                    schema_variant.id(),
                )
                .await?,
            );

            if !variant_spec.secret_definitions()?.is_empty() {
                let root_prop_id =
                    Prop::find_prop_id_by_path(ctx, schema_variant.id(), &PropPath::new(["root"]))
                        .await?;

                let secret_definition_prop = Prop::new_without_ui_optionals(
                    ctx,
                    "secret_definition",
                    PropKind::Object,
                    root_prop_id,
                )
                .await?;
                let secret_definition_prop_id = secret_definition_prop.id();

                side_effects.extend(
                    create_props(
                        ctx,
                        change_set_id,
                        variant_spec,
                        SchemaVariantSpecPropRoot::SecretDefinition,
                        secret_definition_prop_id,
                        schema_variant.id(),
                    )
                    .await?,
                );
            }

            SchemaVariant::finalize(ctx, schema_variant.id()).await?;

            for socket in variant_spec.sockets()? {
                import_socket(ctx, change_set_id, socket, schema_variant.id(), thing_map).await?;
            }

            for action_func in &variant_spec.action_funcs()? {
                let prototype = import_action_func(
                    ctx,
                    change_set_id,
                    action_func,
                    schema_variant.id(),
                    thing_map,
                )
                .await?;

                if let (Some(prototype), Some(unique_id)) = (prototype, action_func.unique_id()) {
                    thing_map.insert(
                        change_set_id,
                        unique_id.to_owned(),
                        Thing::ActionPrototype(prototype),
                    );
                }
            }

            for auth_func in &variant_spec.auth_funcs()? {
                let prototype = import_auth_func(
                    ctx,
                    change_set_id,
                    auth_func,
                    schema_variant.id(),
                    thing_map,
                )
                .await?;

                if let (Some(prototype), Some(unique_id)) = (prototype, auth_func.unique_id()) {
                    thing_map.insert(
                        change_set_id,
                        unique_id.to_owned(),
                        Thing::AuthPrototype(prototype),
                    );
                }
            }

            for leaf_func in variant_spec.leaf_functions()? {
                import_leaf_function(
                    ctx,
                    change_set_id,
                    leaf_func,
                    schema_variant.id(),
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
                let name_prop_id = Prop::find_prop_id_by_path(
                    ctx,
                    schema_variant.id(),
                    &PropPath::new(["root", "si", "name"]),
                )
                .await?;
                let name_default_value_info = DefaultValueInfo::String {
                    prop_id: name_prop_id,
                    default_value: schema.name.to_owned().to_lowercase(),
                };

                set_default_value(ctx, name_default_value_info).await?;
            }

            for si_prop_func in variant_spec.si_prop_funcs()? {
                let prop_id = Prop::find_prop_id_by_path(
                    ctx,
                    schema_variant.id(),
                    &PropPath::new(si_prop_func.kind().prop_path()),
                )
                .await?;
                import_attr_func_for_prop(
                    ctx,
                    change_set_id,
                    schema_variant.id(),
                    AttrFuncInfo {
                        func_unique_id: si_prop_func.func_unique_id().to_owned(),
                        prop_id,
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

                let prop_id = Prop::find_prop_id_by_path(
                    ctx,
                    schema_variant.id(),
                    &PropPath::new(root_prop_func.prop().path_parts()),
                )
                .await?;
                import_attr_func_for_prop(
                    ctx,
                    change_set_id,
                    schema_variant.id(),
                    AttrFuncInfo {
                        func_unique_id: root_prop_func.func_unique_id().to_owned(),
                        prop_id,
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
                attach_resource_payload_to_value(ctx, schema_variant.id()).await?;
            }

            for attr_func in side_effects.attr_funcs {
                import_attr_func_for_prop(
                    ctx,
                    change_set_id,
                    schema_variant.id(),
                    attr_func,
                    None,
                    thing_map,
                )
                .await?;
            }

            for (key, map_key_func) in side_effects.map_key_funcs {
                import_attr_func_for_prop(
                    ctx,
                    change_set_id,
                    schema_variant.id(),
                    map_key_func,
                    Some(key),
                    thing_map,
                )
                .await?;
            }

            Some(schema_variant)
        }
    };

    Ok(schema_variant)
}

async fn set_default_value(
    ctx: &DalContext,
    default_value_info: DefaultValueInfo,
) -> PkgResult<()> {
    let prop_id = match &default_value_info {
        DefaultValueInfo::Number { prop_id, .. }
        | DefaultValueInfo::String { prop_id, .. }
        | DefaultValueInfo::Boolean { prop_id, .. } => *prop_id,
    };

    match default_value_info {
        DefaultValueInfo::Boolean { default_value, .. } => {
            Prop::set_default_value(ctx, prop_id, default_value).await?
        }
        DefaultValueInfo::Number { default_value, .. } => {
            Prop::set_default_value(ctx, prop_id, default_value).await?
        }
        DefaultValueInfo::String { default_value, .. } => {
            Prop::set_default_value(ctx, prop_id, default_value).await?
        }
    }

    Ok(())
}

async fn import_attr_func_for_prop(
    ctx: &DalContext,
    change_set_id: Option<ChangeSetId>,
    schema_variant_id: SchemaVariantId,
    AttrFuncInfo {
        func_unique_id,
        prop_id,
        inputs,
    }: AttrFuncInfo,
    key: Option<String>,
    thing_map: &mut ThingMap,
) -> PkgResult<()> {
    match thing_map.get(change_set_id, &func_unique_id.to_owned()) {
        Some(Thing::Func(func)) => {
            import_attr_func(
                ctx,
                change_set_id,
                AttrFuncContext::Prop(prop_id),
                key,
                schema_variant_id,
                func.id,
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
    change_set_id: Option<ChangeSetId>,
    schema_variant_id: SchemaVariantId,
    output_socket_id: OutputSocketId,
    func_unique_id: &str,
    inputs: Vec<SiPkgAttrFuncInputView>,
    thing_map: &mut ThingMap,
) -> PkgResult<()> {
    match thing_map.get(change_set_id, &func_unique_id.to_owned()) {
        Some(Thing::Func(func)) => {
            import_attr_func(
                ctx,
                change_set_id,
                AttrFuncContext::OutputSocket(output_socket_id),
                None,
                schema_variant_id,
                func.id,
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
    context: AttrFuncContext,
    key: Option<String>,
) -> PkgResult<AttributePrototypeId> {
    if key.is_some() {
        #[allow(clippy::infallible_destructuring_match)]
        let map_prop_id = match context {
            AttrFuncContext::Prop(prop_id) => prop_id,
            _ => Err(PkgError::AttributeFuncForKeyMissingProp(
                context,
                key.to_owned().expect("check above ensures this is some"),
            ))?,
        };
        let map_prop = Prop::get_by_id_or_error(ctx, map_prop_id).await?;

        if map_prop.kind != PropKind::Map {
            return Err(PkgError::AttributeFuncForKeySetOnWrongKind(
                map_prop_id,
                key.to_owned().expect("check above ensures this is some"),
                map_prop.kind,
            ));
        }

        let element_prop_id = map_prop.element_prop_id(ctx).await?;
        Ok(
            match AttributePrototype::find_for_prop(ctx, element_prop_id, &key).await? {
                None => {
                    let unset_func_id = Func::find_intrinsic(ctx, IntrinsicFunc::Unset).await?;
                    let prototype_id = AttributePrototype::new(ctx, unset_func_id).await?.id();
                    Prop::add_edge_to_attribute_prototype(
                        ctx,
                        element_prop_id,
                        prototype_id,
                        EdgeWeightKind::Prototype(None),
                    )
                    .await?;

                    prototype_id
                }
                Some(prototype_id) => prototype_id,
            },
        )
    } else {
        Ok(match context {
            AttrFuncContext::Prop(prop_id) => {
                AttributePrototype::find_for_prop(ctx, prop_id, &None)
                    .await?
                    .ok_or(PkgError::PropMissingPrototype(prop_id))?
            }
            AttrFuncContext::OutputSocket(output_socket_id) => {
                AttributePrototype::find_for_output_socket(ctx, output_socket_id)
                    .await?
                    .ok_or(PkgError::OutputSocketMissingPrototype(output_socket_id))?
            }
        })
    }
}

async fn create_attr_proto_arg(
    ctx: &DalContext,
    prototype_id: AttributePrototypeId,
    input: &SiPkgAttrFuncInputView,
    func_id: FuncId,
    schema_variant_id: SchemaVariantId,
) -> PkgResult<AttributePrototypeArgumentId> {
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
            let prop_id =
                Prop::find_prop_id_by_path(ctx, schema_variant_id, &prop_path.into()).await?;
            let apa = AttributePrototypeArgument::new(ctx, prototype_id, arg.id).await?;
            let apa_id = apa.id();

            apa.set_value_from_prop_id(ctx, prop_id).await?;

            apa_id
        }
        SiPkgAttrFuncInputView::InputSocket { socket_name, .. } => {
            let input_socket = InputSocket::find_with_name(ctx, socket_name, schema_variant_id)
                .await?
                .ok_or(PkgError::MissingInputSocketName(socket_name.to_owned()))?;
            let apa = AttributePrototypeArgument::new(ctx, prototype_id, arg.id).await?;
            let apa_id = apa.id();

            apa.set_value_from_input_socket_id(ctx, input_socket.id())
                .await?;
            apa_id
        }
        SiPkgAttrFuncInputView::OutputSocket {
            name, socket_name, ..
        } => {
            return Err(PkgError::TakingOutputSocketAsInputForPropUnsupported(
                name.to_owned(),
                socket_name.to_owned(),
            ));
        }
    })
}

#[derive(Debug, Clone)]
pub enum AttrFuncContext {
    Prop(PropId),
    OutputSocket(OutputSocketId),
}

#[allow(clippy::too_many_arguments)]
async fn import_attr_func(
    ctx: &DalContext,
    change_set_id: Option<ChangeSetId>,
    context: AttrFuncContext,
    key: Option<String>,
    schema_variant_id: SchemaVariantId,
    func_id: FuncId,
    inputs: Vec<SiPkgAttrFuncInputView>,
    _thing_map: &mut ThingMap,
) -> PkgResult<()> {
    let prototype_id = get_prototype_for_context(ctx, context, key).await?;

    let prototype_func_id = AttributePrototype::func_id(ctx, prototype_id).await?;

    if prototype_func_id != func_id {
        AttributePrototype::update_func_by_id(ctx, prototype_id, func_id).await?;
    }

    for input in &inputs {
        match change_set_id {
            None => {
                create_attr_proto_arg(ctx, prototype_id, input, func_id, schema_variant_id).await?;
            }
            Some(_) => return Err(PkgError::WorkspaceExportNotSupported()),
        }
    }

    Ok(())
}

fn prop_kind_for_pkg_prop(pkg_prop: &SiPkgProp<'_>) -> PropKind {
    match pkg_prop {
        SiPkgProp::Array { .. } => PropKind::Array,
        SiPkgProp::Boolean { .. } => PropKind::Boolean,
        SiPkgProp::Json { .. } => PropKind::Json,
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
    parent_prop_info: Option<ParentPropInfo>,
) -> PkgResult<Prop> {
    let prop = match parent_prop_info {
        Some(parent_info) => Prop::new(
            ctx,
            &data.name,
            kind,
            data.hidden,
            data.doc_link.as_ref().map(|l| l.to_string()),
            Some(((&data.widget_kind).into(), data.widget_options.to_owned())),
            data.validation_format.clone(),
            parent_info.prop_id,
        )
        .await
        .map_err(SiPkgError::visit_prop)?,
        None => Prop::new_root(
            ctx,
            &data.name,
            kind,
            data.hidden,
            data.doc_link.as_ref().map(|l| l.to_string()),
            Some(((&data.widget_kind).into(), data.widget_options.to_owned())),
            data.validation_format.clone(),
            schema_variant_id,
        )
        .await
        .map_err(SiPkgError::visit_prop)?,
    };

    Ok(prop)
}

#[derive(Debug, Clone)]
struct ParentPropInfo {
    prop_id: PropId,
    path: PropPath,
}

async fn create_prop(
    spec: SiPkgProp<'_>,
    parent_prop_info: Option<ParentPropInfo>,
    ctx: &PropVisitContext<'_>,
) -> PkgResult<Option<ParentPropInfo>> {
    let prop = match ctx.change_set_id {
        None => {
            let data = spec.data().ok_or(PkgError::DataNotFound("prop".into()))?;
            create_dal_prop(
                ctx.ctx,
                data,
                prop_kind_for_pkg_prop(&spec),
                ctx.schema_variant_id,
                parent_prop_info,
            )
            .await?
        }
        Some(_) => {
            let parent_path = parent_prop_info
                .as_ref()
                .map(|info| info.path.to_owned())
                .unwrap_or(PropPath::new(["root"]));

            let path = parent_path.join(&PropPath::new([spec.name()]));

            match Prop::find_prop_id_by_path_opt(ctx.ctx, ctx.schema_variant_id, &path).await? {
                None => {
                    let data = spec.data().ok_or(PkgError::DataNotFound("prop".into()))?;
                    create_dal_prop(
                        ctx.ctx,
                        data,
                        prop_kind_for_pkg_prop(&spec),
                        ctx.schema_variant_id,
                        parent_prop_info,
                    )
                    .await?
                }
                Some(prop_id) => Prop::get_by_id_or_error(ctx.ctx, prop_id).await?,
            }
        }
    };

    let prop_id = prop.id();

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

    Ok(Some(ParentPropInfo {
        prop_id: prop.id(),
        path: prop.path(ctx.ctx).await?,
    }))
}

pub async fn attach_resource_payload_to_value(
    ctx: &DalContext,
    schema_variant_id: SchemaVariantId,
) -> PkgResult<()> {
    let func_id = Func::find_by_name(ctx, "si:resourcePayloadToValue")
        .await?
        .ok_or(PkgError::FuncNotFoundByName(
            "si:resourcePayloadToValue".into(),
        ))?;

    let func_argument_id = FuncArgument::find_by_name_for_func(ctx, "payload", func_id)
        .await?
        .ok_or(PkgError::FuncArgumentNotFoundByName(
            func_id,
            "payload".into(),
        ))?
        .id;

    let source_prop_id = Prop::find_prop_id_by_path(
        ctx,
        schema_variant_id,
        &PropPath::new(["root", "resource", "payload"]),
    )
    .await?;

    let target_id = {
        let resource_value_prop_id = Prop::find_prop_id_by_path(
            ctx,
            schema_variant_id,
            &PropPath::new(["root", "resource_value"]),
        )
        .await?;

        let prototype_id =
            get_prototype_for_context(ctx, AttrFuncContext::Prop(resource_value_prop_id), None)
                .await?;

        AttributePrototype::update_func_by_id(ctx, prototype_id, func_id).await?;

        prototype_id
    };

    let mut rv_input_apa_id = None;
    for apa_id in AttributePrototypeArgument::list_ids_for_prototype(ctx, target_id).await? {
        if func_argument_id
            == AttributePrototypeArgument::func_argument_id_by_id(ctx, apa_id).await?
        {
            rv_input_apa_id = Some(apa_id);
            break;
        }
    }

    match rv_input_apa_id {
        Some(apa_id) => {
            dbg!("existing apa");
            if !{
                if let Some(ValueSource::Prop(prop_id)) =
                    AttributePrototypeArgument::value_source_by_id(ctx, apa_id).await?
                {
                    prop_id == source_prop_id
                } else {
                    false
                }
            } {
                let apa = AttributePrototypeArgument::get_by_id(ctx, apa_id).await?;
                apa.set_value_from_prop_id(ctx, source_prop_id).await?;
            }
        }
        None => {
            let apa = AttributePrototypeArgument::new(ctx, target_id, func_argument_id).await?;
            apa.set_value_from_prop_id(ctx, source_prop_id).await?;
        }
    }

    Ok(())
}
