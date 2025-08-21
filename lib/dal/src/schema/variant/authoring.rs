use std::{
    collections::HashMap,
    fmt::Debug,
};

use base64::{
    Engine,
    engine::general_purpose,
};
use chrono::Utc;
use convert_case::{
    Case,
    Casing,
};
use pkg::import::import_schema_variant;
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::error::Category;
use si_events::{
    FuncRunId,
    FuncRunState,
    ulid::Ulid,
};
use si_layer_cache::LayerDbError;
use si_pkg::{
    FuncSpec,
    FuncSpecBackendKind,
    FuncSpecBackendResponseType,
    FuncSpecData,
    MergeSkip,
    PkgSpec,
    SchemaVariantSpec,
    SiPkg,
    SiPkgError,
    SpecError,
};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    Component,
    ComponentError,
    ComponentType,
    DalContext,
    Func,
    FuncBackendKind,
    FuncBackendResponseType,
    FuncError,
    FuncId,
    Schema,
    SchemaError,
    SchemaId,
    SchemaVariant,
    SchemaVariantError,
    SchemaVariantId,
    WorkspaceSnapshotError,
    action::prototype::ActionPrototypeError,
    attribute::prototype::{
        AttributePrototypeError,
        argument::AttributePrototypeArgumentError,
    },
    func::{
        authoring::FuncAuthoringError,
        intrinsics::IntrinsicFunc,
        runner::{
            FuncRunner,
            FuncRunnerError,
        },
    },
    pkg::{
        self,
        ImportOptions,
        PkgError,
        export::PkgExporter,
        import::import_only_new_funcs,
        import_pkg_from_pkg,
    },
    prop::PropError,
    schema::variant::{
        SchemaVariantJson,
        SchemaVariantMetadataJson,
    },
    socket::{
        input::InputSocketError,
        output::OutputSocketError,
    },
};

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Error, Debug)]
pub enum VariantAuthoringError {
    #[error("action prototype error: {0}")]
    ActionPrototype(#[from] Box<ActionPrototypeError>),
    #[error(
        "found unexpected return type: expected type 'Asset' to be returned for asset func (FuncId {0}): raw error: {1})"
    )]
    AssetTypeNotReturnedForAssetFunc(FuncId, String),
    #[error("attribute prototype error: {0}")]
    AttributePrototype(#[from] Box<AttributePrototypeError>),
    #[error("attribute prototype error: {0}")]
    AttributePrototypeArgument(#[from] Box<AttributePrototypeArgumentError>),
    #[error("component error: {0}")]
    Component(#[from] Box<ComponentError>),
    #[error("there already exists a Schema with the name {0}")]
    DuplicatedSchemaName(String),
    #[error("empty value within func run value (FuncId {0} and FuncRunId {1})")]
    EmptyValueWithinFuncRunValue(FuncId, FuncRunId),
    #[error("func error: {0}")]
    Func(#[from] Box<FuncError>),
    #[error("func authoring error: {0}")]
    FuncAuthoring(#[from] Box<FuncAuthoringError>),
    #[error("func execution failure error: {0}")]
    FuncExecutionFailure(String),
    #[error("func run error: {0}")]
    FuncRun(#[from] Box<FuncRunnerError>),
    #[error("func run value sender has terminated without sending")]
    FuncRunGone,
    #[error("input socket error: {0}")]
    InputSocket(#[from] Box<InputSocketError>),
    #[error("layer db error: {0}")]
    LayerDb(#[from] LayerDbError),
    #[error("trying to modify locked variant: {0}")]
    LockedVariant(SchemaVariantId),
    #[error("no new asset was created")]
    NoAssetCreated,
    #[error("output socket error: {0}")]
    OutputSocket(#[from] Box<OutputSocketError>),
    #[error("pkg error: {0}")]
    Pkg(#[from] Box<PkgError>),
    #[error("constructed package has no schema node")]
    PkgMissingSchema,
    #[error("constructed package has no schema variant node")]
    PkgMissingSchemaVariant,
    #[error("prop error: {0}")]
    Prop(#[from] Box<PropError>),
    #[error("schema error: {0}")]
    Schema(#[from] Box<SchemaError>),
    #[error("schema {0} already has unlocked variant: {1}")]
    SchemaAlreadyUnlocked(SchemaId, SchemaVariantId),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] Box<SchemaVariantError>),
    #[error("schema variant asset func not found: {0}")]
    SchemaVariantAssetNotFound(SchemaVariantId),
    #[error("schema variant not found: {0}")]
    SchemaVariantNotFound(SchemaVariantId),
    #[error("schema variant not updated: {0}")]
    SchemaVariantUpdatedFailed(SchemaVariantId),
    #[error("json serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("si db error: {0}")]
    SiDb(#[from] si_db::Error),
    #[error("si pkg error: {0}")]
    SiPkg(#[from] SiPkgError),
    #[error("spec error: {0}")]
    Spec(#[from] SpecError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

impl From<ActionPrototypeError> for VariantAuthoringError {
    fn from(value: ActionPrototypeError) -> Self {
        Box::new(value).into()
    }
}

impl From<AttributePrototypeError> for VariantAuthoringError {
    fn from(value: AttributePrototypeError) -> Self {
        Box::new(value).into()
    }
}

impl From<AttributePrototypeArgumentError> for VariantAuthoringError {
    fn from(value: AttributePrototypeArgumentError) -> Self {
        Box::new(value).into()
    }
}

impl From<ComponentError> for VariantAuthoringError {
    fn from(value: ComponentError) -> Self {
        Box::new(value).into()
    }
}

impl From<FuncError> for VariantAuthoringError {
    fn from(value: FuncError) -> Self {
        Box::new(value).into()
    }
}

impl From<FuncAuthoringError> for VariantAuthoringError {
    fn from(value: FuncAuthoringError) -> Self {
        Box::new(value).into()
    }
}

impl From<FuncRunnerError> for VariantAuthoringError {
    fn from(value: FuncRunnerError) -> Self {
        Box::new(value).into()
    }
}

impl From<InputSocketError> for VariantAuthoringError {
    fn from(value: InputSocketError) -> Self {
        Box::new(value).into()
    }
}

impl From<OutputSocketError> for VariantAuthoringError {
    fn from(value: OutputSocketError) -> Self {
        Box::new(value).into()
    }
}

impl From<PkgError> for VariantAuthoringError {
    fn from(value: PkgError) -> Self {
        Box::new(value).into()
    }
}

impl From<PropError> for VariantAuthoringError {
    fn from(value: PropError) -> Self {
        Box::new(value).into()
    }
}

impl From<SchemaError> for VariantAuthoringError {
    fn from(value: SchemaError) -> Self {
        Box::new(value).into()
    }
}

impl From<SchemaVariantError> for VariantAuthoringError {
    fn from(value: SchemaVariantError) -> Self {
        Box::new(value).into()
    }
}

type VariantAuthoringResult<T> = Result<T, VariantAuthoringError>;

const DEFAULT_ASSET_CODE: &str = r#"function main() {
  const asset = new AssetBuilder();
  return asset.build();
}"#;

#[derive(Debug, Deserialize, Serialize)]
struct SchemaVariantJsonWrapper {
    definition: Option<SchemaVariantJson>,
    error: Option<String>,
}

#[derive(Debug)]
pub struct VariantAuthoringClient;

impl VariantAuthoringClient {
    /// Creates a [`SchemaVariant`] and returns the [result](SchemaVariant).
    #[instrument(name = "variant.authoring.create_variant", level = "info", skip_all)]
    #[allow(clippy::too_many_arguments)]
    pub async fn create_schema_and_variant_from_code(
        ctx: &DalContext,
        name: impl Into<String>,
        description: Option<String>,
        link: Option<String>,
        category: impl Into<String>,
        color: impl Into<String>,
        code: impl AsRef<str>,
    ) -> VariantAuthoringResult<SchemaVariant> {
        let name = name.into();
        if Schema::is_name_taken(ctx, &name).await? {
            return Err(VariantAuthoringError::DuplicatedSchemaName(name));
        };

        let variant_version = SchemaVariant::generate_version_string();

        let code_base64 = general_purpose::STANDARD_NO_PAD.encode(code.as_ref());
        let asset_func = Func::new(
            ctx,
            generate_scaffold_func_name(&name),
            Some(name.clone()),
            description.clone(),
            link.clone(),
            false,
            false,
            FuncBackendKind::JsSchemaVariantDefinition,
            FuncBackendResponseType::SchemaVariantDefinition,
            Some("main"),
            Some(code_base64),
            false,
        )
        .await?;

        let asset_func_spec = build_asset_func_spec(&asset_func)?;
        let definition = Self::execute_asset_func(ctx, &asset_func).await?;

        let metadata = SchemaVariantMetadataJson {
            schema_name: name.clone(),
            version: variant_version.clone(),
            display_name: name.clone(),
            category: category.into(),
            color: color.into(),
            component_type: ComponentType::Component,
            link: link.clone(),
            description: description.clone(),
        };
        let email = ctx.history_actor().email(ctx).await?;

        let pkg_spec =
            build_pkg_spec_for_variant(&name, definition, &asset_func_spec, &metadata, &email)?;

        let pkg = SiPkg::load_from_spec(pkg_spec.clone())?;

        let (_, schema_variant_ids, _) = import_pkg_from_pkg(
            ctx,
            &pkg,
            Some(ImportOptions {
                skip_import_funcs: Some(HashMap::from_iter([(
                    asset_func_spec.unique_id.to_owned(),
                    asset_func.clone(),
                )])),
                create_unlocked: true,
                schema_id: Some(Ulid::new()),
                ..Default::default()
            }),
        )
        .await?;

        let schema_variant_id = schema_variant_ids
            .first()
            .copied()
            .ok_or(VariantAuthoringError::NoAssetCreated)?;
        ctx.workspace_snapshot()?
            .clear_prop_suggestions_cache()
            .await;

        Ok(SchemaVariant::get_by_id(ctx, schema_variant_id).await?)
    }

    pub async fn create_schema_and_variant(
        ctx: &DalContext,
        name: impl Into<String>,
        description: Option<String>,
        link: Option<String>,
        category: impl Into<String>,
        color: impl Into<String>,
    ) -> VariantAuthoringResult<SchemaVariant> {
        Self::create_schema_and_variant_from_code(
            ctx,
            name,
            description,
            link,
            category,
            color,
            DEFAULT_ASSET_CODE,
        )
        .await
    }

    #[instrument(
        name = "variant.authoring.new_schema_with_cloned_variant",
        level = "info",
        skip_all
    )]
    pub async fn new_schema_with_cloned_variant(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
        schema_name: String,
    ) -> VariantAuthoringResult<(SchemaVariant, Schema)> {
        if Schema::is_name_taken(ctx, &schema_name).await? {
            return Err(VariantAuthoringError::DuplicatedSchemaName(schema_name));
        };

        let variant = SchemaVariant::get_by_id(ctx, schema_variant_id).await?;
        let schema = variant.schema(ctx).await?;

        if let Some(asset_func_id) = variant.asset_func_id() {
            let old_func = Func::get_by_id(ctx, asset_func_id).await?;

            let cloned_func = old_func
                .clone_func_with_new_name(ctx, schema_name.clone())
                .await?;
            let cloned_func_spec = build_asset_func_spec(&cloned_func)?;
            let definition = Self::execute_asset_func(ctx, &cloned_func).await?;
            let display_name = format!("{}-Clone", variant.display_name());

            let metadata = SchemaVariantMetadataJson {
                schema_name: schema_name.clone(),
                version: SchemaVariant::generate_version_string(),
                display_name,
                category: variant.category().to_string(),
                color: variant.get_color(ctx).await?,
                component_type: variant.component_type(),
                link: variant.link().clone(),
                description: variant.description().clone(),
            };
            let email = ctx.history_actor().email(ctx).await?;
            let pkg_spec = build_pkg_spec_for_variant(
                &schema.name,
                definition,
                &cloned_func_spec,
                &metadata,
                &email,
            )?;

            let pkg = SiPkg::load_from_spec(pkg_spec.clone())?;

            let (_, schema_variant_ids, _) = import_pkg_from_pkg(
                ctx,
                &pkg,
                Some(ImportOptions {
                    skip_import_funcs: Some(HashMap::from_iter([(
                        cloned_func_spec.unique_id.to_owned(),
                        cloned_func.clone(),
                    )])),
                    create_unlocked: true,
                    schema_id: Some(Ulid::new()),
                    ..Default::default()
                }),
            )
            .await?;

            let new_schema_variant_id = schema_variant_ids
                .first()
                .copied()
                .ok_or(VariantAuthoringError::NoAssetCreated)?;
            ctx.workspace_snapshot()?
                .clear_prop_suggestions_cache()
                .await;

            Ok((
                SchemaVariant::get_by_id(ctx, new_schema_variant_id).await?,
                schema,
            ))
        } else {
            return Err(VariantAuthoringError::SchemaVariantAssetNotFound(
                schema_variant_id,
            ));
        }
    }

    #[instrument(
        name = "variant.authoring.regenerate_variant",
        level = "info",
        skip(ctx)
    )]
    pub async fn regenerate_variant(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> VariantAuthoringResult<SchemaVariantId> {
        let schema_variant = SchemaVariant::get_by_id(ctx, schema_variant_id).await?;

        if schema_variant.is_locked {
            return Err(VariantAuthoringError::LockedVariant(schema_variant_id));
        };

        let schema = schema_variant.schema(ctx).await?;

        let components_in_use = SchemaVariant::list_component_ids(ctx, schema_variant_id).await?;

        if components_in_use.is_empty() {
            Self::update_existing_variant_and_regenerate(
                ctx,
                schema_variant_id,
                schema.name,
                schema_variant.display_name,
                schema_variant.category,
                schema_variant.color,
                schema_variant.link,
                schema_variant.description,
                schema_variant.component_type,
            )
            .await?;
            Ok(schema_variant_id)
        } else {
            let original_is_default = schema_variant.is_default(ctx).await?;

            let new_variant = Self::generate_variant_with_updates(
                ctx,
                schema_variant_id,
                &schema.name,
                schema_variant.display_name,
                schema_variant.category,
                schema_variant.color,
                schema_variant.link,
                schema_variant.description,
                schema_variant.component_type,
            )
            .await?;

            for component_id in components_in_use {
                Component::upgrade_to_new_variant(ctx, component_id, new_variant.id).await?;
            }

            if original_is_default {
                schema.set_default_variant_id(ctx, new_variant.id).await?;
            }

            // When we get here, "schema_variant_id" points to an unlocked variant with no components
            // and no way to create new components from it. No other changesets point to it,
            // since it's been created on the changeset (otherwise it would not be unlocked)
            // So we should clean it up
            SchemaVariant::cleanup_unlocked_variant(ctx, schema_variant_id).await?;
            ctx.workspace_snapshot()?
                .clear_prop_suggestions_cache()
                .await;

            Ok(new_variant.id)
        }
    }

    #[allow(clippy::too_many_arguments)]
    #[instrument(
        name = "variant.authoring.update_existing_variant_and_regenerate",
        level = "info",
        skip_all
    )]
    async fn update_existing_variant_and_regenerate(
        ctx: &DalContext,
        current_schema_variant_id: SchemaVariantId,
        schema_name: impl Into<String>,
        display_name: impl Into<String>,
        category: impl Into<String>,
        color: impl Into<String>,
        link: Option<String>,
        description: Option<String>,
        component_type: ComponentType,
    ) -> VariantAuthoringResult<()> {
        // Ok we need to delete the first level of outgoing children for the schema variant
        let current_schema_variant =
            SchemaVariant::get_by_id(ctx, current_schema_variant_id).await?;

        // then we can build the package and reimport ALL but the schema variant itself
        let asset_func = current_schema_variant.get_asset_func(ctx).await?;

        let display_name = display_name.into();
        let category = category.into();
        let color = color.into();
        let schema_name = schema_name.into();

        let asset_func_spec = build_asset_func_spec(&asset_func)?;
        let definition = Self::execute_asset_func(ctx, &asset_func).await?;
        let metadata = SchemaVariantMetadataJson {
            schema_name: schema_name.clone(),
            version: SchemaVariant::generate_version_string(),
            display_name: display_name.clone(),
            category: category.clone(),
            color: color.clone(),
            component_type,
            link: link.clone(),
            description: description.clone(),
        };

        let (new_variant_spec, _skips, variant_funcs) =
            build_variant_spec_based_on_existing_variant(
                ctx,
                definition,
                &asset_func_spec,
                &metadata,
                current_schema_variant_id,
            )
            .await?;

        let schema_spec = metadata.to_schema_spec(new_variant_spec)?;
        let email = ctx.history_actor().email(ctx).await?;
        let pkg_spec = PkgSpec::builder()
            .name(schema_name.clone())
            .created_by(&email)
            .funcs(variant_funcs.clone())
            .func(asset_func_spec)
            .schema(schema_spec)
            .version("0")
            .build()?;
        let pkg = SiPkg::load_from_spec(pkg_spec)?;

        let pkg_schemas = pkg.schemas()?;
        let pkg_variants = pkg_schemas
            .first()
            .ok_or(VariantAuthoringError::PkgMissingSchema)?
            .variants()?;

        let schema_spec = pkg_schemas
            .first()
            .ok_or(VariantAuthoringError::PkgMissingSchema)?;
        let variant_pkg_spec = pkg_variants
            .first()
            .ok_or(VariantAuthoringError::PkgMissingSchemaVariant)?;

        let schema = SchemaVariant::get_by_id(ctx, current_schema_variant_id)
            .await?
            .schema(ctx)
            .await?;

        schema
            .clone()
            .modify(ctx, |s| {
                s.name.clone_from(&schema_name.clone());
                Ok(())
            })
            .await?;

        // We need to clean up the old graph before we re-import the new parts!
        current_schema_variant
            .remove_external_connections(ctx)
            .await?;
        current_schema_variant
            .rebuild_variant_root_prop(ctx)
            .await?;

        // Now we can reimport all parts of the schema variant in place!
        let mut thing_map = import_only_new_funcs(ctx, pkg.funcs()?).await?;
        let new_schema_variant = import_schema_variant(
            ctx,
            &schema,
            schema_spec.clone(),
            variant_pkg_spec,
            None,
            &mut thing_map,
            Some(current_schema_variant),
        )
        .await?;

        if new_schema_variant.id != current_schema_variant_id {
            return Err(VariantAuthoringError::SchemaVariantUpdatedFailed(
                current_schema_variant_id,
            ));
        }

        // Let's update the SV struct now to reflect any changes
        new_schema_variant
            .clone()
            .modify(ctx, |sv| {
                sv.description = description;
                sv.link = link;
                sv.category.clone_from(&category);
                sv.component_type = component_type;
                sv.color.clone_from(&color);
                sv.display_name = display_name;
                Ok(())
            })
            .await?;

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    #[instrument(
        name = "variant.authoring.update_and_generate_variant_with_new_version",
        level = "info",
        skip_all
    )]
    async fn generate_variant_with_updates(
        ctx: &DalContext,
        current_sv_id: SchemaVariantId,
        schema_name: impl Into<String>,
        display_name: String,
        category: impl Into<String>,
        color: impl Into<String>,
        link: Option<String>,
        description: Option<String>,
        component_type: ComponentType,
    ) -> VariantAuthoringResult<SchemaVariant> {
        let schema_name = schema_name.into();

        let old_sv = SchemaVariant::get_by_id(ctx, current_sv_id).await?;
        let old_asset_func = old_sv.get_asset_func(ctx).await?;

        let new_asset_func = old_asset_func
            .clone_func_with_new_name(ctx, generate_scaffold_func_name(&schema_name))
            .await?;

        let asset_func_spec = build_asset_func_spec(&new_asset_func.clone())?;
        let definition = Self::execute_asset_func(ctx, &new_asset_func).await?;

        let metadata = SchemaVariantMetadataJson {
            schema_name: schema_name.clone(),
            version: SchemaVariant::generate_version_string(),
            display_name: display_name.clone(),
            category: category.into(),
            color: color.into(),
            component_type,
            link: link.clone(),
            description: description.clone(),
        };

        let (new_variant_spec, _skips, variant_funcs) =
            build_variant_spec_based_on_existing_variant(
                ctx,
                definition,
                &asset_func_spec,
                &metadata,
                current_sv_id,
            )
            .await?;

        let schema_spec = metadata.to_schema_spec(new_variant_spec)?;

        let email = ctx.history_actor().email(ctx).await?;

        let pkg_spec = PkgSpec::builder()
            .name(schema_name.clone())
            .created_by(&email)
            .funcs(variant_funcs.clone())
            .func(asset_func_spec)
            .schema(schema_spec)
            .version("0")
            .build()?;
        let pkg = SiPkg::load_from_spec(pkg_spec)?;

        let pkg_schemas = pkg.schemas()?;
        let pkg_variants = pkg_schemas
            .first()
            .ok_or(VariantAuthoringError::PkgMissingSchema)?
            .variants()?;

        let schema_spec = pkg_schemas
            .first()
            .ok_or(VariantAuthoringError::PkgMissingSchema)?;
        let variant_pkg_spec = pkg_variants
            .first()
            .ok_or(VariantAuthoringError::PkgMissingSchemaVariant)?;

        let schema = SchemaVariant::get_by_id(ctx, current_sv_id)
            .await?
            .schema(ctx)
            .await?;

        schema
            .clone()
            .modify(ctx, |s| {
                s.name = schema_name;
                Ok(())
            })
            .await?;

        let mut thing_map = import_only_new_funcs(ctx, pkg.funcs()?).await?;

        Ok(import_schema_variant(
            ctx,
            &schema,
            schema_spec.clone(),
            variant_pkg_spec,
            None,
            &mut thing_map,
            None,
        )
        .await?)
    }

    // Note(victor): This is very similar to the logic in update_and_generate_variant_with_new_version, with a few differences:
    // 1. it makes an exact copy of the schema variant - in the future there'll be no updates on unlocked copies
    // 2. it does not update the default schema variant
    #[instrument(
        name = "variant.authoring.create_unlocked_variant_copy",
        level = "info",
        skip_all
    )]
    pub async fn create_unlocked_variant_copy(
        ctx: &DalContext,
        locked_variant_id: SchemaVariantId,
    ) -> VariantAuthoringResult<SchemaVariant> {
        let locked_variant = SchemaVariant::get_by_id(ctx, locked_variant_id).await?;
        let schema = locked_variant.schema(ctx).await?;

        if let Some(variant) = SchemaVariant::get_unlocked_for_schema(ctx, schema.id).await? {
            return Err(VariantAuthoringError::SchemaAlreadyUnlocked(
                schema.id, variant.id,
            ));
        }

        // Create copy of asset func
        let locked_variant_asset_func_id = locked_variant.get_asset_func(ctx).await?;

        let unlocked_variant_asset_func = locked_variant_asset_func_id
            .clone_func_with_new_name(ctx, generate_scaffold_func_name(&schema.name))
            .await?;

        let (existing_variant_spec, variant_funcs) = PkgExporter::export_variant_standalone(
            ctx,
            &locked_variant,
            schema.name(),
            Some(unlocked_variant_asset_func.id),
        )
        .await?;

        let unlocked_asset_spec = build_asset_func_spec(&unlocked_variant_asset_func)?;
        let metadata = SchemaVariantMetadataJson {
            schema_name: schema.name.clone(),
            version: SchemaVariant::generate_version_string(),
            display_name: locked_variant.display_name().to_string(),
            category: locked_variant.category().to_string(),
            color: locked_variant.color().to_string(),
            component_type: locked_variant.component_type(),
            link: locked_variant.link(),
            description: locked_variant.description(),
        };

        let schema_spec = metadata.to_schema_spec(existing_variant_spec)?;
        let creator_email = ctx.history_actor().email(ctx).await?;
        let pkg_spec = PkgSpec::builder()
            .name(schema.name())
            .created_by(creator_email)
            .funcs(variant_funcs.clone())
            .func(unlocked_asset_spec)
            .schema(schema_spec)
            .version("0")
            .build()?;
        let pkg: SiPkg = SiPkg::load_from_spec(pkg_spec)?;

        let pkg_schemas = pkg.schemas()?;
        let schema_spec = pkg_schemas
            .first()
            .ok_or(VariantAuthoringError::PkgMissingSchema)?;

        let pkg_variants = schema_spec.variants()?;
        let variant_pkg_spec = pkg_variants
            .first()
            .ok_or(VariantAuthoringError::PkgMissingSchemaVariant)?;

        let mut thing_map = import_only_new_funcs(ctx, pkg.funcs()?).await?;
        let new_schema_variant = import_schema_variant(
            ctx,
            &schema,
            schema_spec.clone(),
            variant_pkg_spec,
            None,
            &mut thing_map,
            None,
        )
        .await?;

        // need to manually modify the variant to get the new version
        // as the pkg spec used to generate this new version only has the
        // old one.
        Ok(new_schema_variant
            .modify(ctx, |sv| {
                sv.version = metadata.version;
                Ok(())
            })
            .await?)
    }

    #[allow(clippy::too_many_arguments)]
    #[instrument(
        name = "variant.authoring.save_variant_content",
        level = "info",
        skip_all
    )]
    pub async fn save_variant_content(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
        schema_name: impl Into<String>,
        display_name: impl Into<String>,
        category: impl Into<String>,
        description: Option<String>,
        link: Option<String>,
        color: impl Into<String>,
        component_type: ComponentType,
        code: Option<impl Into<String>>,
    ) -> VariantAuthoringResult<()> {
        let schema_variant = SchemaVariant::get_by_id(ctx, schema_variant_id).await?;

        if schema_variant.is_locked {
            return Err(VariantAuthoringError::LockedVariant(schema_variant_id));
        };

        let schema = schema_variant.schema(ctx).await?;

        let asset_func_id = schema_variant.asset_func_id.ok_or(
            VariantAuthoringError::SchemaVariantAssetNotFound(schema_variant_id),
        )?;

        let schema_name = &schema_name.into();
        schema
            .modify(ctx, |s| {
                s.name.clone_from(schema_name);
                Ok(())
            })
            .await?;

        let variant_description = description.clone();
        let variant_link = link.clone();
        let display_name = &display_name.into();
        let color = &color.into();

        // cache default values to compare and update
        let original_color = schema_variant.color.clone();
        let original_type = schema_variant.component_type;

        let schema_variant = schema_variant
            .modify(ctx, |sv| {
                sv.description = variant_description;
                sv.link = variant_link;
                sv.category.clone_from(&category.into());
                sv.component_type = component_type;
                sv.color.clone_from(color);
                sv.display_name.clone_from(display_name);
                Ok(())
            })
            .await?;

        // now need to update the default values for the schema variant so newly created components get the latest values
        if original_color != *color {
            schema_variant.set_color(ctx, color).await?;
        }
        if original_type != component_type {
            schema_variant
                .set_type(ctx, component_type.to_string())
                .await?;
        }

        let code_base64 = code.map(|c| general_purpose::STANDARD_NO_PAD.encode(c.into()));
        let current_func = Func::get_by_id(ctx, asset_func_id).await?;
        current_func
            .modify(ctx, |func| {
                func.name = generate_scaffold_func_name(schema_name);
                func.backend_kind = FuncBackendKind::JsSchemaVariantDefinition;
                func.backend_response_type = FuncBackendResponseType::SchemaVariantDefinition;
                func.display_name = Some(display_name.clone());
                if let Some(code_base64) = code_base64 {
                    func.code_base64 = Some(code_base64);
                }
                func.description.clone_from(&description);
                func.handler = Some("main".to_string());
                func.hidden = false;
                func.link.clone_from(&link);
                Ok(())
            })
            .await?;
        Ok(())
    }

    pub async fn execute_asset_func(
        ctx: &DalContext,
        asset_func: &Func,
    ) -> VariantAuthoringResult<SchemaVariantJson> {
        let result_channel = FuncRunner::run_asset_definition_func(ctx, asset_func).await?;
        let func_run_value = result_channel
            .await
            .map_err(|_| VariantAuthoringError::FuncRunGone)??;

        let value = func_run_value
            .value()
            .ok_or(VariantAuthoringError::EmptyValueWithinFuncRunValue(
                asset_func.id,
                func_run_value.func_run_id(),
            ))?
            .to_owned();

        let wrapper: SchemaVariantJsonWrapper = match serde_json::from_value(value) {
            Ok(wrapper) => wrapper,
            Err(err)
                if err.classify() == Category::Data
                    && err.to_string().contains("unknown field") =>
            {
                return Err(VariantAuthoringError::AssetTypeNotReturnedForAssetFunc(
                    asset_func.id,
                    err.to_string(),
                ));
            }
            Err(err) => return Err(err.into()),
        };

        if let Some(error) = &wrapper.error {
            return Err(VariantAuthoringError::FuncExecutionFailure(
                error.to_owned(),
            ));
        }

        let Some(definition) = wrapper.definition else {
            return Err(VariantAuthoringError::FuncExecutionFailure(
                "definition cannot be undefined or null".to_string(),
            ));
        };

        FuncRunner::update_run(ctx, func_run_value.func_run_id(), |func_run| {
            func_run.set_state(FuncRunState::Success)
        })
        .await?;

        Ok(definition)
    }
}

async fn build_variant_spec_based_on_existing_variant(
    ctx: &DalContext,
    definition: SchemaVariantJson,
    asset_func_spec: &FuncSpec,
    metadata: &SchemaVariantMetadataJson,
    existing_variant_id: SchemaVariantId,
) -> VariantAuthoringResult<(SchemaVariantSpec, Vec<MergeSkip>, Vec<FuncSpec>)> {
    let identity_func_spec = IntrinsicFunc::Identity.to_spec()?;

    let existing_variant = SchemaVariant::get_by_id(ctx, existing_variant_id).await?;
    let schema = existing_variant.schema(ctx).await?;
    let variant_spec = definition.to_spec(
        metadata.clone(),
        &identity_func_spec.unique_id,
        &asset_func_spec.unique_id,
    )?;

    let (existing_variant_spec, variant_funcs) =
        PkgExporter::export_variant_standalone(ctx, &existing_variant, schema.name(), None).await?;

    let (merged_variant, skips) = variant_spec.merge_prototypes_from(&existing_variant_spec);

    Ok((merged_variant, skips, variant_funcs))
}

#[allow(clippy::result_large_err)]
fn build_asset_func_spec(asset_func: &Func) -> VariantAuthoringResult<FuncSpec> {
    let mut schema_variant_func_spec = FuncSpec::builder();
    schema_variant_func_spec.name(asset_func.name.clone());
    schema_variant_func_spec.unique_id(asset_func.id.to_string());
    let mut func_spec_data_builder = FuncSpecData::builder();
    func_spec_data_builder
        .name(asset_func.name.clone())
        .backend_kind(FuncSpecBackendKind::JsSchemaVariantDefinition)
        .response_type(FuncSpecBackendResponseType::SchemaVariantDefinition)
        .hidden(asset_func.hidden);
    if let Some(code) = asset_func.code_plaintext()? {
        func_spec_data_builder.code_plaintext(code);
    }
    if let Some(handler) = asset_func.handler.clone() {
        func_spec_data_builder.handler(handler.to_string());
    }
    if let Some(description) = asset_func.description.clone() {
        func_spec_data_builder.description(description.to_string());
    }
    if let Some(display_name) = asset_func.display_name.clone() {
        func_spec_data_builder.display_name(display_name.to_string());
    }
    Ok(schema_variant_func_spec
        .data(func_spec_data_builder.build()?)
        .build()?)
}

#[allow(clippy::result_large_err)]
fn build_pkg_spec_for_variant(
    schema_name: &str,
    definition: SchemaVariantJson,
    asset_func_spec: &FuncSpec,
    metadata: &SchemaVariantMetadataJson,
    user_email: &str,
) -> VariantAuthoringResult<PkgSpec> {
    // we need to change this to use the PkgImport
    let identity_func_spec = IntrinsicFunc::Identity.to_spec()?;
    let variant_spec = definition.to_spec(
        metadata.clone(),
        &identity_func_spec.unique_id,
        &asset_func_spec.unique_id,
    )?;
    let schema_spec = metadata.to_schema_spec(variant_spec)?;
    Ok(PkgSpec::builder()
        .name(schema_name)
        .created_by(user_email)
        .func(identity_func_spec)
        .func(asset_func_spec.clone())
        .schema(schema_spec)
        .version("0.0.1")
        .build()?)
}

fn generate_scaffold_func_name(name: impl AsRef<str>) -> String {
    let version = Utc::now().format("%Y%m%d%H%M%S%f").to_string();
    let generated_name = format!("{}Scaffold_{}", name.as_ref().to_case(Case::Camel), version);
    generated_name
}
