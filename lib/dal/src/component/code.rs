use serde::Deserialize;

use telemetry::prelude::*;

use crate::attribute::context::AttributeContextBuilder;
use crate::attribute::value::AttributeValue;
use crate::attribute::{context::UNSET_ID_VALUE, value::AttributeValueError};
use crate::component::ComponentResult;
use crate::func::backend::js_code_generation::FuncBackendJsCodeGenerationArgs;
use crate::func::binding::FuncBinding;
use crate::func::binding_return_value::{FuncBindingReturnValue, FuncBindingReturnValueError};

use crate::ws_event::WsEvent;
use crate::{
    AttributeReadContext, CodeGenerationPrototype, CodeLanguage, CodeView, ComponentError,
    ComponentId, DalContext, Func, Prop, PropError, StandardModel,
};

use crate::Component;

impl Component {
    /// Creates code generation [`FuncBinding`](crate::FuncBinding), a
    /// [`FuncBindingReturnValue`](crate::FuncBindingReturnValue) without a value.
    /// If "prepare" is `true`, the func will not be executed and is just a placeholder for some
    /// code generation that will be executed.
    pub async fn generate_code(&self, ctx: &DalContext, prepare: bool) -> ComponentResult<()> {
        let schema = self
            .schema(ctx)
            .await?
            .ok_or(ComponentError::NoSchema(self.id))?;
        let schema_variant = self
            .schema_variant(ctx)
            .await?
            .ok_or(ComponentError::NoSchemaVariant(self.id))?;

        // Collect all prototypes for the schema variant and the attribute value for
        // "/root/resource".
        let code_generation_prototypes =
            CodeGenerationPrototype::list_for_schema_variant(ctx, *schema_variant.id()).await?;
        let code_attribute_value =
            Self::root_child_attribute_value_for_component(ctx, "code", self.id).await?;

        // Update the resolver and the "/root/code/<prop-for-prototype>" attribute value for
        // each prototype.
        for prototype in code_generation_prototypes {
            let func = Func::get_by_id(ctx, &prototype.func_id())
                .await?
                .ok_or_else(|| ComponentError::MissingFunc(prototype.func_id().to_string()))?;

            let args = FuncBackendJsCodeGenerationArgs {
                component: self.veritech_code_generation_component(ctx).await?,
            };
            let json_args = serde_json::to_value(args)?;

            let (func_binding, _created) = FuncBinding::find_or_create(
                ctx,
                json_args,
                prototype.func_id(),
                *func.backend_kind(),
            )
            .await?;

            if prepare {
                // Empty func binding return value means the function is still being executed.
                // Even though we have a func binding return value at this point, we do not
                // want to set the "/root/code/<prop-for-prototype>" field yet.
                FuncBindingReturnValue::upsert(
                    ctx,
                    None,
                    None,
                    prototype.func_id(),
                    *func_binding.id(),
                    UNSET_ID_VALUE.into(),
                )
                .await?;
            } else {
                // We always re-execute the code generation, as the previous one might have failed
                // This is a temporary work-around until we have a battle-tested failure-detection
                // system for async tasks

                // Note for future humans - if this isn't a built in, then we need to
                // think about execution time. Probably higher up than this? But just
                // an FYI.
                let new_func_binding_return_value = func_binding.execute(ctx).await?;
                let new_value = new_func_binding_return_value.value();

                // Now, set the corresponding prop underneath "/root/code".
                let code_child_read_context = AttributeReadContext {
                    prop_id: Some(prototype.prop_id()),
                    schema_id: Some(*schema.id()),
                    schema_variant_id: Some(*schema_variant.id()),
                    component_id: Some(self.id),
                    ..AttributeReadContext::default()
                };
                let code_child_attribute_value =
                    AttributeValue::find_for_context(ctx, code_child_read_context)
                        .await?
                        .ok_or(AttributeValueError::NotFoundForReadContext(
                            code_child_read_context,
                        ))?;

                // Do not update the attribute value if the existing func binding return value
                // has the same value as the new func binding return value. We do this to ensure
                // that we do not enter an infinite loop. Essentially, if anything under "/root"
                // is updated, then code generation is re-ran. Thus, we do not want to update the
                // attribute values for "/root/code/<props-for-prototype>" unless we have to.
                if let Some(existing_func_binding_return_value) = FuncBindingReturnValue::get_by_id(
                    ctx,
                    &code_child_attribute_value.func_binding_return_value_id(),
                )
                .await?
                {
                    // Only perform the update if needed, as mentioned.
                    if existing_func_binding_return_value.value() == new_value {
                        continue;
                    }
                }

                let code_child_context =
                    AttributeContextBuilder::from(code_child_read_context).to_context()?;
                let (_, _) = AttributeValue::update_for_context(
                    ctx,
                    *code_child_attribute_value.id(),
                    Some(*code_attribute_value.id()),
                    code_child_context,
                    new_value.cloned(),
                    None,
                )
                .await?;
            }
        }

        WsEvent::code_generated(ctx, self.id).publish(ctx).await?;

        Ok(())
    }

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

        let code_attribute_value =
            Self::root_child_attribute_value_for_component(ctx, "code", component_id).await?;
        let code_prop_id = code_attribute_value.context.prop_id();
        let code_prop = Prop::get_by_id(ctx, &code_prop_id)
            .await?
            .ok_or_else(|| PropError::NotFound(code_prop_id, *ctx.visibility()))?;

        let base_read_context = AttributeReadContext {
            prop_id: None,
            schema_id: Some(*schema.id()),
            schema_variant_id: Some(*schema_variant.id()),
            component_id: Some(component_id),
            ..AttributeReadContext::default()
        };

        // Iterate over the child props of the code prop and create a code view from each one's
        // current value.
        let mut code_views: Vec<CodeView> = Vec::new();
        for child_prop in code_prop.child_props(ctx).await? {
            let child_read_context = AttributeReadContext {
                prop_id: Some(*child_prop.id()),
                ..base_read_context
            };
            let child_attribute_value = AttributeValue::find_for_context(ctx, child_read_context)
                .await?
                .ok_or(AttributeValueError::NotFoundForReadContext(
                    child_read_context,
                ))?;

            let func_binding_return_value = FuncBindingReturnValue::get_by_id(
                ctx,
                &child_attribute_value.func_binding_return_value_id(),
            )
            .await?
            .ok_or(FuncBindingReturnValueError::Missing)?;

            // Get the output format.
            let prototype = CodeGenerationPrototype::find_for_prop(ctx, *child_prop.id()).await?;
            let output_format = *prototype.output_format();

            if let Some(value) = func_binding_return_value
                .value()
                .filter(|v| *v != &serde_json::Value::String(String::new()))
            {
                let code_generated = veritech_client::CodeGenerated::deserialize(value)?;

                let lang = CodeLanguage::deserialize(serde_json::Value::String(
                    code_generated.format.clone(),
                ))
                .unwrap_or_else(|err| {
                    error!(
                        "Unable to identify format {} ({err})",
                        code_generated.format
                    );
                    CodeLanguage::Unknown
                });

                if lang != output_format {
                    return Err(ComponentError::CodeLanguageMismatch(lang, output_format));
                }

                code_views.push(CodeView::new(output_format, Some(code_generated.code)));
            } else {
                // Means the code generation is being executed
                code_views.push(CodeView::new(output_format, None));
            }
        }
        Ok(code_views)
    }
}
