use serde::{Deserialize, Serialize};
use serde_json::Value;

use telemetry::prelude::*;

use crate::attribute::context::AttributeContextBuilder;
use crate::edit_field::widget::WidgetKind;
use crate::func::argument::{FuncArgument, FuncArgumentId};
use crate::func::backend::validation::FuncBackendValidationArgs;
use crate::schema::RootProp;
use crate::validation::Validation;
use crate::{
    component::ComponentKind,
    func::{
        binding::{FuncBinding, FuncBindingId},
        binding_return_value::FuncBindingReturnValueId,
    },
    AttributeReadContext, AttributeValue, BuiltinsError, BuiltinsResult, DalContext, Func,
    FuncError, FuncId, Prop, PropError, PropId, PropKind, Schema, SchemaId, SchemaKind,
    SchemaVariant, SchemaVariantId, StandardModel, ValidationPrototype, ValidationPrototypeContext,
};

mod aws;
mod coreos;
mod docker;
mod kubernetes;
mod systeminit;

pub async fn migrate(ctx: &DalContext) -> BuiltinsResult<()> {
    systeminit::migrate(ctx).await?;
    docker::migrate(ctx).await?;
    kubernetes::migrate(ctx).await?;
    coreos::migrate(ctx).await?;
    aws::migrate(ctx).await?;
    Ok(())
}

// TODO(nick): once shape is finalized and we stop serializing this within builtins, please
// move to a more formal type in the property editor module.
#[derive(Deserialize, Serialize, Debug)]
pub struct SelectWidgetOption {
    label: String,
    value: String,
}

/// This unit struct (zero bytes) provides a singular place to index helpers for creating builtin
/// [`Schemas`](crate::Schema).
pub struct BuiltinSchemaHelpers;

impl BuiltinSchemaHelpers {
    /// Create a [`Schema`](crate::Schema) and [`SchemaVariant`](crate::SchemaVariant). In addition, set the node
    /// color for the given [`SchemaKind`](crate::SchemaKind) (which may correspond to a
    /// [`DiagramKind`](crate::DiagramKind)).
    pub async fn create_schema_and_variant(
        ctx: &DalContext,
        name: impl AsRef<str>,
        kind: SchemaKind,
        component_kind: ComponentKind,
        node_color: Option<i64>,
    ) -> BuiltinsResult<Option<(Schema, SchemaVariant, RootProp)>> {
        let name = name.as_ref();
        info!("migrating {name} schema");

        // NOTE(nick): There's one issue here. If the schema kind has changed, then this check will be
        // inaccurate. As a result, we will be unable to re-create the schema without manual intervention.
        // This should be fine since this code should likely only last as long as default schemas need to
        // be created... which is hopefully not long.... hopefully...

        let default_schema_exists = !Schema::find_by_attr(ctx, "name", &name.to_string())
            .await?
            .is_empty();
        if default_schema_exists {
            return Ok(None);
        }

        let mut schema = Schema::new(ctx, name, &kind, &component_kind).await?;
        let (mut schema_variant, root_prop) = SchemaVariant::new(ctx, *schema.id(), "v0").await?;
        if node_color.is_some() {
            schema_variant.set_color(ctx, node_color).await?;
        }
        schema
            .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
            .await?;

        Ok(Some((schema, schema_variant, root_prop)))
    }

    /// Creates a [`Prop`](crate::Prop) with some common settings.
    #[allow(clippy::too_many_arguments)]
    pub async fn create_prop(
        ctx: &DalContext,
        prop_name: &str,
        prop_kind: PropKind,
        widget_kind_and_options: Option<(WidgetKind, Value)>,
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

    /// Get the "si:identity" [`Func`](crate::Func) and execute (if necessary).
    pub async fn setup_identity_func(
        ctx: &DalContext,
    ) -> BuiltinsResult<(
        FuncId,
        FuncBindingId,
        FuncBindingReturnValueId,
        FuncArgumentId,
    )> {
        let identity_func_name = "si:identity".to_string();
        let identity_func: Func = Func::find_by_attr(ctx, "name", &identity_func_name)
            .await?
            .pop()
            .ok_or_else(|| FuncError::NotFoundByName(identity_func_name.clone()))?;

        let identity_func_identity_arg = FuncArgument::list_for_func(ctx, *identity_func.id())
            .await?
            .pop()
            .ok_or_else(|| {
                BuiltinsError::BuiltinMissingFuncArgument(
                    identity_func_name.clone(),
                    "identity".to_string(),
                )
            })?;

        let (identity_func_binding, identity_func_binding_return_value, _) =
            FuncBinding::find_or_create_and_execute(
                ctx,
                serde_json::json![{ "identity": null }],
                *identity_func.id(),
            )
            .await?;

        Ok((
            *identity_func.id(),
            *identity_func_binding.id(),
            *identity_func_binding_return_value.id(),
            *identity_func_identity_arg.id(),
        ))
    }

    /// Find the child of a [`Prop`](crate::Prop) by name.
    ///
    /// _Use with caution!_
    pub async fn find_child_prop_by_name(
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
        ctx: &DalContext,
        prop_id: PropId,
        schema_id: SchemaId,
        schema_variant_id: SchemaVariantId,
        value: Value,
    ) -> BuiltinsResult<()> {
        let prop = Prop::get_by_id(ctx, &prop_id)
            .await?
            .ok_or(BuiltinsError::PropNotFound(prop_id))?;
        match prop.kind() {
            PropKind::String | PropKind::Boolean | PropKind::Integer => {
                // Every other field must be set to unset (-1) and cannot be "any".
                let base_attribute_read_context = AttributeReadContext {
                    prop_id: Some(prop_id),
                    schema_id: Some(schema_id),
                    schema_variant_id: Some(schema_variant_id),
                    ..AttributeReadContext::default()
                };

                let attribute_value =
                    AttributeValue::find_for_context(ctx, base_attribute_read_context)
                        .await?
                        .ok_or(BuiltinsError::AttributeValueNotFoundForContext(
                            base_attribute_read_context,
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

                let context =
                    AttributeContextBuilder::from(base_attribute_read_context).to_context()?;
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

    /// Create a [`validation`](crate::validation) for a [`Prop`](crate::Prop) within a
    /// [`Schema`](crate::Schema) and [`SchemaVariant`](crate::SchemaVariant).
    ///
    /// Users of this helper should provide a [`None`] value to the "value" (or similar) field.
    pub async fn create_validation(
        ctx: &DalContext,
        validation: Validation,
        prop_id: PropId,
        schema_id: SchemaId,
        schema_variant_id: SchemaVariantId,
    ) -> BuiltinsResult<()> {
        let func_name = "si:validation".to_string();
        let mut funcs = Func::find_by_attr(ctx, "name", &func_name).await?;
        let func = funcs.pop().ok_or(FuncError::NotFoundByName(func_name))?;

        let mut builder = ValidationPrototypeContext::builder();
        builder
            .set_prop_id(prop_id)
            .set_schema_id(schema_id)
            .set_schema_variant_id(schema_variant_id);

        ValidationPrototype::new(
            ctx,
            *func.id(),
            serde_json::to_value(FuncBackendValidationArgs::new(validation))?,
            builder.to_context(ctx).await?,
        )
        .await?;
        Ok(())
    }
}
