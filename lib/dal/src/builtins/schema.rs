use crate::attribute::context::AttributeContextBuilder;
use crate::{
    component::ComponentKind,
    func::{
        binding::{FuncBinding, FuncBindingId},
        binding_return_value::FuncBindingReturnValueId,
    },
    AttributeReadContext, AttributeValue, BuiltinsError, BuiltinsResult, DalContext, Func,
    FuncError, FuncId, Prop, PropError, PropId, PropKind, Schema, SchemaId, SchemaKind,
    SchemaVariantId, StandardModel,
};
use serde_json::Value;

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

/// This unit struct (zero bytes) provides a singular place to index helpers for creating builtin
/// [`Schemas`](crate::Schema).
pub struct BuiltinSchemaHelpers;

impl BuiltinSchemaHelpers {
    pub async fn create_schema(
        ctx: &DalContext,
        schema_name: &str,
        schema_kind: &SchemaKind,
    ) -> BuiltinsResult<Option<Schema>> {
        // TODO(nick): there's one issue here. If the schema kind has changed, then this check will be
        // inaccurate. As a result, we will be unable to re-create the schema without manual intervention.
        // This should be fine since this code should likely only last as long as default schemas need to
        // be created... which is hopefully not long.... hopefully...
        let default_schema_exists = !Schema::find_by_attr(ctx, "name", &schema_name.to_string())
            .await?
            .is_empty();

        // TODO(nick): this should probably return an "AlreadyExists" error instead of "None", but
        // since the calling function would have to deal with the result similarly, this should suffice
        // for now.
        match default_schema_exists {
            true => Ok(None),
            false => {
                let schema =
                    Schema::new(ctx, schema_name, schema_kind, &ComponentKind::Standard).await?;
                Ok(Some(schema))
            }
        }
    }

    /// Creates a [`Prop`]. While a base [`AttributeReadContext`] is required for this function, it is
    /// only used when a parent [`PropId`] is provided.
    #[allow(clippy::too_many_arguments)]
    pub async fn create_prop(
        ctx: &DalContext,
        prop_name: &str,
        prop_kind: PropKind,
        parent_prop_id: Option<PropId>,
        doc_link: Option<String>,
    ) -> BuiltinsResult<Prop> {
        let mut prop = Prop::new(ctx, prop_name, prop_kind).await?;
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
    ) -> BuiltinsResult<(FuncId, FuncBindingId, FuncBindingReturnValueId)> {
        let identity_func_name = "si:identity".to_string();
        let identity_func: Func = Func::find_by_attr(ctx, "name", &identity_func_name)
            .await?
            .pop()
            .ok_or(FuncError::NotFoundByName(identity_func_name))?;
        let (identity_func_binding, identity_func_binding_return_value) =
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
}
