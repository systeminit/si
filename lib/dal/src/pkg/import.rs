use std::{
    collections::{
        HashMap,
        HashSet,
    },
    fmt::Debug,
    path::Path,
    str::FromStr,
};

use chrono::DateTime;
use si_events::ulid::Ulid;
use si_pkg::{
    SchemaVariantSpecPropRoot,
    SiPkg,
    SiPkgActionFunc,
    SiPkgAttrFuncInputView,
    SiPkgAuthFunc,
    SiPkgComponent,
    SiPkgEdge,
    SiPkgError,
    SiPkgFunc,
    SiPkgFuncArgument,
    SiPkgKind,
    SiPkgLeafFunction,
    SiPkgManagementFunc,
    SiPkgMetadata,
    SiPkgProp,
    SiPkgPropData,
    SiPkgSchema,
    SiPkgSchemaData,
    SiPkgSchemaVariant,
    SiPkgSocket,
    SiPkgSocketData,
    SocketSpecKind,
};
use telemetry::prelude::*;
use tokio::sync::Mutex;

use super::{
    PkgError,
    PkgResult,
};
use crate::{
    AttributePrototype,
    AttributePrototypeId,
    DalContext,
    EdgeWeightKind,
    Func,
    FuncId,
    InputSocket,
    OutputSocket,
    OutputSocketId,
    Prop,
    PropId,
    PropKind,
    Schema,
    SchemaVariant,
    SchemaVariantId,
    SocketKind,
    action::prototype::ActionPrototype,
    attribute::prototype::argument::{
        AttributePrototypeArgument,
        AttributePrototypeArgumentId,
        value_source::ValueSource,
    },
    authentication_prototype::{
        AuthenticationPrototype,
        AuthenticationPrototypeId,
    },
    func::{
        FuncKind,
        argument::FuncArgument,
        binding::attribute::AttributeBinding,
        intrinsics::IntrinsicFunc,
    },
    management::prototype::ManagementPrototype,
    module::{
        Module,
        ModuleId,
    },
    prop::PropPath,
    schema::variant::{
        SchemaVariantJson,
        leaves::{
            LeafInputLocation,
            LeafKind,
        },
    },
    socket::connection_annotation::ConnectionAnnotation,
};

#[derive(Clone, Debug)]
pub enum Thing {
    ActionPrototype(ActionPrototype),
    AuthPrototype(AuthenticationPrototype),
    // DeprecatedActionPrototype(DeprecatedActionPrototype),
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
    /// List of schema names to import. If set to `None`, the importer will import everything
    pub schemas: Option<Vec<String>>,
    pub skip_import_funcs: Option<HashMap<String, Func>>,
    /// If set to `true`, the importer will install the assets from the module
    /// but will not make a record of the install as an "installed module".
    pub no_record: bool,
    /// If set to `true` then we will set the functions to a builtin
    /// in the UI. They will be marked as such.
    pub is_builtin: bool,
    /// Locked schema variants can't be edited directly. Setting this to `true` will create
    /// editable components.
    pub create_unlocked: bool,
    /// The "schema id" for this asset, provided by the module index API
    pub schema_id: Option<Ulid>,
    /// A list of "past hashes" for this module, used to find the existing
    /// schema if a schema_id is not provided
    pub past_module_hashes: Option<Vec<String>>,
}

#[allow(clippy::too_many_arguments)]
async fn import_change_set(
    ctx: &DalContext,
    metadata: &SiPkgMetadata,
    funcs: &[SiPkgFunc<'_>],
    schemas: &[SiPkgSchema<'_>],
    _components: &[SiPkgComponent<'_>],
    _edges: &[SiPkgEdge<'_>],
    installed_module: Option<Module>,
    thing_map: &mut ThingMap,
    options: &ImportOptions,
) -> PkgResult<(
    Vec<SchemaVariantId>,
    Vec<(String, Vec<bool /*ImportAttributeSkip*/>)>,
    Vec<bool /*ImportEdgeSkip*/>,
)> {
    // Cache the intrinsic funcs pkg in case we need it.
    let unsafe_to_install_intrinsic_funcs_pkg = SiPkg::load_from_spec(IntrinsicFunc::pkg_spec()?)?;

    for func_spec in funcs {
        if let Some(intrinsic) = IntrinsicFunc::maybe_from_str(func_spec.name()) {
            let maybe_func_id = match intrinsic {
                IntrinsicFunc::ResourcePayloadToValue | IntrinsicFunc::NormalizeToArray => {
                    Func::find_id_by_name_and_kind(ctx, func_spec.name(), FuncKind::Intrinsic)
                        .await?
                }
                _ => Func::find_id_by_name(ctx, func_spec.name()).await?,
            };

            if let Some(func_id) = maybe_func_id {
                let func = Func::get_by_id(ctx, func_id).await?;

                thing_map.insert(
                    func_spec.unique_id().to_owned(),
                    Thing::Func(func.to_owned()),
                );
            } else {
                let mut override_intrinsic_func_specs =
                    unsafe_to_install_intrinsic_funcs_pkg.funcs_for_name(intrinsic.name())?;
                let override_intrinsic_func_spec = override_intrinsic_func_specs.pop().ok_or(
                    PkgError::IntrinsicFuncSpecsNoneForName(intrinsic.name().to_owned()),
                )?;
                if !override_intrinsic_func_specs.is_empty() {
                    return Err(PkgError::IntrinsicFuncSpecsMultipleForName(
                        intrinsic.name().to_owned(),
                    ));
                }

                // We need to override the unique ID so that accessors grab the correct func.
                thing_map.insert_override(
                    func_spec.unique_id().into(),
                    override_intrinsic_func_spec.unique_id().into(),
                );

                let func = import_func(
                    ctx,
                    &override_intrinsic_func_spec,
                    installed_module.clone(),
                    thing_map,
                    false,
                )
                .await?;

                let args = override_intrinsic_func_spec.arguments()?;

                if !args.is_empty() {
                    import_func_arguments(ctx, func.id, &args).await?;
                }
            }
        } else {
            if let Some(Some(func)) = options
                .skip_import_funcs
                .as_ref()
                .map(|skip_funcs| skip_funcs.get(func_spec.unique_id()))
            {
                if let Some(module) = installed_module.clone() {
                    module.create_association(ctx, func.id.into()).await?;
                }

                // We're not going to import this func but we need it in the map for lookups later
                thing_map.insert(
                    func_spec.unique_id().to_owned(),
                    Thing::Func(func.to_owned()),
                );

                continue;
            }

            let func_spec_data = func_spec
                .data()
                .ok_or(PkgError::DataNotFound(func_spec.name().into()))?;

            // If this func is a transformation, we need to see if it updates an existing func before creating it
            if func_spec_data.is_transformation() {
                let func_id = FuncId::from_str(func_spec.unique_id())?;

                // In the future we may have a strategy to update funcs that already exist, but right now we'll just skip them
                if Func::get_by_id_opt(ctx, func_id).await?.is_some() {
                    // // if no updated timestamp, skip
                    // let Some(incoming_update_timestamp) = func_spec_data.last_updated_at() else {
                    //     continue;
                    // };

                    // if incoming_update_timestamp > existing_func.timestamp.updated_at {
                    //     let func = upsert_func(ctx, func_spec, false).await?;
                    //
                    //     thing_map.insert(
                    //         func_spec.unique_id().to_owned(),
                    //         Thing::Func(func.to_owned()),
                    //     );
                    //
                    //     if let Some(module) = installed_module.clone() {
                    //         module.create_association(ctx, func.id.into()).await?;
                    //     }
                    // }

                    continue;
                }
            }

            let func = import_func(
                ctx,
                func_spec,
                installed_module.clone(),
                thing_map,
                options.create_unlocked,
            )
            .await?;

            thing_map.insert(
                func_spec.unique_id().to_owned(),
                Thing::Func(func.to_owned()),
            );

            if let Some(module) = installed_module.clone() {
                module.create_association(ctx, func.id.into()).await?;
            }

            let args = func_spec.arguments()?;

            if !args.is_empty() {
                import_func_arguments(ctx, func.id, &args).await?;
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

        debug!(
            "installing schema '{}' from {}",
            schema_spec.name(),
            metadata.name(),
        );

        let schema_variant_ids = import_schema(
            ctx,
            schema_spec,
            installed_module.clone(),
            thing_map,
            options.create_unlocked,
            options.past_module_hashes.clone(),
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
                options.schema_id,
            )
            .await?,
        )
    };
    let mut change_set_things = ThingMap::new();

    match metadata.kind() {
        SiPkgKind::Module => {
            let (installed_schema_variant_ids, _, _) = import_change_set(
                ctx,
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

    let func = if func_spec_data.is_transformation() {
        let func_id = FuncId::from_str(func_spec.unique_id())?;

        Func::upsert_with_id(
            ctx,
            func_id,
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
            func_spec_data.is_transformation(),
            func_spec_data.last_updated_at(),
        )
        .await?
    } else {
        Func::new(
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
            func_spec_data.is_transformation(),
        )
        .await?
    };

    Ok(func)
}

#[allow(unused)]
async fn upsert_func(
    ctx: &DalContext,
    func_spec: &SiPkgFunc<'_>,
    is_builtin: bool,
) -> PkgResult<Func> {
    let name = func_spec.name();

    let func_spec_data = func_spec
        .data()
        .ok_or(PkgError::DataNotFound(name.into()))?;

    let func_id = FuncId::from_str(func_spec.unique_id())?;

    let func = Func::upsert_with_id(
        ctx,
        func_id,
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
        func_spec_data.is_transformation(),
        func_spec_data.last_updated_at(),
    )
    .await?;

    // Update values that depend on the function
    let attribute_prototypes = AttributePrototype::list_ids_for_func_id(ctx, func_id).await?;
    for attribute_prototype_id in attribute_prototypes {
        AttributeBinding::enqueue_dvu_for_impacted_values(ctx, attribute_prototype_id).await?;
    }

    Ok(func)
}

pub async fn import_func(
    ctx: &DalContext,
    func_spec: &SiPkgFunc<'_>,
    installed_module: Option<Module>,
    thing_map: &mut ThingMap,
    create_unlocked: bool,
) -> PkgResult<Func> {
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

    let func = if let Some(func) = existing_func {
        func
    } else {
        let func = create_func(ctx, func_spec, false).await?;

        if !create_unlocked {
            func.lock(ctx).await?
        } else {
            func
        }
    };

    if let Some(installed_pkg) = installed_module {
        installed_pkg
            .create_association(ctx, func.id.into())
            .await?;
    }

    thing_map.insert(
        func_spec.unique_id().to_owned(),
        Thing::Func(func.to_owned()),
    );

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
    func_id: FuncId,
    func_arguments: &[SiPkgFuncArgument<'_>],
) -> PkgResult<()> {
    for arg in func_arguments {
        create_func_argument(ctx, func_id, arg).await?;
    }

    Ok(())
}

async fn create_schema(
    ctx: &DalContext,
    maybe_existing_schema_id: Option<Ulid>,
    schema_spec_data: &SiPkgSchemaData,
) -> PkgResult<Schema> {
    let schema = match maybe_existing_schema_id {
        Some(id) => Schema::new_with_id(ctx, id.into(), schema_spec_data.name()).await?,
        None => Schema::new(ctx, schema_spec_data.name()).await?,
    }
    .modify(ctx, |schema| {
        schema.ui_hidden = schema_spec_data.ui_hidden();
        Ok(())
    })
    .await?;

    Ok(schema)
}

async fn import_schema(
    ctx: &DalContext,
    schema_spec: &SiPkgSchema<'_>,
    installed_module: Option<Module>,
    thing_map: &mut ThingMap,
    create_unlocked: bool,
    past_hashes: Option<Vec<String>>,
) -> PkgResult<Vec<SchemaVariantId>> {
    let mut existing_schema: Option<Schema> = None;
    let mut existing_schema_id = None;

    if let Some(installed_module) = installed_module.as_ref() {
        existing_schema_id = installed_module.schema_id();
        // loop through past hashes to find matching schema
        if let Some(maybe_past_hashes) = past_hashes {
            for past_hash in maybe_past_hashes {
                // find if there's an existing module
                // if there is, find the asssociated schemas
                if let Some(found) = Module::find_by_root_hash(ctx, past_hash).await? {
                    match found.list_associated_schemas(ctx).await?.into_iter().next() {
                        Some(existing) => {
                            existing_schema = Some(existing);
                            break;
                        }
                        None => continue,
                    }
                }
            }
        }
    }

    let data = schema_spec
        .data()
        .ok_or(PkgError::DataNotFound("schema".into()))?;

    let schema_already_existed = existing_schema.is_some();
    let schema = match existing_schema {
        None => create_schema(ctx, existing_schema_id, data).await?,
        Some(installed_schema_record) => installed_schema_record,
    };

    // Even if the asset is already installed, we write a record of the asset installation so that
    // we can track the installed packages that share schemas.
    if let Some(module) = installed_module.clone() {
        module.create_association(ctx, schema.id().into()).await?;
    }

    import_schema_variants_for_imported_schema(
        ctx,
        schema_spec,
        installed_module,
        thing_map,
        create_unlocked,
        schema,
        schema_already_existed,
    )
    .await
}

async fn import_schema_variants_for_imported_schema(
    ctx: &DalContext,
    schema_spec: &SiPkgSchema<'_>,
    installed_module: Option<Module>,
    thing_map: &mut ThingMap,
    create_unlocked: bool,
    schema: Schema,
    schema_already_existed: bool,
) -> PkgResult<Vec<SchemaVariantId>> {
    if let Some(unique_id) = schema_spec.unique_id() {
        thing_map.insert(unique_id.to_owned(), Thing::Schema(schema.to_owned()));
    }

    let data = schema_spec
        .data()
        .ok_or(PkgError::DataNotFound("schema".into()))?;
    let variant_specs_in_schema_spec = schema_spec.variants()?;

    // Okay, so this is a weird one. Before we even begin importing variants, we need to make sure
    // everyone agrees on which one is the default. Here's the problem: if the package knows its
    // default variant, but the first one in the iterator does not have a unique ID, we need to
    // keep looking through the variants until we find one that does. If none of them have unique
    // IDs, then the first variant will be considered the default variant in the package. That
    // sounds completely nuts, but hear me out: in the land before time, packages did not know
    // their default schema variants and variant specs did not have unique IDs. How did we know
    // which one was the default back then? We didn't. We just YOLO made the first one the default.
    // Yeehaw. How do we get around this in a world where they can be mixed and matched? You just
    // don't trust anything at all whatsoever. Essentially, we will look at all the variants just
    // to be sure that we have found the default variant or we will fallback to the first.
    let default_variant_spec_index =
        match determine_default_variant_spec_index(data, &variant_specs_in_schema_spec) {
            Some(found_default_variant_spec_index) => found_default_variant_spec_index,
            None => {
                // This is only possible if there are no variants in the spec.
                return Ok(Vec::new());
            }
        };

    let mut installed_schema_variant_ids = Vec::new();
    for (index, variant_spec) in variant_specs_in_schema_spec.iter().enumerate() {
        let variant = import_schema_variant(
            ctx,
            &schema,
            schema_spec.clone(),
            variant_spec,
            installed_module.clone(),
            thing_map,
            None,
        )
        .await?;

        let variant_id = variant.id();
        installed_schema_variant_ids.push(variant_id);

        // If we are working with the package's default schema variant, then we need to figure out
        // how to handle it.
        //
        // If the schema already existed, then we must leave the package's default variant unlocked
        // (again, the one we are currently working with). This is because we need to ensure that
        // the user's local changes to the assets are not automatically clobbered when installing
        // updated assets from the module index. Not only that, but its corresponding asset func
        // needs to be unlocked so that the user can regenerate and make changes at will, so we
        // will do that too.
        //
        // If the schema did not already exist, we can safely set the default schema variant and we
        // don't care about whether or not it becomes locked.
        let mut can_lock = true;
        if index == default_variant_spec_index {
            if schema_already_existed {
                can_lock = false;
                variant.unlock_asset_func_without_copy(ctx).await?;
            } else {
                schema.set_default_variant_id(ctx, variant_id).await?;
            }
        }

        // We will only block locking the variant if we are currently working with the default
        // variant and the schema already existed locally.
        if can_lock && !create_unlocked {
            variant.lock(ctx).await?;
        }
    }

    Ok(installed_schema_variant_ids)
}

fn determine_default_variant_spec_index(
    data: &SiPkgSchemaData,
    variant_specs_in_schema_spec: &[SiPkgSchemaVariant],
) -> Option<usize> {
    let mut default_variant_spec_index = None;

    for (index, variant_spec) in variant_specs_in_schema_spec.iter().enumerate() {
        match (data.default_schema_variant(), variant_spec.unique_id()) {
            (Some(default_schema_variant_for_package), Some(variant_spec_unique_id)) => {
                // This is the ideal case: the package knows its default variant and the
                // variant spec has a unique ID that matches it.
                if default_schema_variant_for_package == variant_spec_unique_id {
                    return Some(index);
                }
            }
            (Some(_), None) => {
                // If the package knows its default variant, then we have a chance to find a
                // variant spec who has a unique ID that matches it. For now, let's just grab
                // the first variant spec we see just in case that never happens.
                if default_variant_spec_index.is_none() {
                    default_variant_spec_index = Some(index);
                }
            }
            (None, _) => {
                // If the package doesn't know its default variant, then just choose the first
                // one and return early. This is how importing older packages has worked in
                // the past.
                return Some(index);
            }
        }
    }

    // If this is "None", then there were no variants to import.
    default_variant_spec_index
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
}

async fn import_leaf_function(
    ctx: &DalContext,
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

    match thing_map.get(&leaf_func.func_unique_id().to_owned()) {
        Some(Thing::Func(func)) => {
            SchemaVariant::upsert_leaf_function(ctx, schema_variant_id, kind, &inputs, func)
                .await?;
        }
        _ => {
            return Err(PkgError::MissingFuncUniqueId(
                leaf_func.func_unique_id().to_string(),
                "error found while importing leaf function",
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
    socket_spec: SiPkgSocket<'_>,
    schema_variant_id: SchemaVariantId,
    thing_map: &mut ThingMap,
) -> PkgResult<()> {
    let (input_socket, output_socket) = {
        let data = socket_spec
            .data()
            .ok_or(PkgError::DataNotFound(socket_spec.name().into()))?;

        create_socket(ctx, data, schema_variant_id).await?
    };

    if let Some(unique_id) = socket_spec.unique_id() {
        thing_map.insert(
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

async fn create_action_prototype(
    ctx: &DalContext,
    action_func_spec: &SiPkgActionFunc<'_>,
    func_id: FuncId,
    schema_variant_id: SchemaVariantId,
) -> PkgResult<ActionPrototype> {
    let kind: crate::action::prototype::ActionKind = action_func_spec.kind().into();
    let name = action_func_spec
        .name()
        .map_or_else(|| kind.to_string(), |n| n.to_owned());
    let proto = ActionPrototype::new(ctx, kind, name, None, schema_variant_id, func_id).await?;

    Ok(proto)
}

async fn import_management_func(
    ctx: &DalContext,
    management_func_spec: &SiPkgManagementFunc<'_>,
    schema_variant_id: SchemaVariantId,
    thing_map: &ThingMap,
) -> PkgResult<ManagementPrototype> {
    let Some(Thing::Func(func)) = thing_map.get(&management_func_spec.func_unique_id().to_owned())
    else {
        return Err(PkgError::MissingFuncUniqueId(
            management_func_spec.func_unique_id().into(),
            "error found while importing management func",
        ));
    };
    let func_id = func.id;

    let prototype = ManagementPrototype::new(
        ctx,
        management_func_spec.name().to_string(),
        management_func_spec.description().map(Into::into),
        func_id,
        schema_variant_id,
    )
    .await?;

    Ok(prototype)
}

async fn import_action_func(
    ctx: &DalContext,
    action_func_spec: &SiPkgActionFunc<'_>,
    schema_variant_id: SchemaVariantId,
    thing_map: &ThingMap,
) -> PkgResult<Option<ActionPrototype>> {
    let prototype = match thing_map.get(&action_func_spec.func_unique_id().to_owned()) {
        Some(Thing::Func(func)) => {
            let func_id = func.id;

            if let Some(unique_id) = action_func_spec.unique_id() {
                match thing_map.get(&unique_id.to_owned()) {
                    Some(Thing::ActionPrototype(_prototype)) => {
                        return Err(PkgError::WorkspaceExportNotSupported());
                    }
                    _ => {
                        if action_func_spec.deleted() {
                            None
                        } else {
                            let action_prototype = create_action_prototype(
                                ctx,
                                action_func_spec,
                                func_id,
                                schema_variant_id,
                            )
                            .await?;
                            Some(action_prototype)
                        }
                    }
                }
            } else {
                let action_prototype =
                    create_action_prototype(ctx, action_func_spec, func_id, schema_variant_id)
                        .await?;
                Some(action_prototype)
            }
        }
        _ => {
            return Err(PkgError::MissingFuncUniqueId(
                action_func_spec.func_unique_id().into(),
                "error found while importing action func",
            ));
        }
    };

    Ok(prototype)
}

async fn import_auth_func(
    ctx: &DalContext,
    func_spec: &SiPkgAuthFunc<'_>,
    schema_variant_id: SchemaVariantId,
    thing_map: &ThingMap,
) -> PkgResult<Option<AuthenticationPrototype>> {
    let prototype = match thing_map.get(&func_spec.func_unique_id().to_owned()) {
        Some(Thing::Func(func)) => {
            let func_id = func.id;

            if let Some(unique_id) = func_spec.unique_id() {
                match thing_map.get(&unique_id.to_owned()) {
                    Some(Thing::AuthPrototype(_prototype)) => {
                        return Err(PkgError::WorkspaceExportNotSupported());
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
                "error found while importing auth func",
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

    let parent_info = ParentPropInfo {
        prop_id: prop_root_prop_id,
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

/// Import only new [`Funcs`](Func) and return a [`ThingMap`] with all [`Funcs`](Func) included. This is used for
/// importing a standalone [`SchemaVariant`].
pub async fn import_only_new_funcs(
    ctx: &DalContext,
    funcs: Vec<SiPkgFunc<'_>>,
) -> PkgResult<ThingMap> {
    let mut thing_map = ThingMap::new();

    // Cache the intrinsic funcs pkg in case we need it.
    let unsafe_to_install_intrinsic_funcs_pkg = SiPkg::load_from_spec(IntrinsicFunc::pkg_spec()?)?;

    // Iterate through all func specs. If the func is an intrinsic, we need to handle it
    // separately. If it is any other kind of func, we find or create the func and find or create
    // its arguments.
    for func_spec in funcs {
        if let Some(intrinsic) = IntrinsicFunc::maybe_from_str(func_spec.name()) {
            if intrinsic == IntrinsicFunc::ResourcePayloadToValue
                || intrinsic == IntrinsicFunc::NormalizeToArray
            {
                if let Some(func_id) =
                    Func::find_id_by_name_and_kind(ctx, func_spec.name(), FuncKind::Intrinsic)
                        .await?
                {
                    let func = Func::get_by_id(ctx, func_id).await?;
                    thing_map.insert(func_spec.unique_id().into(), Thing::Func(func));
                } else {
                    let mut override_intrinsic_func_specs =
                        unsafe_to_install_intrinsic_funcs_pkg.funcs_for_name(intrinsic.name())?;
                    let override_intrinsic_func_spec = override_intrinsic_func_specs.pop().ok_or(
                        PkgError::IntrinsicFuncSpecsNoneForName(intrinsic.name().to_owned()),
                    )?;
                    if !override_intrinsic_func_specs.is_empty() {
                        return Err(PkgError::IntrinsicFuncSpecsMultipleForName(
                            intrinsic.name().to_owned(),
                        ));
                    }

                    // Use the override func spec to create the func.
                    let func = create_func(ctx, &override_intrinsic_func_spec, false).await?;

                    // Find or create the func arguments for the provided spec.
                    for argument in func_spec.arguments()? {
                        if FuncArgument::find_by_name_for_func(ctx, argument.name(), func.id)
                            .await?
                            .is_none()
                        {
                            create_func_argument(ctx, func.id, &argument).await?;
                        }
                    }

                    // However, use the _provided_ func spec when inserting into the thing map. Why
                    // not use the override? Because we are initializing the thing map here and the
                    // users of the thing map don't need the "overrides" re-direction: they just
                    // have the func they need here.
                    thing_map.insert(func_spec.unique_id().into(), Thing::Func(func));
                }
            } else {
                let func_id = Func::find_id_by_name(ctx, func_spec.name())
                    .await?
                    .ok_or(PkgError::MissingIntrinsicFunc(func_spec.name().to_owned()))?;

                let func = Func::get_by_id(ctx, func_id).await?;
                thing_map.insert(func_spec.unique_id().into(), Thing::Func(func));
            }
        } else {
            // Find or create the func for the provided spec.
            let func = if let Some(func) =
                Func::get_by_id_opt(ctx, FuncId::from_str(func_spec.unique_id())?).await?
            {
                func
            } else {
                create_func(ctx, &func_spec, false).await?
            };

            // Find or create the func arguments for the provided spec.
            for argument in func_spec.arguments()? {
                if FuncArgument::find_by_name_for_func(ctx, argument.name(), func.id)
                    .await?
                    .is_none()
                {
                    create_func_argument(ctx, func.id, &argument).await?;
                }
            }

            thing_map.insert(func_spec.unique_id().into(), Thing::Func(func));
        }
    }

    Ok(thing_map)
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn import_schema_variant(
    ctx: &DalContext,
    schema: &Schema,
    schema_spec: SiPkgSchema<'_>,
    variant_spec: &SiPkgSchemaVariant<'_>,
    installed_module: Option<Module>,
    thing_map: &mut ThingMap,
    installed_schema_variant: Option<SchemaVariant>,
) -> PkgResult<SchemaVariant> {
    let mut existing_schema_variant: Option<SchemaVariant> = None;
    if let Some(installed_pkg) = installed_module.clone() {
        let associated_schema_variants = installed_pkg.list_associated_schema_variants(ctx).await?;
        let mut maybe_matching_schema_variant: Vec<SchemaVariant> = associated_schema_variants
            .into_iter()
            .filter(|s| s.version() == schema_spec.name())
            .collect();
        if let Some(matching_schema_variant) = maybe_matching_schema_variant.pop() {
            existing_schema_variant = Some(matching_schema_variant);
        }
    }

    if let Some(existing_sv) = installed_schema_variant {
        existing_schema_variant = Some(existing_sv);
    }

    let schema_variant = if let Some(variant) = existing_schema_variant {
        variant
    } else {
        let spec = schema_spec.to_spec().await?;
        let metadata = SchemaVariantJson::metadata_from_spec(spec)?;

        let mut asset_func_id: Option<FuncId> = None;
        if let Some(variant_spec_data) = variant_spec.data() {
            let func_unique_id = variant_spec_data.func_unique_id().to_owned();
            if let Thing::Func(asset_func) =
                thing_map
                    .get(&func_unique_id)
                    .ok_or(PkgError::MissingFuncUniqueId(
                        func_unique_id.to_string(),
                        "error found while importing schema variant",
                    ))?
            {
                asset_func_id = Some(asset_func.id)
            }
        }
        let old_versions = ["v0", "v1", "v2"];
        let version_date = if old_versions.contains(&variant_spec.version()) {
            let date = DateTime::UNIX_EPOCH;
            format!("{}", date.format("%Y%m%d%H%M%S"))
        } else {
            variant_spec.version().to_owned()
        };

        SchemaVariant::new(
            ctx,
            schema.id(),
            version_date,
            metadata.display_name,
            metadata.category,
            metadata.color,
            metadata.component_type,
            metadata.link,
            metadata.description,
            asset_func_id,
            variant_spec.is_builtin(),
        )
        .await?
        .0
    };

    if let Some(module) = installed_module.clone() {
        module
            .create_association(ctx, schema_variant.id().into())
            .await?;
    }

    if let Some(unique_id) = variant_spec.unique_id() {
        thing_map.insert(
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

    let domain_prop_id =
        Prop::find_prop_id_by_path(ctx, schema_variant.id(), &PropPath::new(["root", "domain"]))
            .await?;

    side_effects.extend(
        create_props(
            ctx,
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
            variant_spec,
            SchemaVariantSpecPropRoot::Secrets,
            secrets_prop_id,
            schema_variant.id(),
        )
        .await?,
    );

    if !variant_spec.secret_definitions()?.is_empty() {
        let root_prop_id =
            Prop::find_prop_id_by_path(ctx, schema_variant.id(), &PropPath::new(["root"])).await?;

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
        import_socket(ctx, socket, schema_variant.id(), thing_map).await?;
    }

    for action_func in &variant_spec.action_funcs()? {
        let prototype =
            import_action_func(ctx, action_func, schema_variant.id(), thing_map).await?;

        if let (Some(prototype), Some(unique_id)) = (prototype, action_func.unique_id()) {
            thing_map.insert(unique_id.to_owned(), Thing::ActionPrototype(prototype));
        }
    }

    for auth_func in &variant_spec.auth_funcs()? {
        let prototype = import_auth_func(ctx, auth_func, schema_variant.id(), thing_map).await?;

        if let (Some(prototype), Some(unique_id)) = (prototype, auth_func.unique_id()) {
            thing_map.insert(unique_id.to_owned(), Thing::AuthPrototype(prototype));
        }
    }

    for leaf_func in variant_spec.leaf_functions()? {
        import_leaf_function(ctx, leaf_func, schema_variant.id(), thing_map).await?;
    }

    for management_func in variant_spec.management_funcs()? {
        import_management_func(ctx, &management_func, schema_variant.id(), thing_map).await?;
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
        import_attr_func_for_prop(ctx, schema_variant.id(), attr_func, None, thing_map).await?;
    }

    for (key, map_key_func) in side_effects.map_key_funcs {
        import_attr_func_for_prop(ctx, schema_variant.id(), map_key_func, Some(key), thing_map)
            .await?;
    }

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
    schema_variant_id: SchemaVariantId,
    AttrFuncInfo {
        func_unique_id,
        prop_id,
        inputs,
    }: AttrFuncInfo,
    key: Option<String>,
    thing_map: &mut ThingMap,
) -> PkgResult<()> {
    match thing_map.get(&func_unique_id.to_owned()) {
        Some(Thing::Func(func)) => {
            import_attr_func(
                ctx,
                AttrFuncContext::Prop(prop_id),
                key,
                schema_variant_id,
                func.id,
                inputs,
                thing_map,
            )
            .await?;
        }
        _ => {
            return Err(PkgError::MissingFuncUniqueId(
                func_unique_id.to_string(),
                "error found while importing attribute func for prop",
            ));
        }
    }

    Ok(())
}

async fn import_attr_func_for_output_socket(
    ctx: &DalContext,
    schema_variant_id: SchemaVariantId,
    output_socket_id: OutputSocketId,
    func_unique_id: &str,
    inputs: Vec<SiPkgAttrFuncInputView>,
    thing_map: &mut ThingMap,
) -> PkgResult<()> {
    match thing_map.get(&func_unique_id.to_owned()) {
        Some(Thing::Func(func)) => {
            import_attr_func(
                ctx,
                AttrFuncContext::OutputSocket(output_socket_id),
                None,
                schema_variant_id,
                func.id,
                inputs,
                thing_map,
            )
            .await?;
        }
        _ => {
            return Err(PkgError::MissingFuncUniqueId(
                func_unique_id.to_string(),
                "import attribute func for output socket",
            ));
        }
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
        let map_prop = Prop::get_by_id(ctx, map_prop_id).await?;

        if map_prop.kind != PropKind::Map {
            return Err(PkgError::AttributeFuncForKeySetOnWrongKind(
                map_prop_id,
                key.to_owned().expect("check above ensures this is some"),
                map_prop.kind,
            ));
        }

        let element_prop_id = Prop::element_prop_id(ctx, map_prop.id).await?;
        Ok(
            match AttributePrototype::find_for_prop(ctx, element_prop_id, &key).await? {
                None => {
                    let unset_func_id = Func::find_intrinsic(ctx, IntrinsicFunc::Unset).await?;
                    let prototype_id = AttributePrototype::new(ctx, unset_func_id).await?.id();
                    Prop::add_edge_to_attribute_prototype(
                        ctx,
                        element_prop_id,
                        prototype_id,
                        EdgeWeightKind::Prototype(key),
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
            AttributePrototypeArgument::new(ctx, prototype_id, arg.id, prop_id).await?
        }
        SiPkgAttrFuncInputView::InputSocket { socket_name, .. } => {
            let input_socket = InputSocket::find_with_name(ctx, socket_name, schema_variant_id)
                .await?
                .ok_or(PkgError::MissingInputSocketName(socket_name.to_owned()))?;
            AttributePrototypeArgument::new(ctx, prototype_id, arg.id, input_socket.id()).await?
        }
        SiPkgAttrFuncInputView::OutputSocket {
            name, socket_name, ..
        } => {
            return Err(PkgError::TakingOutputSocketAsInputForPropUnsupported(
                name.to_owned(),
                socket_name.to_owned(),
            ));
        }
    }
    .id())
}

#[derive(Debug, Clone)]
pub enum AttrFuncContext {
    Prop(PropId),
    OutputSocket(OutputSocketId),
}

#[allow(clippy::too_many_arguments)]
async fn import_attr_func(
    ctx: &DalContext,
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
        create_attr_proto_arg(ctx, prototype_id, input, func_id, schema_variant_id).await?;
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
        SiPkgProp::Float { .. } => PropKind::Float,
        SiPkgProp::Object { .. } => PropKind::Object,
        SiPkgProp::String { .. } => PropKind::String,
    }
}

async fn create_dal_prop(
    ctx: &DalContext,
    SiPkgPropData {
        name,
        default_value: _,  // unused for some reason?
        func_unique_id: _, // unused for some reason?
        widget_kind,
        widget_options,
        doc_link,
        hidden,
        documentation,
        validation_format,
        ui_optionals,
    }: &SiPkgPropData,
    kind: PropKind,
    schema_variant_id: SchemaVariantId,
    parent_prop_info: Option<ParentPropInfo>,
) -> PkgResult<Prop> {
    let ui_optionals = ui_optionals
        .iter()
        .map(|(key, value)| (key.clone(), value.clone().into()))
        .collect();
    let prop = match parent_prop_info {
        Some(parent_info) => Prop::new(
            ctx,
            name.as_str(),
            kind,
            *hidden,
            doc_link.as_ref().map(|l| l.to_string()),
            documentation.clone(),
            Some((widget_kind.into(), widget_options.to_owned())),
            validation_format.clone(),
            ui_optionals,
            parent_info.prop_id,
        )
        .await
        .map_err(SiPkgError::visit_prop)?,
        None => Prop::new_root(
            ctx,
            name.as_str(),
            kind,
            *hidden,
            doc_link.as_ref().map(|l| l.to_string()),
            documentation.clone(),
            Some((widget_kind.into(), widget_options.to_owned())),
            validation_format.clone(),
            ui_optionals,
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
}

async fn create_prop(
    spec: SiPkgProp<'_>,
    parent_prop_info: Option<ParentPropInfo>,
    ctx: &PropVisitContext<'_>,
) -> PkgResult<Option<ParentPropInfo>> {
    let prop = {
        let data = spec.data().ok_or(PkgError::DataNotFound("prop".into()))?;
        create_dal_prop(
            ctx.ctx,
            data,
            prop_kind_for_pkg_prop(&spec),
            ctx.schema_variant_id,
            parent_prop_info,
        )
        .await?
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

    Ok(Some(ParentPropInfo { prop_id: prop.id() }))
}

pub async fn attach_resource_payload_to_value(
    ctx: &DalContext,
    schema_variant_id: SchemaVariantId,
) -> PkgResult<()> {
    let name = "si:resourcePayloadToValue";
    let func_id = if let Some(func_id) =
        Func::find_id_by_name_and_kind(ctx, name, FuncKind::Intrinsic).await?
    {
        func_id
    } else {
        trace!(
            "installing the intrinsic version of 'si:resourcePayloadToValue' (neither found nor was specified in the package spec)"
        );

        // If we did not find it by this point, the package did not specify it.
        let unsafe_to_install_intrinsic_funcs_pkg =
            SiPkg::load_from_spec(IntrinsicFunc::pkg_spec()?)?;
        let mut intrinsic_func_specs =
            unsafe_to_install_intrinsic_funcs_pkg.funcs_for_name(name)?;
        let intrinsic_func_spec = intrinsic_func_specs
            .pop()
            .ok_or(PkgError::IntrinsicFuncSpecsNoneForName(name.to_owned()))?;
        if !intrinsic_func_specs.is_empty() {
            return Err(PkgError::IntrinsicFuncSpecsMultipleForName(name.to_owned()));
        }

        // Create the func and arguments without interacting with the thing map and import flow. We
        // do that because the package did not specify it.
        let func = create_func(ctx, &intrinsic_func_spec, false).await?;
        for argument in intrinsic_func_spec.arguments()? {
            if FuncArgument::find_by_name_for_func(ctx, argument.name(), func.id)
                .await?
                .is_none()
            {
                create_func_argument(ctx, func.id, &argument).await?;
            }
        }

        func.id
    };

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
            if !{
                if let Some(ValueSource::Prop(prop_id)) =
                    AttributePrototypeArgument::value_source_opt(ctx, apa_id).await?
                {
                    prop_id == source_prop_id
                } else {
                    false
                }
            } {
                AttributePrototypeArgument::set_value_source(ctx, apa_id, source_prop_id).await?;
            }
        }
        None => {
            AttributePrototypeArgument::new(ctx, target_id, func_argument_id, source_prop_id)
                .await?;
        }
    }

    Ok(())
}
