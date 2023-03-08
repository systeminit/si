use serde_json::Value;
use std::collections::{HashMap, HashSet};
use telemetry::prelude::*;

use crate::action_prototype::ActionKind;
use crate::attribute::context::AttributeContextBuilder;
use crate::edit_field::widget::WidgetKind;
use crate::func::argument::{FuncArgument, FuncArgumentId};
use crate::func::backend::validation::FuncBackendValidationArgs;
use crate::schema::variant::definition::{
    PropCache, SchemaVariantDefinitionJson, SchemaVariantDefinitionMetadataJson,
};
use crate::schema::{RootProp, SchemaUiMenu};
use crate::validation::Validation;
use crate::{
    func::{
        binding::{FuncBinding, FuncBindingId},
        binding_return_value::FuncBindingReturnValueId,
    },
    ActionPrototype, ActionPrototypeContext, AttributeReadContext, AttributeValue,
    BuiltinSchemaOption, BuiltinsError, BuiltinsResult, DalContext, ExternalProvider, Func,
    FuncDescription, FuncDescriptionContents, FuncError, FuncId, InternalProvider, LeafInput,
    LeafInputLocation, LeafKind, Prop, PropError, PropId, PropKind, Schema, SchemaError, SchemaId,
    SchemaVariant, SchemaVariantError, SchemaVariantId, StandardModel, ValidationPrototype,
    ValidationPrototypeContext, WorkflowPrototype, WorkflowPrototypeContext,
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

const NODE_COLOR_FRAMES: &str = "FFFFFF";
// Reference: https://aws.amazon.com/trademark-guidelines/
const NODE_COLOR_AWS: &str = "FF9900";
// Reference: https://getfedora.org/
const NODE_COLOR_COREOS: &str = "E26B70";
// Reference: https://www.docker.com/company/newsroom/media-resources/
const NODE_COLOR_DOCKER: &str = "4695E7";
// This node color is purely meant the complement existing node colors.
// It does not reflect an official branding Kubernetes color.
const NODE_COLOR_KUBERNETES: &str = "30BA78";

pub async fn migrate(
    ctx: &DalContext,
    builtin_schema_option: BuiltinSchemaOption,
) -> BuiltinsResult<()> {
    // Determine whether or not to migrate everything based on the option provided.
    let (migrate_all, specific_builtin_schemas) = match builtin_schema_option {
        BuiltinSchemaOption::All => {
            info!("migrating schemas");
            (true, HashSet::new())
        }
        BuiltinSchemaOption::None => {
            info!("skipping migrating schemas (this should only be possible when running tests)");
            return Ok(());
        }
        BuiltinSchemaOption::Some(provided_set) => {
            info!("migrating schemas based on a provided set of names (this should only be possible when running tests)");
            debug!("provided set of builtin schemas: {:?}", &provided_set);
            (false, provided_set)
        }
    };

    // Once we know what to migrate, create the driver.
    let driver = MigrationDriver::new(ctx).await?;

    // Perform migrations.
    // we only want to migrate the kubernetes schemas if specifically requested
    if specific_builtin_schemas.contains("kubernetes deployment") {
        driver
            .migrate_kubernetes_deployment(ctx, "Kubernetes", NODE_COLOR_KUBERNETES)
            .await?;
    }
    if specific_builtin_schemas.contains("kubernetes namespace") {
        driver
            .migrate_kubernetes_namespace(ctx, "Kubernetes", NODE_COLOR_KUBERNETES)
            .await?;
    }
    if migrate_all || specific_builtin_schemas.contains("docker image") {
        driver
            .migrate_docker_image(ctx, "Docker", NODE_COLOR_DOCKER)
            .await?;
    }
    if migrate_all || specific_builtin_schemas.contains("docker hub credential") {
        driver
            .migrate_docker_hub_credential(ctx, "Docker", NODE_COLOR_DOCKER)
            .await?;
    }
    if migrate_all || specific_builtin_schemas.contains("coreos butane") {
        driver
            .migrate_coreos_butane(ctx, "CoreOS", NODE_COLOR_COREOS)
            .await?;
    }
    if migrate_all || specific_builtin_schemas.contains("aws ami") {
        driver.migrate_aws_ami(ctx, "AWS", NODE_COLOR_AWS).await?;
    }
    if migrate_all
        || specific_builtin_schemas.contains("aws ec2")
        || specific_builtin_schemas.contains("aws ec2 instance")
    {
        driver
            .migrate_aws_ec2_instance(ctx, "AWS", NODE_COLOR_AWS)
            .await?;
    }
    if migrate_all || specific_builtin_schemas.contains("aws region") {
        driver
            .migrate_aws_region(ctx, "AWS", NODE_COLOR_AWS)
            .await?;
    }
    if migrate_all || specific_builtin_schemas.contains("aws eip") {
        driver.migrate_aws_eip(ctx, "AWS", NODE_COLOR_AWS).await?;
    }
    if migrate_all
        || specific_builtin_schemas.contains("aws key pair")
        || specific_builtin_schemas.contains("aws keypair")
    {
        driver
            .migrate_aws_keypair(ctx, "AWS", NODE_COLOR_AWS)
            .await?;
    }
    if migrate_all || specific_builtin_schemas.contains("aws ingress") {
        driver
            .migrate_aws_ingress(ctx, "AWS", NODE_COLOR_AWS)
            .await?;
    }
    if migrate_all || specific_builtin_schemas.contains("aws egress") {
        driver
            .migrate_aws_egress(ctx, "AWS", NODE_COLOR_AWS)
            .await?;
    }
    if migrate_all
        || specific_builtin_schemas.contains("aws security group")
        || specific_builtin_schemas.contains("aws securitygroup")
    {
        driver
            .migrate_aws_security_group(ctx, "AWS", NODE_COLOR_AWS)
            .await?;
    }
    if migrate_all
        || specific_builtin_schemas.contains("systeminit generic frame")
        || specific_builtin_schemas.contains("si generic frame")
        || specific_builtin_schemas.contains("generic frame")
    {
        driver
            .migrate_systeminit_generic_frame(ctx, "Frames", NODE_COLOR_FRAMES)
            .await?;
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
        validation: Validation,
        prop_id: PropId,
        schema_id: SchemaId,
        schema_variant_id: SchemaVariantId,
    ) -> BuiltinsResult<()> {
        let validation_func_id = self
            .get_func_id("si:validation")
            .ok_or(BuiltinsError::FuncNotFoundInMigrationCache("si:validation"))?;

        let mut builder = ValidationPrototypeContext::builder();
        builder
            .set_prop_id(prop_id)
            .set_schema_id(schema_id)
            .set_schema_variant_id(schema_variant_id);

        ValidationPrototype::new(
            ctx,
            validation_func_id,
            serde_json::to_value(FuncBackendValidationArgs::new(validation))?,
            builder.to_context(ctx).await?,
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
        let func: Func = Func::find_by_attr(ctx, "name", &func_name)
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

    /// Create a [`Schema`](crate::Schema) and [`SchemaVariant`](crate::SchemaVariant).
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
        // NOTE(nick): There's one issue here. If the schema kind has changed, then this check will be
        // inaccurate. As a result, we will be unable to re-create the schema without manual intervention.
        // This should be fine since this code should likely only last as long as default schemas need to
        // be created... which is hopefully not long.... hopefully...
        let default_schema_exists =
            !Schema::find_by_attr(ctx, "name", &definition_metadata.name.to_string())
                .await?
                .is_empty();
        if default_schema_exists {
            info!(
                "skipping {} schema (already migrated)",
                &definition_metadata.name
            );
            return Ok(None);
        }
        info!("migrating {} schema", &definition_metadata.name);

        // Create the schema and a ui menu.
        let mut schema = Schema::new(
            ctx,
            definition_metadata.name.clone(),
            &definition_metadata.component_kind,
        )
        .await?;

        let ui_menu_name = match definition_metadata.menu_name {
            Some(ref provided_override) => provided_override.to_owned(),
            None => definition_metadata.name.clone(),
        };
        let ui_menu = SchemaUiMenu::new(ctx, ui_menu_name, &definition_metadata.category).await?;
        ui_menu.set_schema(ctx, schema.id()).await?;

        // NOTE(nick): D.R.Y. not desired, but feel free to do so.
        if let Some(definition) = definition {
            let (
                schema_variant,
                root_prop,
                prop_cache,
                explicit_internal_providers,
                external_providers,
            ) = SchemaVariant::new_with_definition(ctx, definition_metadata.clone(), definition)
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
            let (mut schema_variant, root_prop) =
                SchemaVariant::new(ctx, *schema.id(), "v0").await?;
            schema_variant
                .set_color(ctx, Some(definition_metadata.color_as_i64()?))
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
    ) -> BuiltinsResult<Prop> {
        let mut prop = Prop::new(ctx, prop_name, prop_kind, widget_kind_and_options).await?;
        if let Some(parent_prop_id) = parent_prop_id {
            prop.set_parent_prop(ctx, parent_prop_id).await?;
        }
        if doc_link.is_some() {
            prop.set_doc_link(ctx, doc_link).await?;
        }
        Ok(prop)
    }

    /// Find the child of a [`Prop`](crate::Prop) by name.
    ///
    /// _Use with caution!_
    pub async fn find_child_prop_by_name(
        &self,
        ctx: &DalContext,
        prop_id: PropId,
        child_prop_name: &str,
    ) -> BuiltinsResult<Prop> {
        let prop = Prop::get_by_id(ctx, &prop_id)
            .await?
            .ok_or_else(|| PropError::NotFound(prop_id, *ctx.visibility()))?;
        for current in prop.child_props(ctx).await? {
            if current.name() == child_prop_name {
                return Ok(current);
            }
        }
        Err(PropError::ExpectedChildNotFound(child_prop_name.to_string()).into())
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
    pub async fn set_default_value_for_prop(
        &self,
        ctx: &DalContext,
        prop_id: PropId,
        value: Value,
    ) -> BuiltinsResult<()> {
        let prop = Prop::get_by_id(ctx, &prop_id)
            .await?
            .ok_or(BuiltinsError::PropNotFound(prop_id))?;
        match prop.kind() {
            PropKind::String | PropKind::Boolean | PropKind::Integer => {
                let attribute_read_context = AttributeReadContext::default_with_prop(prop_id);
                let attribute_value = AttributeValue::find_for_context(ctx, attribute_read_context)
                    .await?
                    .ok_or(BuiltinsError::AttributeValueNotFoundForContext(
                        attribute_read_context,
                    ))?;
                let parent_attribute_value = attribute_value
                    .parent_attribute_value(ctx)
                    .await?
                    .ok_or_else(|| {
                        BuiltinsError::AttributeValueDoesNotHaveParent(*attribute_value.id())
                    })?;

                // Ensure the parent project is an object. Technically, we should ensure that every
                // prop in entire lineage is of kind object, but this should (hopefully) suffice
                // for now. Ideally, this would be handled in a query.
                let parent_prop = Prop::get_by_id(ctx, &parent_attribute_value.context.prop_id())
                    .await?
                    .ok_or_else(|| {
                        BuiltinsError::PropNotFound(parent_attribute_value.context.prop_id())
                    })?;
                if parent_prop.kind() != &PropKind::Object {
                    return Err(BuiltinsError::ParentPropIsNotObjectForPropWithDefaultValue(
                        *parent_prop.kind(),
                    ));
                }

                let context = AttributeContextBuilder::from(attribute_read_context).to_context()?;
                AttributeValue::update_for_context(
                    ctx,
                    *attribute_value.id(),
                    Some(*parent_attribute_value.id()),
                    context,
                    Some(value),
                    None,
                )
                .await?;
                Ok(())
            }
            _ => Err(BuiltinsError::NonPrimitivePropKind(*prop.kind())),
        }
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
