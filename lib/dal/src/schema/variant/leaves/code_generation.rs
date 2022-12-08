//! This module contains logic for populating "/root/code" in a [`SchemaVariant`](crate::SchemaVariant).

use serde::Deserialize;
use serde::Serialize;

use crate::func::argument::FuncArgumentId;
use crate::schema::variant::{SchemaVariantError, SchemaVariantResult};
use crate::{
    AttributeContext, AttributePrototypeArgument, AttributeReadContext, AttributeValue,
    AttributeValueError, ComponentId, DalContext, Func, FuncError, FuncId, PropId, SchemaVariant,
    SchemaVariantId, StandardModel, WsEvent, WsPayload,
};

impl SchemaVariant {
    /// Insert an [`object`](crate::PropKind::Object) entry into the "/root/code"
    /// [`map`](crate::PropKind::Map) with a code generation [`Func`](crate::Func) that populates
    /// the subtree entry.
    pub async fn add_code_generation(
        ctx: &DalContext,
        func_id: FuncId,
        func_argument_id: FuncArgumentId,
        schema_variant_id: SchemaVariantId,
    ) -> SchemaVariantResult<PropId> {
        if schema_variant_id.is_none() {
            return Err(SchemaVariantError::InvalidSchemaVariant);
        }
        // NOTE(nick): perhaps, considering only finalizing once and outside of this method. We only
        // need to finalize once if multiple code generation leaves are added.
        SchemaVariant::finalize_for_id(ctx, schema_variant_id).await?;

        // Assemble the values we need to insert an object into the map.
        let item_prop = SchemaVariant::find_code_item_prop(ctx, schema_variant_id).await?;
        let map_prop = item_prop
            .parent_prop(ctx)
            .await?
            .ok_or_else(|| SchemaVariantError::ParentPropNotFound(*item_prop.id()))?;
        let map_attribute_read_context = AttributeReadContext::default_with_prop(*map_prop.id());
        let map_attribute_value = AttributeValue::find_for_context(ctx, map_attribute_read_context)
            .await?
            .ok_or(AttributeValueError::NotFoundForReadContext(
                map_attribute_read_context,
            ))?;
        let insert_attribute_context = AttributeContext::builder()
            .set_prop_id(*item_prop.id())
            .to_context()?;

        // Insert an item into the map and setup its function. The new entry is named after the func
        // name since func names must be unique for a given tenancy and visibility. If that changes,
        // then this will break.
        let func = Func::get_by_id(ctx, &func_id)
            .await?
            .ok_or(FuncError::NotFound(func_id))?;
        let inserted_attribute_value_id = AttributeValue::insert_for_context(
            ctx,
            insert_attribute_context,
            *map_attribute_value.id(),
            Some(serde_json::json![{}]),
            Some(func.name().to_string()),
        )
        .await?;
        let inserted_attribute_value = AttributeValue::get_by_id(ctx, &inserted_attribute_value_id)
            .await?
            .ok_or_else(|| {
                AttributeValueError::NotFound(inserted_attribute_value_id, *ctx.visibility())
            })?;
        let mut inserted_attribute_prototype = inserted_attribute_value
            .attribute_prototype(ctx)
            .await?
            .ok_or(AttributeValueError::MissingAttributePrototype)?;
        inserted_attribute_prototype
            .set_func_id(ctx, func_id)
            .await?;

        // Code generation relies on "/root/domain".
        let domain_implicit_internal_provider =
            SchemaVariant::find_domain_implicit_internal_provider(ctx, schema_variant_id).await?;
        AttributePrototypeArgument::new_for_intra_component(
            ctx,
            *inserted_attribute_prototype.id(),
            func_argument_id,
            *domain_implicit_internal_provider.id(),
        )
        .await?;

        // Return the prop id for the entire map so that its implicit internal provider can be
        // used for intelligence functions.
        Ok(*map_prop.id())
    }
}

// NOTE(nick): consider moving this somewhere else.
#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CodeGeneratedPayload {
    component_id: ComponentId,
}

// NOTE(nick): consider moving this somewhere else.
impl WsEvent {
    pub fn code_generated(ctx: &DalContext, component_id: ComponentId) -> Self {
        WsEvent::new(
            ctx,
            WsPayload::CodeGenerated(CodeGeneratedPayload { component_id }),
        )
    }
}
