use serde::Serialize;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use telemetry::prelude::*;

use crate::action_prototype::ActionKind;
use crate::func::argument::{FuncArgument, FuncArgumentId};
use crate::property_editor::schema::WidgetKind;
use crate::schema::variant::definition::{
    PropCache, SchemaVariantDefinitionJson, SchemaVariantDefinitionMetadataJson,
};
use crate::schema::{RootProp, SchemaUiMenu};
use crate::{
    func::{
        binding::{FuncBinding, FuncBindingId},
        binding_return_value::FuncBindingReturnValueId,
    },
    validation::{create_validation, ValidationKind},
    ActionPrototype, ActionPrototypeContext, AttributePrototypeArgument, AttributeReadContext,
    AttributeValue, BuiltinsError, BuiltinsResult, DalContext, ExternalProvider, Func,
    FuncDescription, FuncDescriptionContents, FuncError, FuncId, InternalProvider,
    InternalProviderId, LeafInput, LeafInputLocation, LeafKind, Prop, PropId, PropKind, Schema,
    SchemaError, SchemaId, SchemaVariant, SchemaVariantError, SchemaVariantId,
    SelectedTestBuiltinSchemas, StandardModel, WorkflowPrototype, WorkflowPrototypeContext,
};

mod aws_ami;
mod aws_ec2_instance;
mod aws_egress;
mod aws_eip;
mod aws_ingress;
mod aws_keypair;
mod aws_region;
mod aws_security_group;
mod coreos_butane;
mod docker_hub_credential;
mod docker_image;
mod kubernetes_deployment;
mod kubernetes_namespace;
mod systeminit_generic_frame;
mod test_exclusive_fallout;
mod test_exclusive_starfield;

const NODE_COLOR_FRAMES: &str = "#FFFFFF";
// Reference: https://aws.amazon.com/trademark-guidelines/
const NODE_COLOR_AWS: &str = "#FF9900";
// Reference: https://getfedora.org/
const NODE_COLOR_COREOS: &str = "#E26B70";
// Reference: https://www.docker.com/company/newsroom/media-resources/
const NODE_COLOR_DOCKER: &str = "#4695E7";
// This node color is purely meant the complement existing node colors.
// It does not reflect an official branding Kubernetes color.
const NODE_COLOR_KUBERNETES: &str = "#30BA78";

/// Migrate [`Schemas`](crate::Schema) for production use.
pub async fn migrate_for_production(ctx: &DalContext) -> BuiltinsResult<()> {
    info!("migrating schemas");
    let driver = MigrationDriver::new(ctx).await?;
    ctx.blocking_commit().await?;

    driver
        .migrate_docker_image(ctx, "Docker", NODE_COLOR_DOCKER)
        .await?;
    ctx.blocking_commit().await?;

    driver
        .migrate_docker_hub_credential(ctx, "Docker", NODE_COLOR_DOCKER)
        .await?;
    ctx.blocking_commit().await?;

    driver
        .migrate_kubernetes_deployment(ctx, "Kubernetes", NODE_COLOR_KUBERNETES)
        .await?;
    ctx.blocking_commit().await?;

    driver
        .migrate_kubernetes_namespace(ctx, "Kubernetes", NODE_COLOR_KUBERNETES)
        .await?;
    ctx.blocking_commit().await?;

    driver
        .migrate_coreos_butane(ctx, "CoreOS", NODE_COLOR_COREOS)
        .await?;
    ctx.blocking_commit().await?;

    driver.migrate_aws_ami(ctx, "AWS", NODE_COLOR_AWS).await?;
    ctx.blocking_commit().await?;

    driver
        .migrate_aws_ec2_instance(ctx, "AWS", NODE_COLOR_AWS)
        .await?;
    ctx.blocking_commit().await?;

    driver
        .migrate_aws_region(ctx, "AWS", NODE_COLOR_AWS)
        .await?;
    ctx.blocking_commit().await?;

    driver.migrate_aws_eip(ctx, "AWS", NODE_COLOR_AWS).await?;
    ctx.blocking_commit().await?;

    driver
        .migrate_aws_keypair(ctx, "AWS", NODE_COLOR_AWS)
        .await?;
    ctx.blocking_commit().await?;

    driver
        .migrate_aws_ingress(ctx, "AWS", NODE_COLOR_AWS)
        .await?;
    ctx.blocking_commit().await?;

    driver
        .migrate_aws_egress(ctx, "AWS", NODE_COLOR_AWS)
        .await?;
    ctx.blocking_commit().await?;

    driver
        .migrate_aws_security_group(ctx, "AWS", NODE_COLOR_AWS)
        .await?;
    ctx.blocking_commit().await?;

    driver
        .migrate_systeminit_generic_frame(ctx, "Frames", NODE_COLOR_FRAMES)
        .await?;
    ctx.blocking_commit().await?;

    Ok(())
}

/// Migrate [`Schemas`](crate::Schema) for use in tests.
pub async fn migrate_for_tests(
    ctx: &DalContext,
    selected_test_builtin_schemas: SelectedTestBuiltinSchemas,
) -> BuiltinsResult<()> {
    // Determine what to migrate based on the selected test builtin schemas provided.
    let (migrate_all, migrate_test_exclusive, specific_builtin_schemas) =
        match selected_test_builtin_schemas {
            SelectedTestBuiltinSchemas::All => {
                info!("migrating schemas for tests");
                (true, false, HashSet::new())
            }
            SelectedTestBuiltinSchemas::None => {
                info!("skipping migrating schemas for tests");
                return Ok(());
            }
            SelectedTestBuiltinSchemas::Some(provided_set) => {
                info!("migrating schemas for tests based on a provided set of names");
                debug!("provided set of builtin schemas: {:?}", &provided_set);
                (false, false, provided_set)
            }
            SelectedTestBuiltinSchemas::Test => {
                info!("migrating test-exclusive schemas solely");
                (false, true, HashSet::new())
            }
        };

    // Once we know what to migrate, create the driver.
    let driver = MigrationDriver::new(ctx).await?;
    ctx.blocking_commit().await?;

    // Perform migrations on production schemas.
    if migrate_all || specific_builtin_schemas.contains("docker image") {
        driver
            .migrate_docker_image(ctx, "Docker", NODE_COLOR_DOCKER)
            .await?;
        ctx.blocking_commit().await?;
    }
    if migrate_all || specific_builtin_schemas.contains("coreos butane") {
        driver
            .migrate_coreos_butane(ctx, "CoreOS", NODE_COLOR_COREOS)
            .await?;
        ctx.blocking_commit().await?;
    }
    if migrate_all || specific_builtin_schemas.contains("aws ami") {
        driver.migrate_aws_ami(ctx, "AWS", NODE_COLOR_AWS).await?;
        ctx.blocking_commit().await?;
    }
    if migrate_all
        || specific_builtin_schemas.contains("aws ec2")
        || specific_builtin_schemas.contains("aws ec2 instance")
    {
        driver
            .migrate_aws_ec2_instance(ctx, "AWS", NODE_COLOR_AWS)
            .await?;
        ctx.blocking_commit().await?;
    }
    if migrate_all || specific_builtin_schemas.contains("aws region") {
        driver
            .migrate_aws_region(ctx, "AWS", NODE_COLOR_AWS)
            .await?;
        ctx.blocking_commit().await?;
    }
    if migrate_all || specific_builtin_schemas.contains("aws eip") {
        driver.migrate_aws_eip(ctx, "AWS", NODE_COLOR_AWS).await?;
        ctx.blocking_commit().await?;
    }
    if migrate_all
        || specific_builtin_schemas.contains("aws key pair")
        || specific_builtin_schemas.contains("aws keypair")
    {
        driver
            .migrate_aws_keypair(ctx, "AWS", NODE_COLOR_AWS)
            .await?;
        ctx.blocking_commit().await?;
    }
    if migrate_all || specific_builtin_schemas.contains("aws ingress") {
        driver
            .migrate_aws_ingress(ctx, "AWS", NODE_COLOR_AWS)
            .await?;
        ctx.blocking_commit().await?;
    }
    if migrate_all || specific_builtin_schemas.contains("aws egress") {
        driver
            .migrate_aws_egress(ctx, "AWS", NODE_COLOR_AWS)
            .await?;
        ctx.blocking_commit().await?;
    }
    if migrate_all
        || specific_builtin_schemas.contains("aws security group")
        || specific_builtin_schemas.contains("aws securitygroup")
    {
        driver
            .migrate_aws_security_group(ctx, "AWS", NODE_COLOR_AWS)
            .await?;
        ctx.blocking_commit().await?;
    }
    if migrate_all
        || specific_builtin_schemas.contains("systeminit generic frame")
        || specific_builtin_schemas.contains("si generic frame")
        || specific_builtin_schemas.contains("generic frame")
    {
        driver
            .migrate_systeminit_generic_frame(ctx, "Frames", NODE_COLOR_FRAMES)
            .await?;
        ctx.blocking_commit().await?;
    }

    // Perform migrations on "hidden" schemas.
    if migrate_all || specific_builtin_schemas.contains("docker hub credential") {
        driver
            .migrate_docker_hub_credential(ctx, "Docker", NODE_COLOR_DOCKER)
            .await?;
        ctx.blocking_commit().await?;
    }
    if migrate_all || specific_builtin_schemas.contains("kubernetes deployment") {
        driver
            .migrate_kubernetes_deployment(ctx, "Kubernetes", NODE_COLOR_KUBERNETES)
            .await?;
        ctx.blocking_commit().await?;
    }
    if migrate_all || specific_builtin_schemas.contains("kubernetes namespace") {
        driver
            .migrate_kubernetes_namespace(ctx, "Kubernetes", NODE_COLOR_KUBERNETES)
            .await?;
        ctx.blocking_commit().await?;
    }

    // Perform migrations on test-exclusive schemas.
    if migrate_all || migrate_test_exclusive || specific_builtin_schemas.contains("starfield") {
        driver.migrate_test_exclusive_starfield(ctx).await?;
    }
    if migrate_all || migrate_test_exclusive || specific_builtin_schemas.contains("fallout") {
        driver.migrate_test_exclusive_fallout(ctx).await?;
    }

    Ok(())
}

/// A _private_ item containing useful metadata alongside a [`FuncId`](crate::Func). This is used by
/// the [`MigrationDriver`].
#[derive(Copy, Clone, Debug)]
struct FuncCacheItem {
    pub func_id: FuncId,
    pub func_binding_id: FuncBindingId,
    pub func_binding_return_value_id: FuncBindingReturnValueId,
    pub func_argument_id: FuncArgumentId,
}

/// This _private_ driver providing caches and helper methods for efficiently creating builtin
/// [`Schemas`](crate::Schema).
#[derive(Default)]
struct MigrationDriver {
    pub func_item_cache: HashMap<String, FuncCacheItem>,
    pub func_id_cache: HashMap<String, FuncId>,
}

impl MigrationDriver {
    /// Create a [`driver`](Self) with commonly used, cached data.
    pub async fn new(ctx: &DalContext) -> BuiltinsResult<Self> {
        let mut driver = Self::default();

        driver
            .add_func_item(
                ctx,
                "si:identity".to_string(),
                serde_json::json![{ "identity": null }],
                "identity".to_string(),
            )
            .await?;

        for builtin_func_name in ["si:validation", "si:generateYAML"] {
            driver
                .add_func_id(ctx, builtin_func_name.to_string())
                .await?;
        }

        Ok(driver)
    }

    /// Create a [`validation`](crate::validation) for a [`Prop`](crate::Prop) within a
    /// [`Schema`](crate::Schema) and [`SchemaVariant`](crate::SchemaVariant).
    ///
    /// Users of this helper should provide a [`None`] value to the "value" (or similar) field.
    pub async fn create_validation(
        &self,
        ctx: &DalContext,
        validation_kind: ValidationKind,
        prop_id: PropId,
        schema_id: SchemaId,
        schema_variant_id: SchemaVariantId,
    ) -> BuiltinsResult<()> {
        create_validation(
            ctx,
            validation_kind,
            self.get_func_id("si:validation")
                .ok_or(BuiltinsError::FuncNotFoundInMigrationCache("si:validation"))?,
            prop_id,
            schema_id,
            schema_variant_id,
        )
        .await?;

        Ok(())
    }

    /// Add a `FuncCacheItem` for a given [`Func`](crate::Func) name.
    pub async fn add_func_item(
        &mut self,
        ctx: &DalContext,
        func_name: String,
        func_binding_args: Value,
        func_argument_name: String,
    ) -> BuiltinsResult<()> {
        let func: Func = Func::find_by_attr(ctx, "name", &func_name)
            .await?
            .pop()
            .ok_or_else(|| FuncError::NotFoundByName(func_name.clone()))?;
        let func_id = *func.id();
        let (func_binding, func_binding_return_value) =
            FuncBinding::create_and_execute(ctx, func_binding_args, func_id).await?;
        let func_argument = FuncArgument::find_by_name_for_func(ctx, &func_argument_name, func_id)
            .await?
            .ok_or_else(|| {
                BuiltinsError::BuiltinMissingFuncArgument(func_name.clone(), func_argument_name)
            })?;
        self.func_item_cache.insert(
            func_name,
            FuncCacheItem {
                func_id,
                func_binding_id: *func_binding.id(),
                func_binding_return_value_id: *func_binding_return_value.id(),
                func_argument_id: *func_argument.id(),
            },
        );

        Ok(())
    }

    /// Add a [`FuncId`](crate::Func) for a given [`Func`](crate::Func) name.
    pub async fn add_func_id(&mut self, ctx: &DalContext, func_name: String) -> BuiltinsResult<()> {
        let func = Func::find_by_attr(ctx, "name", &func_name)
            .await?
            .pop()
            .ok_or_else(|| FuncError::NotFoundByName(func_name.clone()))?;
        self.func_id_cache.insert(func_name, *func.id());
        Ok(())
    }

    /// Get a `FuncCacheItem` (from the cache) for a given [`Func`](crate::Func) name.
    pub fn get_func_item(&self, name: impl AsRef<str>) -> Option<FuncCacheItem> {
        self.func_item_cache.get(name.as_ref()).copied()
    }

    /// Get a [`FuncId`](crate::Func) (from the cache) for a given [`Func`](crate::Func) name.
    pub fn get_func_id(&self, name: impl AsRef<str>) -> Option<FuncId> {
        self.func_id_cache.get(name.as_ref()).copied()
    }

    /// Create a [`Schema`](crate::Schema) with a default [`SchemaVariant`](crate::SchemaVariant) named "v0".
    ///
    /// If a UI menu name is not provided, we will fallback to the provided
    /// [`Schema`](crate::Schema) name.
    #[allow(clippy::too_many_arguments)]
    pub async fn create_schema_and_variant(
        &self,
        ctx: &DalContext,
        definition_metadata: SchemaVariantDefinitionMetadataJson,
        definition: Option<SchemaVariantDefinitionJson>,
    ) -> BuiltinsResult<
        Option<(
            Schema,
            SchemaVariant,
            RootProp,
            Option<PropCache>,
            Vec<InternalProvider>,
            Vec<ExternalProvider>,
        )>,
    > {
        self.create_schema_and_variant_with_name(ctx, definition_metadata, definition, "v0")
            .await
    }

    /// Create a [`Schema`](crate::Schema) with a default [`SchemaVariant`](crate::SchemaVariant) with custom a name
    ///
    /// If a UI menu name is not provided, we will fallback to the provided
    /// [`Schema`](crate::Schema) name.
    #[allow(clippy::too_many_arguments)]
    pub async fn create_schema_and_variant_with_name(
        &self,
        ctx: &DalContext,
        definition_metadata: SchemaVariantDefinitionMetadataJson,
        definition: Option<SchemaVariantDefinitionJson>,
        schema_variant_name: &str,
    ) -> BuiltinsResult<
        Option<(
            Schema,
            SchemaVariant,
            RootProp,
            Option<PropCache>,
            Vec<InternalProvider>,
            Vec<ExternalProvider>,
        )>,
    > {
        // NOTE(nick): There's one issue here. If the schema kind has changed, then this check will be
        // inaccurate. As a result, we will be unable to re-create the schema without manual intervention.
        // This should be fine since this code should likely only last as long as default schemas need to
        // be created... which is hopefully not long.... hopefully...
        let mut schema =
            match Schema::find_by_attr(ctx, "name", &definition_metadata.name.to_string())
                .await?
                .pop()
            {
                Some(schema) => schema,
                None => {
                    // Create the schema and a ui menu.
                    let schema = Schema::new(
                        ctx,
                        definition_metadata.name.clone(),
                        &definition_metadata.component_kind,
                    )
                    .await?;

                    let ui_menu_name = match definition_metadata.menu_name {
                        Some(ref provided_override) => provided_override.to_owned(),
                        None => definition_metadata.name.clone(),
                    };
                    let ui_menu =
                        SchemaUiMenu::new(ctx, ui_menu_name, &definition_metadata.category).await?;
                    ui_menu.set_schema(ctx, schema.id()).await?;
                    schema
                }
            };

        if schema
            .variants(ctx)
            .await?
            .iter()
            .any(|v| v.name() == schema_variant_name)
        {
            info!(
                "skipping {}:{schema_variant_name} schema variant (already migrated)",
                definition_metadata.name
            );
            return Ok(None);
        } else {
            info!(
                "migrating {}:{schema_variant_name} schema variant",
                &definition_metadata.name
            );
        }

        // NOTE(nick): D.R.Y. not desired, but feel free to do so.
        if let Some(definition) = definition {
            let (
                schema_variant,
                root_prop,
                prop_cache,
                explicit_internal_providers,
                external_providers,
            ) = SchemaVariant::new_with_definition(
                ctx,
                definition_metadata.clone(),
                definition,
                schema_variant_name,
            )
            .await?;
            schema
                .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
                .await?;
            Ok(Some((
                schema,
                schema_variant,
                root_prop,
                Some(prop_cache),
                explicit_internal_providers,
                external_providers,
            )))
        } else {
            let (schema_variant, root_prop) =
                SchemaVariant::new(ctx, *schema.id(), schema_variant_name).await?;
            schema_variant
                .set_color(ctx, definition_metadata.color)
                .await?;
            schema
                .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
                .await?;
            Ok(Some((
                schema,
                schema_variant,
                root_prop,
                None,
                vec![],
                vec![],
            )))
        }
    }

    /// Creates a [`Prop`](crate::Prop) with some common settings.
    #[allow(clippy::too_many_arguments)]
    pub async fn create_prop(
        &self,
        ctx: &DalContext,
        prop_name: &str,
        prop_kind: PropKind,
        widget_kind_and_options: Option<(WidgetKind, Option<Value>)>,
        parent_prop_id: Option<PropId>,
        doc_link: Option<String>,
        schema_variant_id: SchemaVariantId,
    ) -> BuiltinsResult<Prop> {
        let mut prop = Prop::new(
            ctx,
            prop_name,
            prop_kind,
            widget_kind_and_options,
            schema_variant_id,
            parent_prop_id,
        )
        .await?;
        if doc_link.is_some() {
            prop.set_doc_link(ctx, doc_link).await?;
        }
        Ok(prop)
    }

    /// Set a default [`Value`](serde_json::Value) for a given [`Prop`](crate::Prop) in a
    /// [`Schema`](crate::Schema) and [`SchemaVariant`](crate::SchemaVariant).
    ///
    /// **Requirements:**
    /// - The [`Prop's`](crate::Prop) [`kind`](crate::PropKind) must be
    ///   [`String`](crate::PropKind::String), [`Boolean`](crate::PropKind::Boolean), or
    ///   [`Integer`](crate::PropKind::Integer).
    /// - The parent (and entire [`Prop`](crate::Prop) lineage) must have all have their
    ///   [`kind`](crate::PropKind) be [`Object`](crate::PropKind::Object).
    ///
    /// This function should only be used _after_
    /// [`SchemaVariant::finalize()`](crate::SchemaVariant::finalize()) within a builtin migration.
    pub async fn set_default_value_for_prop<T: Serialize>(
        &self,
        ctx: &DalContext,
        prop_id: PropId,
        value: T,
    ) -> BuiltinsResult<()> {
        let prop = Prop::get_by_id(ctx, &prop_id)
            .await?
            .ok_or(BuiltinsError::PropNotFound(prop_id))?;
        Ok(prop.set_default_value(ctx, value).await?)
    }

    /// Find a single [`Func`](crate::Func) and [`FuncArgument`](crate::FuncArgument) by providing
    /// the name for each, respectively.
    pub async fn find_func_and_single_argument_by_names(
        &self,
        ctx: &DalContext,
        func_name: &str,
        func_argument_name: &str,
    ) -> BuiltinsResult<(FuncId, FuncArgumentId)> {
        // NOTE(nick): we may eventually want to make "self" mutable and perform auto caching.
        let func_name = func_name.to_string();
        let func = Func::find_by_attr(ctx, "name", &func_name)
            .await?
            .pop()
            .ok_or_else(|| SchemaError::FuncNotFound(func_name.clone()))?;
        let func_id = *func.id();
        let func_argument = FuncArgument::find_by_name_for_func(ctx, func_argument_name, func_id)
            .await?
            .ok_or_else(|| {
                BuiltinsError::BuiltinMissingFuncArgument(func_name, func_argument_name.to_string())
            })?;
        Ok((func_id, *func_argument.id()))
    }

    pub async fn link_region_prop_to_explicit_internal_provider(
        &self,
        ctx: &DalContext,
        region_prop_id: &PropId,
        func_id: FuncId,
        func_argument_id: FuncArgumentId,
        region_internal_provider: &InternalProviderId,
    ) -> BuiltinsResult<()> {
        let region_attribute_value_read_context =
            AttributeReadContext::default_with_prop(*region_prop_id);
        let region_attribute_value =
            AttributeValue::find_for_context(ctx, region_attribute_value_read_context)
                .await?
                .ok_or(BuiltinsError::AttributeValueNotFoundForContext(
                    region_attribute_value_read_context,
                ))?;
        let mut region_attribute_prototype = region_attribute_value
            .attribute_prototype(ctx)
            .await?
            .ok_or(BuiltinsError::MissingAttributePrototypeForAttributeValue)?;
        region_attribute_prototype.set_func_id(ctx, func_id).await?;
        AttributePrototypeArgument::new_for_intra_component(
            ctx,
            *region_attribute_prototype.id(),
            func_argument_id,
            *region_internal_provider,
        )
        .await?;
        Ok(())
    }

    pub async fn add_deletion_confirmation_and_workflow(
        &self,
        ctx: &DalContext,
        name: &str,
        schema_variant: &SchemaVariant,
        provider: Option<&str>,
        delete_workflow_func_name: &str,
    ) -> BuiltinsResult<()> {
        // Confirmation
        let delete_confirmation_func_name = "si:confirmationResourceNeedsDeletion";
        let delete_confirmation_func =
            Func::find_by_attr(ctx, "name", &delete_confirmation_func_name)
                .await?
                .pop()
                .ok_or_else(|| {
                    SchemaError::FuncNotFound(delete_confirmation_func_name.to_owned())
                })?;
        let delete_confirmation_func_argument_name = "resource";
        let delete_confirmation_func_argument = FuncArgument::find_by_name_for_func(
            ctx,
            delete_confirmation_func_argument_name,
            *delete_confirmation_func.id(),
        )
        .await?
        .ok_or_else(|| {
            BuiltinsError::BuiltinMissingFuncArgument(
                delete_confirmation_func_name.to_string(),
                delete_confirmation_func_argument_name.to_string(),
            )
        })?;
        let deleted_at_confirmation_func_argument_name = "deleted_at";
        let deleted_at_confirmation_func_argument = FuncArgument::find_by_name_for_func(
            ctx,
            deleted_at_confirmation_func_argument_name,
            *delete_confirmation_func.id(),
        )
        .await?
        .ok_or_else(|| {
            BuiltinsError::BuiltinMissingFuncArgument(
                delete_confirmation_func_name.to_string(),
                deleted_at_confirmation_func_argument_name.to_string(),
            )
        })?;
        SchemaVariant::add_leaf(
            ctx,
            *delete_confirmation_func.id(),
            *schema_variant.id(),
            None,
            LeafKind::Confirmation,
            vec![
                LeafInput {
                    location: LeafInputLocation::DeletedAt,
                    func_argument_id: *deleted_at_confirmation_func_argument.id(),
                },
                LeafInput {
                    location: LeafInputLocation::Resource,
                    func_argument_id: *delete_confirmation_func_argument.id(),
                },
            ],
        )
        .await
        .expect("could not add leaf");

        FuncDescription::new(
            ctx,
            *delete_confirmation_func.id(),
            *schema_variant.id(),
            FuncDescriptionContents::Confirmation {
                name: format!("{name} Needs Deletion?"),
                success_description: Some(format!("{name} doesn't need deletion!")),
                failure_description: Some(
                    format!("This {name} needs deletion. Please run the fix above to delete it!")
                        .to_string(),
                ),
                provider: provider.map(String::from),
            },
        )
        .await?;

        // Workflow
        let schema = schema_variant
            .schema(ctx)
            .await?
            .ok_or_else(|| SchemaVariantError::MissingSchema(*schema_variant.id()))?;

        let delete_workflow_func = Func::find_by_attr(ctx, "name", &delete_workflow_func_name)
            .await?
            .pop()
            .ok_or_else(|| SchemaError::FuncNotFound(delete_workflow_func_name.to_owned()))?;
        let title = format!("Delete {name}");
        let context = WorkflowPrototypeContext {
            schema_id: *schema.id(),
            schema_variant_id: *schema_variant.id(),
            ..Default::default()
        };
        let delete_workflow_prototype = WorkflowPrototype::new(
            ctx,
            *delete_workflow_func.id(),
            serde_json::Value::Null,
            context,
            title,
        )
        .await?;

        let name = "delete";
        let context = ActionPrototypeContext {
            schema_id: *schema.id(),
            schema_variant_id: *schema_variant.id(),
            ..Default::default()
        };
        ActionPrototype::new(
            ctx,
            *delete_workflow_prototype.id(),
            name,
            ActionKind::Destroy,
            context,
        )
        .await?;

        Ok(())
    }
}
