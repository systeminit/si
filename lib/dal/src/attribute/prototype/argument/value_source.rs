use core::fmt;

use thiserror::Error;

use crate::{
    prop::PropError,
    socket::{
        input::{InputSocketError, InputSocketId},
        output::OutputSocketError,
    },
    AttributeValue, AttributeValueId, ComponentId, DalContext, InputSocket, OutputSocket,
    OutputSocketId, Prop, PropId,
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
impl fmt::Display for ValueSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValueSource::InputSocket(_) => write!(f, "Input Socket"),
            ValueSource::OutputSocket(_) => write!(f, "Output Socket"),
            ValueSource::Prop(_) => write!(f, "Prop"),
            ValueSource::StaticArgumentValue(_) => write!(f, "Static Argument"),
        }
    }
}
