use si_events::ulid::Ulid;
use telemetry::prelude::*;
use thiserror::Error;

use super::static_value::StaticArgumentValueId;
use crate::{
    AttributeValue,
    AttributeValueId,
    ComponentId,
    DalContext,
    InputSocket,
    OutputSocket,
    OutputSocketId,
    Prop,
    PropId,
    Secret,
    SecretId,
    attribute::{
        prototype::argument::static_value::StaticArgumentValue,
        value::{
            AttributeValueError,
            subscription::ValueSubscription,
        },
    },
    prop::PropError,
    socket::{
        input::{
            InputSocketError,
            InputSocketId,
        },
        output::OutputSocketError,
    },
};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum ValueSourceError {
    #[error("attribute prototype argument error: {0}")]
    AttributePrototypeArgument(#[from] Box<super::AttributePrototypeArgumentError>),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("source has more than one attribute values when only one was expected: {0}")]
    ComponentHasMultipleValues(ComponentId, ValueSource),
    #[error("source has no attribute values for component {0} at path {1:?}")]
    ComponentHasNoValues(ComponentId, ValueSource),
    #[error("input socket error: {0}")]
    InputSocket(#[from] InputSocketError),
    #[error("output socket error: {0}")]
    OutputSocket(#[from] OutputSocketError),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("secret error: {0}")]
    Secret(#[from] crate::secret::SecretError),
    #[error("source has no attribute values: {0:?}")]
    SourceHasNoValues(ValueSource),
}

pub type ValueSourceResult<T> = Result<T, ValueSourceError>;

#[remain::sorted]
#[derive(Clone, Debug, Eq, PartialEq, derive_more::From)]
pub enum ValueSource {
    InputSocket(InputSocketId),
    OutputSocket(OutputSocketId),
    Prop(PropId),
    Secret(SecretId),
    StaticArgumentValue(StaticArgumentValueId),
    ValueSubscription(ValueSubscription),
}

impl ValueSource {
    async fn all_attribute_values_everywhere(
        &self,
        ctx: &DalContext,
    ) -> ValueSourceResult<Vec<AttributeValueId>> {
        Ok(match self {
            Self::InputSocket(ip_id) => {
                InputSocket::all_attribute_values_everywhere_for_input_socket_id(ctx, *ip_id)
                    .await?
            }
            Self::OutputSocket(ep_id) => {
                OutputSocket::all_attribute_values_everywhere_for_output_socket_id(ctx, *ep_id)
                    .await?
            }
            Self::Prop(prop_id) => {
                Prop::all_attribute_values_everywhere_for_prop_id(ctx, *prop_id).await?
            }
            Self::Secret(_) | Self::StaticArgumentValue(_) | Self::ValueSubscription(_) => {
                return Err(ValueSourceError::SourceHasNoValues(self.clone()));
            }
        })
    }

    pub fn into_inner_id(self) -> Ulid {
        match self {
            ValueSource::InputSocket(id) => id.into(),
            ValueSource::OutputSocket(id) => id.into(),
            ValueSource::Prop(id) => id.into(),
            ValueSource::Secret(id) => id.into(),
            ValueSource::StaticArgumentValue(id) => id.into(),
            ValueSource::ValueSubscription(ValueSubscription {
                attribute_value_id, ..
            }) => attribute_value_id.into(),
        }
    }

    pub async fn attribute_values_for_component_id(
        &self,
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ValueSourceResult<Vec<AttributeValueId>> {
        // We'd like to use Component::attribute_values_for_prop_id and friends, but the Component
        // one in particular treats OrphanedAttributeValue as an error.
        let mut result = vec![];

        for value_id in self.all_attribute_values_everywhere(ctx).await? {
            let value_component_id = match AttributeValue::component_id(ctx, value_id).await {
                Ok(component_id) => Some(component_id),
                // If this is a child value of a value set by a dynamic
                // function, we might encounter an orphaned value (since we're
                // walking backwards from the prop to the values here). That's
                // fine, just skip it. It will be cleaned up on write.
                Err(AttributeValueError::OrphanedAttributeValue(_)) => None,
                Err(other_err) => Err(other_err)?,
            };

            if value_component_id == Some(component_id) {
                result.push(value_id);
            }
        }

        Ok(result)
    }

    pub async fn attribute_value_for_component_id(
        &self,
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ValueSourceResult<AttributeValueId> {
        // We'd like to use Component::attribute_values_for_prop_id and friends, but the Component
        // one in particular treats OrphanedAttributeValue as an error.
        let values = self
            .attribute_values_for_component_id(ctx, component_id)
            .await?;
        if values.len() > 1 {
            return Err(ValueSourceError::ComponentHasMultipleValues(
                component_id,
                self.clone(),
            ));
        }
        match values.first() {
            Some(value) => Ok(*value),
            None => Err(ValueSourceError::ComponentHasNoValues(
                component_id,
                self.clone(),
            )),
        }
    }

    /// Get the value, formatted for debugging/display.
    pub async fn fmt_title(&self, ctx: &DalContext) -> String {
        self.fmt_title_fallible(ctx)
            .await
            .unwrap_or_else(|e| e.to_string())
    }
    pub async fn fmt_title_fallible(&self, ctx: &DalContext) -> ValueSourceResult<String> {
        Ok(match self {
            &ValueSource::InputSocket(socket_id) => format!(
                "input socket {}",
                InputSocket::fmt_title(ctx, socket_id).await
            ),
            &ValueSource::OutputSocket(socket_id) => format!(
                "output socket {}",
                OutputSocket::fmt_title(ctx, socket_id).await
            ),
            &ValueSource::Prop(prop_id) => {
                format!("prop {}", Prop::fmt_title(ctx, prop_id).await)
            }
            &ValueSource::Secret(secret_id) => {
                format!("secret {}", Secret::fmt_title(ctx, secret_id).await)
            }
            &ValueSource::StaticArgumentValue(static_value_id) => {
                StaticArgumentValue::fmt_title(ctx, static_value_id).await
            }
            ValueSource::ValueSubscription(subscription) => subscription.fmt_title(ctx).await,
        })
    }
}

impl From<super::AttributePrototypeArgumentError> for ValueSourceError {
    fn from(err: super::AttributePrototypeArgumentError) -> Self {
        Box::new(err).into()
    }
}
