use serde::Deserialize;
use telemetry::prelude::*;
use veritech_client::CodeGenerated;

use crate::attribute::value::AttributeValue;
use crate::attribute::value::AttributeValueError;
use crate::component::ComponentResult;
use crate::func::binding_return_value::{FuncBindingReturnValue, FuncBindingReturnValueError};
use crate::Component;
use crate::{
    AttributeReadContext, CodeGenerationPrototype, CodeLanguage, CodeView, ComponentError,
    ComponentId, DalContext, InternalProvider, InternalProviderError, StandardModel,
};

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
            prop_id: None,
            schema_id: Some(*schema.id()),
            schema_variant_id: Some(*schema_variant.id()),
            component_id: Some(component_id),
            ..AttributeReadContext::default()
        };

        // Assemble a code view for each prototype (corresponding to all direct props underneath
        // "/root/code".
        let mut code_views: Vec<CodeView> = Vec::new();
        for code_generation_prototype in
            CodeGenerationPrototype::list_for_schema_variant(ctx, *schema_variant.id()).await?
        {
            // Get the code and format.
            let tree_internal_provider =
                InternalProvider::find_for_prop(ctx, code_generation_prototype.tree_prop_id())
                    .await?
                    .ok_or_else(|| {
                        InternalProviderError::NotFoundForProp(
                            code_generation_prototype.tree_prop_id(),
                        )
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
