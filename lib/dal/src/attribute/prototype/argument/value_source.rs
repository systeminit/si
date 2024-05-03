use core::fmt;

use thiserror::Error;

use crate::{
    attribute::value::AttributeValueError,
    prop::PropError,
    socket::{
        input::{InputSocketError, InputSocketId},
        output::OutputSocketError,
    },
    AttributeValue, AttributeValueId, ComponentId, DalContext, InputSocket, OutputSocket,
    OutputSocketId, Prop, PropId, SecretId,
};

use super::static_value::StaticArgumentValueId;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum ValueSourceError {
    #[error("attribute value error: {0}")]
    AttributeValue(String),
    #[error("input socket error: {0}")]
    InputSocket(#[from] InputSocketError),
    #[error("output socket error: {0}")]
    OutputSocket(#[from] OutputSocketError),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("secret sources have no attribute values: {0}")]
    SecretSourcesNoValues(SecretId),
    #[error("static argument value sources have no attribute values")]
    StaticArgumentValueSourcesNoValues,
}

pub type ValueSourceResult<T> = Result<T, ValueSourceError>;

#[remain::sorted]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ValueSource {
    InputSocket(InputSocketId),
    OutputSocket(OutputSocketId),
    Prop(PropId),
    Secret(SecretId),
    StaticArgumentValue(StaticArgumentValueId),
}

impl ValueSource {
    pub async fn attribute_values(
        &self,
        ctx: &DalContext,
    ) -> ValueSourceResult<Vec<AttributeValueId>> {
        Ok(match self {
            Self::Prop(prop_id) => Prop::attribute_values_for_prop_id(ctx, *prop_id).await?,
            Self::OutputSocket(ep_id) => {
                OutputSocket::attribute_values_for_output_socket_id(ctx, *ep_id).await?
            }
            Self::InputSocket(ip_id) => {
                InputSocket::attribute_values_for_input_socket_id(ctx, *ip_id).await?
            }
            Self::Secret(secret_id) => {
                return Err(ValueSourceError::SecretSourcesNoValues(*secret_id))
            }
            Self::StaticArgumentValue(_) => {
                return Err(ValueSourceError::StaticArgumentValueSourcesNoValues);
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
            let value_component_id = match AttributeValue::component_id(ctx, value_id).await {
                Ok(component_id) => Some(component_id),
                // If this is a child value of a value set by a dynamic
                // function, we might encounter an orphaned value (since we're
                // walking backwards from the prop to the values here). That's
                // fine, just skip it. It will be cleaned up on write.
                Err(AttributeValueError::OrphanedAttributeValue(_)) => None,
                Err(other_err) => Err(ValueSourceError::AttributeValue(other_err.to_string()))?,
            };

            if value_component_id == Some(component_id) {
                result.push(value_id);
            }
        }

        Ok(result)
    }
}
impl fmt::Display for ValueSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValueSource::InputSocket(input_socket_id) => {
                write!(f, "ValueSource::InputSocket({input_socket_id})")
            }
            ValueSource::OutputSocket(output_socket_id) => {
                write!(f, "ValueSource::OutputSocket({output_socket_id})")
            }
            ValueSource::Prop(prop_id) => {
                write!(f, "ValueSource::Prop({prop_id})")
            }
            ValueSource::Secret(secret_id) => {
                write!(f, "ValueSource::Secret({secret_id})")
            }
            ValueSource::StaticArgumentValue(id) => {
                write!(f, "ValueSource::StaticArgumentValue({id})")
            }
        }
    }
}
