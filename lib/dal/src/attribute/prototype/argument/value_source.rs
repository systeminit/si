use thiserror::Error;

use crate::{
    prop::PropError,
    provider::{
        external::ExternalProviderError,
        internal::{InternalProviderError, InternalProviderId},
    },
    AttributeValue, AttributeValueId, ComponentId, DalContext, ExternalProvider,
    ExternalProviderId, InternalProvider, Prop, PropId,
};

use super::static_value::StaticArgumentValueId;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum ValueSourceError {
    #[error("attribute value error: {0}")]
    AttributeValue(String),
    #[error("external provider error: {0}")]
    ExternalProvider(#[from] ExternalProviderError),
    #[error("internal provider error: {0}")]
    InternalProvider(#[from] InternalProviderError),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("static argument value sources have no attribute values")]
    StaticArgumentValueSourcesNoValues,
}

pub type ValueSourceResult<T> = Result<T, ValueSourceError>;

#[remain::sorted]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ValueSource {
    ExternalProvider(ExternalProviderId),
    InternalProvider(InternalProviderId),
    Prop(PropId),
    StaticArgumentValue(StaticArgumentValueId),
}

impl ValueSource {
    pub async fn attribute_values(
        &self,
        ctx: &DalContext,
    ) -> ValueSourceResult<Vec<AttributeValueId>> {
        Ok(match self {
            Self::Prop(prop_id) => Prop::attribute_values_for_prop_id(ctx, *prop_id).await?,
            Self::ExternalProvider(ep_id) => {
                ExternalProvider::attribute_values_for_external_provider_id(ctx, *ep_id).await?
            }
            Self::InternalProvider(ip_id) => {
                InternalProvider::attribute_values_for_internal_provider_id(ctx, *ip_id).await?
            }
            Self::StaticArgumentValue(_) => {
                return Err(ValueSourceError::StaticArgumentValueSourcesNoValues)
            }
        })
    }

    pub async fn attribute_values_for_component_id(
        &self,
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ValueSourceResult<Vec<AttributeValueId>> {
        let mut result = vec![];

        for value_id in self.attribute_values(ctx).await? {
            if AttributeValue::component_id(ctx, value_id)
                .await
                .map_err(|err| ValueSourceError::AttributeValue(err.to_string()))?
                == component_id
            {
                result.push(value_id);
            }
        }

        Ok(result)
    }
}
