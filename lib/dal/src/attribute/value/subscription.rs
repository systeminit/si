use si_id::{
    AttributeValueId,
    PropId,
};

use super::{
    AttributeValue,
    AttributeValueResult,
};
use crate::{
    Component,
    DalContext,
    attribute::path::AttributePath,
};

/// A subscription to an attribute value: the root value and path relative to that value
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ValueSubscription {
    // The root attribute value
    pub attribute_value_id: AttributeValueId,
    // The path to the actual attribute value, relative to the root
    pub path: AttributePath,
}

impl ValueSubscription {
    /// Find the attribute value a subscription points to
    /// Returns `None` if the path leads to an attribute value that does not exist
    pub async fn resolve(
        &self,
        ctx: &DalContext,
    ) -> AttributeValueResult<Option<AttributeValueId>> {
        self.path.resolve(ctx, self.attribute_value_id).await
    }

    /// Validate the subscription path matches the schema of the attribute value
    pub async fn validate(&self, ctx: &DalContext) -> AttributeValueResult<PropId> {
        let prop_id = AttributeValue::prop_id(ctx, self.attribute_value_id).await?;
        self.path.validate(ctx, prop_id).await
    }

    /// Get the value, formatted for debugging/display.
    pub async fn fmt_title(&self, ctx: &DalContext) -> String {
        self.fmt_title_fallible(ctx)
            .await
            .unwrap_or_else(|e| e.to_string())
    }
    pub async fn fmt_title_fallible(&self, ctx: &DalContext) -> AttributeValueResult<String> {
        // If the subscription somehow isn't to a root attribute value, make it so.
        let (root_id, child_path) =
            AttributeValue::path_from_root(ctx, self.attribute_value_id).await?;
        let component_id = AttributeValue::component_id(ctx, root_id).await?;
        if root_id != self.attribute_value_id {
            return Ok(format!(
                "subscription to {} on (child AV {} on {})",
                self.path,
                child_path,
                Component::fmt_title(ctx, component_id).await,
            ));
        }
        Ok(format!(
            "subscription to {} on {}",
            self.path,
            Component::fmt_title(ctx, component_id).await,
        ))
    }
}
