use std::path::PathBuf;

use si_pkg::{
    FuncUniqueId, SiPkg, SiPkgError, SiPkgFunc, SiPkgLeafFunction, SiPkgProp, SiPkgSchema,
    SiPkgSchemaVariant, SiPkgValidation, SiPkgWorkflow,
};

use crate::{
    component::ComponentKind,
    installed_pkg::{
        InstalledPkg, InstalledPkgAsset, InstalledPkgAssetKind, InstalledPkgAssetTyped,
        InstalledPkgId,
    },
    schema::{variant::leaves::LeafInputLocation, SchemaUiMenu},
    validation::{create_validation, Validation, ValidationKind},
    ActionPrototype, ActionPrototypeContext, DalContext, Func, FuncArgument, FuncError, Prop,
    PropId, PropKind, Schema, SchemaId, SchemaVariant, SchemaVariantId, StandardModel,
    WorkflowPrototype, WorkflowPrototypeContext,
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
        let func = create_func(ctx, func_spec, *installed_pkg.id()).await?;
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
            // How to handle name conflicts?
            let mut func = Func::new(
                ctx,
                func_spec.name(),
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
            let schema = Schema::new(ctx, schema_spec.name(), &ComponentKind::Standard).await?;
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

struct PropVisitContext<'a, 'b> {
    pub ctx: &'a DalContext,
    pub schema_id: SchemaId,
    pub schema_variant_id: SchemaVariantId,
    pub func_map: &'b FuncMap,
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

            let context = PropVisitContext {
                ctx,
                schema_id: *schema.id(),
                schema_variant_id: *schema_variant.id(),
                func_map,
            };

            schema
                .set_default_schema_variant_id(ctx, Some(schema_variant.id()))
                .await?;

            if let Some(color) = variant_spec.color() {
                schema_variant.set_color(ctx, color.to_owned()).await?;
            }

            let domain_prop_id = root_prop.domain_prop_id;
            variant_spec
                .visit_prop_tree(create_prop, Some(domain_prop_id), &context)
                .await?;

            schema_variant.finalize(ctx, None).await?;

            for leaf_func in variant_spec.leaf_functions()? {
                create_leaf_function(ctx, leaf_func, *schema_variant.id(), func_map).await?;
            }

            for workflow in variant_spec.workflows()? {
                create_workflow(ctx, workflow, *schema.id(), *schema_variant.id(), func_map)
                    .await?;
            }

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
        .ok_or(FuncError::NotFoundByName("si_validation".to_string()))?;

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
    let prop = Prop::new(
        ctx.ctx,
        spec.name(),
        match spec {
            SiPkgProp::String { .. } => PropKind::String,
            SiPkgProp::Number { .. } => PropKind::Integer,
            SiPkgProp::Boolean { .. } => PropKind::Boolean,
            SiPkgProp::Map { .. } => PropKind::Map,
            SiPkgProp::Array { .. } => PropKind::Array,
            SiPkgProp::Object { .. } => PropKind::Object,
        },
        None,
        ctx.schema_variant_id,
        parent_prop_id,
    )
    .await
    .map_err(SiPkgError::visit_prop)?;

    for validation_spec in spec.validations()? {
        create_prop_validation(validation_spec, *prop.id(), ctx).await?;
    }

    Ok(Some(*prop.id()))
}
