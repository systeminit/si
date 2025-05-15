use si_id::AttributeValueId;

use super::{
    AttributeValue,
    AttributeValueResult,
};
use crate::{
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
    pub async fn validate(&self, ctx: &DalContext) -> AttributeValueResult<()> {
        let prop_id = AttributeValue::prop_id(ctx, self.attribute_value_id).await?;
        self.path.validate(ctx, prop_id).await
    }
}
