use super::AttributeValueResult;
use crate::{
    AttributeValueId,
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
}
