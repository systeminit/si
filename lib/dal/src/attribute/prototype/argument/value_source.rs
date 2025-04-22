use core::fmt;

use si_events::ulid::Ulid;
use thiserror::Error;

use super::static_value::StaticArgumentValueId;
use crate::{
    AttributeValue,
    AttributeValueId,
    ComponentError,
    ComponentId,
    DalContext,
    InputSocket,
    OutputSocket,
    OutputSocketId,
    Prop,
    PropId,
    SecretId,
    attribute::value::AttributeValueError,
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
    #[error("attribute value error: {0}")]
    AttributeValue(String),
    #[error("component error: {0}")]
    Component(#[from] Box<ComponentError>),
    #[error("source has more than one attribute values when only one was expected: {0}")]
    ComponentHasMultipleValues(ComponentId, ValueSource),
    #[error("source has no attribute values for component {1}: {0}")]
    ComponentHasNoValues(ComponentId, ValueSource),
    #[error("input socket error: {0}")]
    InputSocket(#[from] InputSocketError),
    #[error("output socket error: {0}")]
    OutputSocket(#[from] OutputSocketError),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("source has no attribute values: {0}")]
    SourceHasNoValues(ValueSource),
}

pub type ValueSourceResult<T> = Result<T, ValueSourceError>;

#[remain::sorted]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ValueSource {
    InputSocket(InputSocketId),
    OutputSocket(OutputSocketId),
    Prop(PropId),
    Secret(SecretId),
    StaticArgumentValue(StaticArgumentValueId),
}

impl From<ValueSource> for si_events::ulid::Ulid {
    fn from(value_source: ValueSource) -> Self {
        value_source.into_inner_id()
    }
}

impl From<InputSocketId> for ValueSource {
    fn from(id: InputSocketId) -> Self {
        Self::InputSocket(id)
    }
}
impl From<OutputSocketId> for ValueSource {
    fn from(id: OutputSocketId) -> Self {
        Self::OutputSocket(id)
    }
}
impl From<PropId> for ValueSource {
    fn from(id: PropId) -> Self {
        Self::Prop(id)
    }
}
impl From<SecretId> for ValueSource {
    fn from(id: SecretId) -> Self {
        Self::Secret(id)
    }
}
impl From<StaticArgumentValueId> for ValueSource {
    fn from(id: StaticArgumentValueId) -> Self {
        Self::StaticArgumentValue(id)
    }
}

impl ValueSource {
    async fn all_attribute_values_everywhere(
        &self,
        ctx: &DalContext,
    ) -> ValueSourceResult<Vec<AttributeValueId>> {
        Ok(match self {
            Self::Prop(prop_id) => {
                Prop::all_attribute_values_everywhere_for_prop_id(ctx, *prop_id).await?
            }
            Self::OutputSocket(ep_id) => {
                OutputSocket::all_attribute_values_everywhere_for_output_socket_id(ctx, *ep_id)
                    .await?
            }
            Self::InputSocket(ip_id) => {
                InputSocket::all_attribute_values_everywhere_for_input_socket_id(ctx, *ip_id)
                    .await?
            }
            Self::Secret(_) => return Err(ValueSourceError::SourceHasNoValues(*self)),
            Self::StaticArgumentValue(_) => {
                return Err(ValueSourceError::SourceHasNoValues(*self));
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
                Err(other_err) => Err(ValueSourceError::AttributeValue(other_err.to_string()))?,
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
                *self,
            ));
        }
        match values.first() {
            Some(value) => Ok(*value),
            None => Err(ValueSourceError::ComponentHasNoValues(component_id, *self)),
        }
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
