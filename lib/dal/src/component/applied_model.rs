use crate::{
    attribute::context::AttributeContextBuilder, component::ComponentResult, AttributeValue,
    AttributeValueError, Component, ComponentError, ComponentView, ComponentViewProperties,
    DalContext, RootPropChild, StandardModel,
};

impl Component {
    /// Sets the "string" field, "/root/applied_model" with a given value. After that, ensure dependent
    /// [`AttributeValues`](crate::AttributeValue) are updated.
    pub async fn set_applied_model(&self, ctx: &DalContext) -> ComponentResult<()> {
        if !ctx.visibility().is_head() {
            return Err(ComponentError::CannotUpdateAppliedModelTreeInChangeSet);
        }

        let applied_model_attribute_value =
            Component::root_prop_child_attribute_value_for_component(
                ctx,
                self.id,
                RootPropChild::AppliedModel,
            )
            .await?;

        let root_attribute_value = applied_model_attribute_value
            .parent_attribute_value(ctx)
            .await?
            .ok_or_else(|| {
                AttributeValueError::ParentNotFound(*applied_model_attribute_value.id())
            })?;

        let update_attribute_context =
            AttributeContextBuilder::from(applied_model_attribute_value.context)
                .set_component_id(self.id)
                .to_context()?;

        let view = ComponentView::new(ctx, self.id).await?;
        let mut properties = ComponentViewProperties::try_from(view)?;

        // Note(paulo): do we want to detect changes to code? As it's generally used in fixes
        // The function might have been edited and re-ran, so it would change the real life resource?
        properties.drop_all_but_domain();

        let (_, _) = AttributeValue::update_for_context(
            ctx,
            *applied_model_attribute_value.id(),
            Some(*root_attribute_value.id()),
            update_attribute_context,
            Some(serde_json::to_value(properties)?),
            None,
        )
        .await?;

        Ok(())
    }
}
