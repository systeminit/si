use serde::Deserialize;
use telemetry::prelude::*;
use veritech_client::CodeGenerated;

use crate::attribute::value::AttributeValue;
use crate::attribute::value::AttributeValueError;
use crate::component::ComponentResult;
use crate::func::binding_return_value::{FuncBindingReturnValue, FuncBindingReturnValueError};
use crate::{
    AttributeReadContext, CodeLanguage, CodeView, ComponentError, ComponentId, DalContext,
    InternalProvider, InternalProviderError, StandardModel,
};
use crate::{Component, Prop, PropError, SchemaVariant};

impl Component {
    #[instrument(skip_all)]
    pub async fn list_code_generated(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<Vec<CodeView>> {
        let component = Self::get_by_id(ctx, &component_id)
            .await?
            .ok_or(ComponentError::NotFound(component_id))?;
        let schema = component
            .schema(ctx)
            .await?
            .ok_or(ComponentError::NoSchema(component_id))?;
        let schema_variant = component
            .schema_variant(ctx)
            .await?
            .ok_or(ComponentError::NoSchemaVariant(component_id))?;
        let base_read_context = AttributeReadContext {
            schema_id: Some(*schema.id()),
            schema_variant_id: Some(*schema_variant.id()),
            component_id: Some(component_id),
            ..AttributeReadContext::default()
        };

        // Prepare to assemble code views and access the "/root/code" prop tree.
        let mut code_views: Vec<CodeView> = Vec::new();
        let root_prop = SchemaVariant::root_prop(ctx, *schema_variant.id()).await?;
        let code_prop = Prop::get_by_id(ctx, &root_prop.code_prop_id)
            .await?
            .ok_or_else(|| PropError::NotFound(root_prop.code_prop_id, *ctx.visibility()))?;

        // Assemble a code view for each code generation tree prop.
        for code_generation_tree_prop in code_prop.child_props(ctx).await? {
            // Get the raw value for the code generated object via the prop tree.
            let tree_internal_provider =
                InternalProvider::find_for_prop(ctx, *code_generation_tree_prop.id())
                    .await?
                    .ok_or_else(|| {
                        InternalProviderError::NotFoundForProp(*code_generation_tree_prop.id())
                    })?;
            let tree_read_context = AttributeReadContext {
                internal_provider_id: Some(*tree_internal_provider.id()),
                ..base_read_context
            };
            let tree_attribute_value = AttributeValue::find_for_context(ctx, tree_read_context)
                .await?
                .ok_or(AttributeValueError::NotFoundForReadContext(
                    tree_read_context,
                ))?;
            let tree_func_binding_return_value = FuncBindingReturnValue::get_by_id(
                ctx,
                &tree_attribute_value.func_binding_return_value_id(),
            )
            .await?
            .ok_or(FuncBindingReturnValueError::Missing)?;

            // Deserialize the value into the code generated object defined by the veritech client.
            let code_generated = match tree_func_binding_return_value.value() {
                Some(value) => Some(CodeGenerated::deserialize(value)?),
                None => None,
            };

            // Assemble the code view. If the code is empty, that means it is still being generated.
            let code_view = match code_generated {
                Some(code_generated) => CodeView::new(
                    CodeLanguage::try_from(code_generated.format)?,
                    Some(code_generated.code),
                ),
                None => CodeView::new(CodeLanguage::Unknown, None),
            };
            code_views.push(code_view);
        }
        Ok(code_views)
    }
}
