use std::path::PathBuf;
use telemetry::prelude::*;
use tokio::sync::Mutex;

use si_pkg::{
    FuncUniqueId, SchemaVariantSpecPropRoot, SiPkg, SiPkgAttrFuncInputView, SiPkgCommandFunc,
    SiPkgError, SiPkgFunc, SiPkgFuncDescription, SiPkgLeafFunction, SiPkgProp, SiPkgSchema,
    SiPkgSchemaVariant, SiPkgSocket, SiPkgValidation, SiPkgWorkflow, SocketSpecKind,
};

use crate::{
    component::ComponentKind,
    func::{binding::FuncBinding, binding_return_value::FuncBindingReturnValue},
    installed_pkg::{
        InstalledPkg, InstalledPkgAsset, InstalledPkgAssetKind, InstalledPkgAssetTyped,
        InstalledPkgId,
    },
    schema::{variant::leaves::LeafInputLocation, SchemaUiMenu},
    validation::{create_validation, Validation, ValidationKind},
    ActionPrototype, ActionPrototypeContext, AttributePrototypeArgument, AttributeReadContext,
    AttributeValue, AttributeValueError, CommandPrototype, CommandPrototypeContext, DalContext,
    ExternalProvider, ExternalProviderId, Func, FuncArgument, FuncDescription,
    FuncDescriptionContents, FuncError, FuncId, InternalProvider, Prop, PropId, PropKind, Schema,
    SchemaId, SchemaVariant, SchemaVariantError, SchemaVariantId, StandardModel, WorkflowPrototype,
    WorkflowPrototypeContext,
};

use super::{PkgError, PkgResult};

type FuncMap = std::collections::HashMap<FuncUniqueId, Func>;

pub async fn import_pkg_from_pkg(ctx: &DalContext, pkg: &SiPkg, file_name: &str) -> PkgResult<()> {
    // We have to write the installed_pkg row first, so that we have an id, and rely on transaction
    // semantics to remove the row if anything in the installation process fails
    let root_hash = pkg.hash()?.to_string();

    if !InstalledPkg::find_by_attr(ctx, "root_hash", &root_hash)
        .await?
        .is_empty()
    {
        return Err(PkgError::PackageAlreadyInstalled(root_hash));
    }

    let installed_pkg = InstalledPkg::new(ctx, &file_name, pkg.hash()?.to_string()).await?;

    let mut funcs_by_unique_id = FuncMap::new();
    for func_spec in pkg.funcs()? {
        let unique_id = func_spec.unique_id();
        let func = if crate::func::is_intrinsic(func_spec.name()) {
            // We don't want to create intrinsic funcs but we need them in our map
            Func::find_by_name(ctx, func_spec.name())
                .await?
                .ok_or(PkgError::MissingIntrinsicFunc(func_spec.name().to_string()))?
        } else {
            create_func(ctx, func_spec, *installed_pkg.id()).await?
        };

        funcs_by_unique_id.insert(unique_id, func);
    }

    // TODO: gather up a record of what wasn't installed and why (the id of the package that
    // already contained the schema or variant)
    for schema_spec in pkg.schemas()? {
        create_schema(ctx, schema_spec, *installed_pkg.id(), &funcs_by_unique_id).await?;
    }

    Ok(())
}

pub async fn import_pkg(
    ctx: &DalContext,
    pkg_file_path: impl Into<PathBuf> + Clone,
) -> PkgResult<SiPkg> {
    let pkg_file_path_str = Into::<PathBuf>::into(pkg_file_path.clone())
        .to_string_lossy()
        .to_string();

    let pkg = SiPkg::load_from_file(pkg_file_path).await?;

    import_pkg_from_pkg(ctx, &pkg, &pkg_file_path_str).await?;

    Ok(pkg)
}

async fn create_func(
    ctx: &DalContext,
    func_spec: SiPkgFunc<'_>,
    installed_pkg_id: InstalledPkgId,
) -> PkgResult<Func> {
    let hash = func_spec.hash().to_string();
    let existing_func =
        InstalledPkgAsset::list_for_kind_and_hash(ctx, InstalledPkgAssetKind::Func, &hash)
            .await?
            .pop();

    let func = match existing_func {
        Some(installed_func_record) => match installed_func_record.as_installed_func()? {
            InstalledPkgAssetTyped::Func { id, .. } => match Func::get_by_id(ctx, &id).await? {
                Some(func) => func,
                None => return Err(PkgError::InstalledFuncMissing(id)),
            },
            _ => unreachable!(),
        },
        None => {
            let name = func_spec.name();

            // How to handle name conflicts?
            let mut func = Func::new(
                ctx,
                name,
                func_spec.backend_kind().into(),
                func_spec.response_type().into(),
            )
            .await?;

            func.set_display_name(ctx, func_spec.display_name()).await?;
            func.set_code_base64(ctx, Some(func_spec.code_base64()))
                .await?;
            func.set_description(ctx, func_spec.description()).await?;
            func.set_handler(ctx, Some(func_spec.handler())).await?;
            func.set_hidden(ctx, func.hidden()).await?;
            func.set_link(ctx, func_spec.link().map(|l| l.to_string()))
                .await?;

            // If the func exists above with the matching hash, we assume the arguments are correct
            // and only create the arguments if we're creating the function
            for arg in func_spec.arguments()? {
                FuncArgument::new(
                    ctx,
                    arg.name(),
                    arg.kind().into(),
                    arg.element_kind().cloned().map(|kind| kind.into()),
                    *func.id(),
                )
                .await?;
            }

            func
        }
    };

    InstalledPkgAsset::new(
        ctx,
        InstalledPkgAssetTyped::new_for_func(*func.id(), installed_pkg_id, hash),
    )
    .await?;

    Ok(func)
}

async fn create_schema(
    ctx: &DalContext,
    schema_spec: SiPkgSchema<'_>,
    installed_pkg_id: InstalledPkgId,
    func_map: &FuncMap,
) -> PkgResult<()> {
    let hash = schema_spec.hash().to_string();
    let existing_schema =
        InstalledPkgAsset::list_for_kind_and_hash(ctx, InstalledPkgAssetKind::Schema, &hash)
            .await?
            .pop();

    let mut schema = match existing_schema {
        None => {
            let mut schema = Schema::new(ctx, schema_spec.name(), &ComponentKind::Standard).await?;
            schema.set_ui_hidden(ctx, schema_spec.ui_hidden()).await?;
            let ui_menu = SchemaUiMenu::new(
                ctx,
                schema_spec
                    .category_name()
                    .unwrap_or_else(|| schema_spec.name()),
                schema_spec.category(),
            )
            .await?;
            ui_menu.set_schema(ctx, schema.id()).await?;

            schema
        }
        Some(installed_schema_record) => match installed_schema_record.as_installed_schema()? {
            InstalledPkgAssetTyped::Schema { id, .. } => match Schema::get_by_id(ctx, &id).await? {
                Some(schema) => schema,
                None => return Err(PkgError::InstalledSchemaMissing(id)),
            },
            _ => unreachable!(),
        },
    };

    // Even if the asset is already installed, we write a record of the asset installation so that
    // we can track the installed packages that share schemas.
    InstalledPkgAsset::new(
        ctx,
        InstalledPkgAssetTyped::new_for_schema(*schema.id(), installed_pkg_id, hash),
    )
    .await?;

    for variant_spec in schema_spec.variants()? {
        create_schema_variant(ctx, &mut schema, variant_spec, installed_pkg_id, func_map).await?;
    }

    Ok(())
}

#[derive(Clone, Debug)]
struct AttrFuncInfo {
    func_unique_id: FuncUniqueId,
    prop_id: PropId,
    inputs: Vec<SiPkgAttrFuncInputView>,
}

#[derive(Clone, Debug)]
enum DefaultValueInfo {
    String {
        prop_id: PropId,
        default_value: String,
    },
    Number {
        prop_id: PropId,
        default_value: i64,
    },
    Boolean {
        prop_id: PropId,
        default_value: bool,
    },
}

struct PropVisitContext<'a, 'b> {
    pub ctx: &'a DalContext,
    pub schema_id: SchemaId,
    pub schema_variant_id: SchemaVariantId,
    pub func_map: &'b FuncMap,
    pub attr_funcs: Mutex<Vec<AttrFuncInfo>>,
    pub default_values: Mutex<Vec<DefaultValueInfo>>,
}

async fn create_func_description(
    ctx: &DalContext,
    func_description: SiPkgFuncDescription<'_>,
    schema_variant_id: SchemaVariantId,
    func_map: &FuncMap,
) -> PkgResult<()> {
    let contents: FuncDescriptionContents =
        serde_json::from_value(func_description.contents().to_owned())?;

    let func =
        func_map
            .get(&func_description.func_unique_id())
            .ok_or(PkgError::MissingFuncUniqueId(
                func_description.func_unique_id().to_string(),
            ))?;

    FuncDescription::new(ctx, *func.id(), schema_variant_id, contents).await?;

    Ok(())
}

async fn create_leaf_function(
    ctx: &DalContext,
    leaf_func: SiPkgLeafFunction<'_>,
    schema_variant_id: SchemaVariantId,
    func_map: &FuncMap,
) -> PkgResult<()> {
    let inputs: Vec<LeafInputLocation> = leaf_func
        .inputs()
        .iter()
        .map(|input| input.into())
        .collect();

    match func_map.get(&leaf_func.func_unique_id()) {
        Some(func) => {
            SchemaVariant::upsert_leaf_function(
                ctx,
                schema_variant_id,
                None,
                leaf_func.leaf_kind().into(),
                &inputs,
                func,
            )
            .await?;
        }
        None => {
            return Err(PkgError::MissingFuncUniqueId(
                leaf_func.func_unique_id().to_string(),
            ))
        }
    };

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
    let (func_binding, func_binding_return_value) =
        FuncBinding::create_and_execute(ctx, serde_json::json![{ "identity": null }], func_id)
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
    socket_spec: SiPkgSocket<'_>,
    schema_id: SchemaId,
    schema_variant_id: SchemaVariantId,
    func_map: &FuncMap,
) -> PkgResult<()> {
    let (identity_func, identity_func_binding, identity_fbrv, _) = get_identity_func(ctx).await?;

    let name = socket_spec.name();
    let arity = socket_spec.arity();

    let mut socket = match socket_spec.kind() {
        SocketSpecKind::Input => {
            let (_, socket) = InternalProvider::new_explicit_with_socket(
                ctx,
                schema_variant_id,
                name,
                *identity_func.id(),
                *identity_func_binding.id(),
                *identity_fbrv.id(),
                arity.into(),
                false,
            )
            .await?;

            if let Some(func_unique_id) = socket_spec.func_unique_id() {
                dbg!(
                    "Input socket that is set by a function?",
                    func_unique_id,
                    socket_spec.inputs()?
                );
            }

            socket
        }
        SocketSpecKind::Output => {
            let (ep, socket) = ExternalProvider::new_with_socket(
                ctx,
                schema_id,
                schema_variant_id,
                name,
                None,
                *identity_func.id(),
                *identity_func_binding.id(),
                *identity_fbrv.id(),
                arity.into(),
                false,
            )
            .await?;

            if let Some(func_unique_id) = socket_spec.func_unique_id() {
                create_attribute_function_for_output_socket(
                    ctx,
                    schema_variant_id,
                    *ep.id(),
                    func_unique_id,
                    socket_spec.inputs()?.drain(..).map(Into::into).collect(),
                    func_map,
                )
                .await?;
            }

            socket
        }
    };

    socket.set_ui_hidden(ctx, socket_spec.ui_hidden()).await?;

    Ok(())
}

async fn create_command_func(
    ctx: &DalContext,
    command_func_spec: SiPkgCommandFunc<'_>,
    schema_variant_id: SchemaVariantId,
    func_map: &FuncMap,
) -> PkgResult<()> {
    let func =
        func_map
            .get(&command_func_spec.func_unique_id())
            .ok_or(PkgError::MissingFuncUniqueId(
                command_func_spec.func_unique_id().to_string(),
            ))?;

    CommandPrototype::new(
        ctx,
        *func.id(),
        CommandPrototypeContext {
            schema_variant_id,
            ..Default::default()
        },
    )
    .await?;

    Ok(())
}

async fn create_workflow(
    ctx: &DalContext,
    workflow_spec: SiPkgWorkflow<'_>,
    schema_id: SchemaId,
    schema_variant_id: SchemaVariantId,
    func_map: &FuncMap,
) -> PkgResult<()> {
    let func =
        func_map
            .get(&workflow_spec.func_unique_id())
            .ok_or(PkgError::MissingFuncUniqueId(
                workflow_spec.func_unique_id().to_string(),
            ))?;

    let workflow_proto = WorkflowPrototype::new(
        ctx,
        *func.id(),
        serde_json::Value::Null,
        WorkflowPrototypeContext {
            schema_id,
            schema_variant_id,
            ..Default::default()
        },
        workflow_spec.title(),
    )
    .await?;

    for action_spec in workflow_spec.actions()? {
        ActionPrototype::new(
            ctx,
            *workflow_proto.id(),
            action_spec.name(),
            action_spec.kind().into(),
            ActionPrototypeContext {
                schema_id,
                schema_variant_id,
                ..Default::default()
            },
        )
        .await?;
    }

    Ok(())
}

async fn create_props(
    ctx: &DalContext,
    variant_spec: &SiPkgSchemaVariant<'_>,
    prop_root: SchemaVariantSpecPropRoot,
    prop_root_prop_id: PropId,
    schema_id: SchemaId,
    schema_variant_id: SchemaVariantId,
    func_map: &FuncMap,
) -> PkgResult<(Vec<AttrFuncInfo>, Vec<DefaultValueInfo>)> {
    let context = PropVisitContext {
        ctx,
        schema_id,
        schema_variant_id,
        func_map,
        attr_funcs: Mutex::new(vec![]),
        default_values: Mutex::new(vec![]),
    };

    variant_spec
        .visit_prop_tree(prop_root, create_prop, Some(prop_root_prop_id), &context)
        .await?;

    Ok((
        context.attr_funcs.into_inner(),
        context.default_values.into_inner(),
    ))
}

async fn create_schema_variant(
    ctx: &DalContext,
    schema: &mut Schema,
    variant_spec: SiPkgSchemaVariant<'_>,
    installed_pkg_id: InstalledPkgId,
    func_map: &FuncMap,
) -> PkgResult<()> {
    let hash = variant_spec.hash().to_string();
    let existing_schema_variant =
        InstalledPkgAsset::list_for_kind_and_hash(ctx, InstalledPkgAssetKind::SchemaVariant, &hash)
            .await?
            .pop();

    let variant_id = match existing_schema_variant {
        Some(installed_sv_record) => match installed_sv_record.as_installed_schema_variant()? {
            InstalledPkgAssetTyped::SchemaVariant { id, .. } => id,
            _ => unreachable!(
                "the as_installed_schema_variant method ensures we cannot hit this branch"
            ),
        },
        None => {
            let (mut schema_variant, root_prop) =
                SchemaVariant::new(ctx, *schema.id(), variant_spec.name()).await?;

            schema
                .set_default_schema_variant_id(ctx, Some(schema_variant.id()))
                .await?;

            if let Some(color) = variant_spec.color() {
                schema_variant.set_color(ctx, color.to_owned()).await?;
            }

            let (domain_attr_funcs, domain_default_values) = create_props(
                ctx,
                &variant_spec,
                SchemaVariantSpecPropRoot::Domain,
                root_prop.domain_prop_id,
                *schema.id(),
                *schema_variant.id(),
                func_map,
            )
            .await?;

            let (rv_attr_funcs, rv_default_values) = match schema_variant
                .find_prop(ctx, &["root", "resource", "value"])
                .await
            {
                Ok(resource_value_prop) => {
                    create_props(
                        ctx,
                        &variant_spec,
                        SchemaVariantSpecPropRoot::ResourceValue,
                        *resource_value_prop.id(),
                        *schema.id(),
                        *schema_variant.id(),
                        func_map,
                    )
                    .await?
                }
                Err(SchemaVariantError::PropNotFoundAtPath(_, _, _)) => {
                    warn!("Cannot find /root/resource/value prop, so skipping creating props under the resource value. If the /root/resource/value pr has been merged, this should be an error!");
                    (vec![], vec![])
                }
                Err(err) => Err(err)?,
            };

            schema_variant
                .finalize(ctx, Some(variant_spec.component_type().into()))
                .await?;

            for command_func in variant_spec.command_funcs()? {
                create_command_func(ctx, command_func, *schema_variant.id(), func_map).await?;
            }

            for leaf_func in variant_spec.leaf_functions()? {
                create_leaf_function(ctx, leaf_func, *schema_variant.id(), func_map).await?;
            }

            for func_description in variant_spec.func_descriptions()? {
                create_func_description(ctx, func_description, *schema_variant.id(), func_map)
                    .await?;
            }

            for workflow in variant_spec.workflows()? {
                create_workflow(ctx, workflow, *schema.id(), *schema_variant.id(), func_map)
                    .await?;
            }

            for socket in variant_spec.sockets()? {
                create_socket(ctx, socket, *schema.id(), *schema_variant.id(), func_map).await?;
            }

            for si_prop_func in variant_spec.si_prop_funcs()? {
                let prop = schema_variant
                    .find_prop(ctx, &si_prop_func.kind().prop_path())
                    .await?;
                create_attribute_function_for_prop(
                    ctx,
                    *schema_variant.id(),
                    AttrFuncInfo {
                        func_unique_id: si_prop_func.func_unique_id(),
                        prop_id: *prop.id(),
                        inputs: si_prop_func
                            .inputs()?
                            .iter()
                            .map(|input| input.to_owned().into())
                            .collect(),
                    },
                    func_map,
                )
                .await?;
            }

            for default_value_info in domain_default_values
                .into_iter()
                .chain(rv_default_values.into_iter())
            {
                set_default_value(ctx, default_value_info).await?;
            }

            for attr_func in domain_attr_funcs
                .into_iter()
                .chain(rv_attr_funcs.into_iter())
            {
                create_attribute_function_for_prop(ctx, *schema_variant.id(), attr_func, func_map)
                    .await?;
            }

            schema_variant
                .finalize(ctx, Some(variant_spec.component_type().into()))
                .await?;

            *schema_variant.id()
        }
    };

    InstalledPkgAsset::new(
        ctx,
        InstalledPkgAssetTyped::new_for_schema_variant(variant_id, installed_pkg_id, hash),
    )
    .await?;

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

async fn create_attribute_function_for_prop(
    ctx: &DalContext,
    schema_variant_id: SchemaVariantId,
    AttrFuncInfo {
        func_unique_id,
        prop_id,
        inputs,
    }: AttrFuncInfo,
    func_map: &FuncMap,
) -> PkgResult<()> {
    let func = func_map
        .get(&func_unique_id)
        .ok_or(PkgError::MissingFuncUniqueId(func_unique_id.to_string()))?;

    create_attribute_function(
        ctx,
        AttributeReadContext {
            prop_id: Some(prop_id),
            ..Default::default()
        },
        schema_variant_id,
        *func.id(),
        inputs,
    )
    .await?;

    Ok(())
}

async fn create_attribute_function_for_output_socket(
    ctx: &DalContext,
    schema_variant_id: SchemaVariantId,
    external_provider_id: ExternalProviderId,
    func_unique_id: FuncUniqueId,
    inputs: Vec<SiPkgAttrFuncInputView>,
    func_map: &FuncMap,
) -> PkgResult<()> {
    let func = func_map
        .get(&func_unique_id)
        .ok_or(PkgError::MissingFuncUniqueId(func_unique_id.to_string()))?;

    create_attribute_function(
        ctx,
        AttributeReadContext {
            external_provider_id: Some(external_provider_id),
            ..Default::default()
        },
        schema_variant_id,
        *func.id(),
        inputs,
    )
    .await?;

    Ok(())
}

async fn create_attribute_function(
    ctx: &DalContext,
    context: AttributeReadContext,
    schema_variant_id: SchemaVariantId,
    func_id: FuncId,
    inputs: Vec<SiPkgAttrFuncInputView>,
) -> PkgResult<()> {
    let value = AttributeValue::find_for_context(ctx, context)
        .await?
        .ok_or(AttributeValueError::Missing)?;

    let mut prototype = value
        .attribute_prototype(ctx)
        .await?
        .ok_or(AttributeValueError::MissingAttributePrototype)?;

    prototype.set_func_id(ctx, func_id).await?;

    for input in inputs {
        let arg = match &input {
            SiPkgAttrFuncInputView::Prop { name, .. }
            | SiPkgAttrFuncInputView::InputSocket { name, .. }
            | SiPkgAttrFuncInputView::OutputSocket { name, .. } => {
                FuncArgument::find_by_name_for_func(ctx, name, func_id)
                    .await?
                    .ok_or(PkgError::MissingFuncArgument(name.to_owned(), func_id))?
            }
        };

        match input {
            SiPkgAttrFuncInputView::Prop { prop_path, .. } => {
                let prop = Prop::find_prop_by_raw_path(ctx, schema_variant_id, &prop_path).await?;
                let prop_ip = InternalProvider::find_for_prop(ctx, *prop.id())
                    .await?
                    .ok_or(PkgError::MissingInternalProviderForProp(*prop.id()))?;

                AttributePrototypeArgument::new_for_intra_component(
                    ctx,
                    *prototype.id(),
                    *arg.id(),
                    *prop_ip.id(),
                )
                .await?;
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
                    *prototype.id(),
                    *arg.id(),
                    *explicit_ip.id(),
                )
                .await?;
            }
            _ => {
                dbg!("unsupported taking external provider as input for prop");
            }
        }
    }

    Ok(())
}

async fn create_prop_validation(
    spec: SiPkgValidation<'_>,
    prop_id: PropId,
    ctx: &PropVisitContext<'_, '_>,
) -> PkgResult<()> {
    // Consider grabbing this much earlier and sticking it on the PropVisitContext, since we will
    // fetch it for every validation!
    let builtin_validation_func = Func::find_by_attr(ctx.ctx, "name", &"si:validation")
        .await?
        .pop()
        .ok_or(FuncError::NotFoundByName("si:validation".to_string()))?;

    let validation_kind = match spec {
        SiPkgValidation::IntegerIsBetweenTwoIntegers {
            lower_bound,
            upper_bound,
            ..
        } => ValidationKind::Builtin(Validation::IntegerIsBetweenTwoIntegers {
            value: None,
            lower_bound,
            upper_bound,
        }),
        SiPkgValidation::IntegerIsNotEmpty { .. } => {
            ValidationKind::Builtin(Validation::IntegerIsNotEmpty { value: None })
        }
        SiPkgValidation::StringEquals { expected, .. } => {
            ValidationKind::Builtin(Validation::StringEquals {
                value: None,
                expected,
            })
        }
        SiPkgValidation::StringHasPrefix { expected, .. } => {
            ValidationKind::Builtin(Validation::StringHasPrefix {
                value: None,
                expected,
            })
        }
        SiPkgValidation::StringInStringArray {
            expected,
            display_expected,
            ..
        } => ValidationKind::Builtin(Validation::StringInStringArray {
            value: None,
            expected,
            display_expected,
        }),
        SiPkgValidation::StringIsHexColor { .. } => {
            ValidationKind::Builtin(Validation::StringIsHexColor { value: None })
        }
        SiPkgValidation::StringIsNotEmpty { .. } => {
            ValidationKind::Builtin(Validation::StringIsNotEmpty { value: None })
        }
        SiPkgValidation::StringIsValidIpAddr { .. } => {
            ValidationKind::Builtin(Validation::StringIsValidIpAddr { value: None })
        }
        SiPkgValidation::CustomValidation { func_unique_id, .. } => ValidationKind::Custom(
            *ctx.func_map
                .get(&func_unique_id)
                .ok_or(PkgError::MissingFuncUniqueId(func_unique_id.to_string()))?
                .id(),
        ),
    };

    create_validation(
        ctx.ctx,
        validation_kind,
        *builtin_validation_func.id(),
        prop_id,
        ctx.schema_id,
        ctx.schema_variant_id,
    )
    .await?;

    Ok(())
}

async fn create_prop(
    spec: SiPkgProp<'_>,
    parent_prop_id: Option<PropId>,
    ctx: &PropVisitContext<'_, '_>,
) -> PkgResult<Option<PropId>> {
    let mut prop = Prop::new(
        ctx.ctx,
        spec.name(),
        match &spec {
            SiPkgProp::String { .. } => PropKind::String,
            SiPkgProp::Number { .. } => PropKind::Integer,
            SiPkgProp::Boolean { .. } => PropKind::Boolean,
            SiPkgProp::Map { .. } => PropKind::Map,
            SiPkgProp::Array { .. } => PropKind::Array,
            SiPkgProp::Object { .. } => PropKind::Object,
        },
        spec.info()
            .widget_kind
            .as_ref()
            .map(|widget_kind| (widget_kind.into(), spec.info().widget_options.to_owned())),
        ctx.schema_variant_id,
        parent_prop_id,
    )
    .await
    .map_err(SiPkgError::visit_prop)?;

    prop.set_hidden(ctx.ctx, spec.info().hidden).await?;
    prop.set_doc_link(
        ctx.ctx,
        spec.info().doc_link.as_ref().map(|l| l.to_string()),
    )
    .await?;

    let prop_id = *prop.id();

    // Both attribute functions and default values have to be set *after* the schema variant is
    // "finalized", so we can't do until we construct the *entire* prop tree. Hence we push work
    // queues up to the outer context via the PropVisitContext, which uses Mutexes for interior
    // mutability (maybe there's a better type for that here?)

    if let Some(default_value_info) = match &spec {
        SiPkgProp::String { default_value, .. } => {
            default_value.as_ref().map(|dv| DefaultValueInfo::String {
                prop_id,
                default_value: dv.to_owned(),
            })
        }
        SiPkgProp::Number { default_value, .. } => {
            default_value.map(|default_value| DefaultValueInfo::Number {
                prop_id,
                default_value,
            })
        }
        SiPkgProp::Boolean { default_value, .. } => {
            default_value.map(|default_value| DefaultValueInfo::Boolean {
                prop_id,
                default_value,
            })
        }
        // Default values for complex types are not yet supported in packages
        _ => None,
    } {
        ctx.default_values.lock().await.push(default_value_info);
    }

    if let Some(func_unique_id) = spec.func_unique_id() {
        let mut inputs = spec.inputs()?;
        ctx.attr_funcs.lock().await.push(AttrFuncInfo {
            func_unique_id,
            prop_id,
            inputs: inputs.drain(..).map(Into::into).collect(),
        });
    }

    for validation_spec in spec.validations()? {
        create_prop_validation(validation_spec, *prop.id(), ctx).await?;
    }

    Ok(Some(*prop.id()))
}
