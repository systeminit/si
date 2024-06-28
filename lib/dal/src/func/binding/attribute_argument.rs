use serde::{Deserialize, Serialize};

use crate::{
    attribute::prototype::argument::{
        static_value::StaticArgumentValue, value_source::ValueSource, AttributePrototypeArgument,
        AttributePrototypeArgumentId,
    },
    func::argument::FuncArgumentId,
    DalContext, InputSocketId, PropId,
};

use super::{FuncBindingsError, FuncBindingsResult};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum AttributeFuncArgumentSource {
    Prop(PropId),
    InputSocket(InputSocketId),
    StaticArgument(String),
}

impl From<AttributeFuncArgumentSource> for Option<si_events::PropId> {
    fn from(value: AttributeFuncArgumentSource) -> Self {
        match value {
            AttributeFuncArgumentSource::Prop(prop_id) => {
                Some(::si_events::PropId::from_raw_id(prop_id.into()))
            }
            AttributeFuncArgumentSource::InputSocket(_) => None,
            AttributeFuncArgumentSource::StaticArgument(_) => None,
        }
    }
}
impl From<AttributeFuncArgumentSource> for Option<si_events::InputSocketId> {
    fn from(value: AttributeFuncArgumentSource) -> Self {
        match value {
            AttributeFuncArgumentSource::Prop(_) => None,
            AttributeFuncArgumentSource::InputSocket(input_socket_id) => Some(
                ::si_events::InputSocketId::from_raw_id(input_socket_id.into()),
            ),
            AttributeFuncArgumentSource::StaticArgument(_) => None,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct AttributeArgumentBinding {
    pub func_argument_id: FuncArgumentId,
    pub attribute_prototype_argument_id: Option<AttributePrototypeArgumentId>,
    pub attribute_func_input_location: AttributeFuncArgumentSource,
}
impl AttributeArgumentBinding {
    pub fn into_frontend_type(&self) -> si_frontend_types::AttributeArgumentBinding {
        si_frontend_types::AttributeArgumentBinding {
            func_argument_id: self.func_argument_id.into(),
            attribute_prototype_argument_id: self.attribute_prototype_argument_id.map(Into::into),
            prop_id: self.attribute_func_input_location.clone().into(),
            input_socket_id: self.attribute_func_input_location.clone().into(),
        }
    }
    pub async fn assemble(
        ctx: &DalContext,
        attribute_prototype_argument_id: AttributePrototypeArgumentId,
    ) -> FuncBindingsResult<Self> {
        let attribute_prototype_argument =
            AttributePrototypeArgument::get_by_id(ctx, attribute_prototype_argument_id).await?;

        let attribute_func_input_location =
            match attribute_prototype_argument.value_source(ctx).await? {
                Some(value_source) => match value_source {
                    ValueSource::InputSocket(input_socket_id) => {
                        AttributeFuncArgumentSource::InputSocket(input_socket_id)
                    }
                    ValueSource::Prop(prop_id) => AttributeFuncArgumentSource::Prop(prop_id),
                    ValueSource::StaticArgumentValue(static_argument_id) => {
                        let static_value =
                            StaticArgumentValue::get_by_id(ctx, static_argument_id).await?;
                        AttributeFuncArgumentSource::StaticArgument(static_value.value.to_string())
                    }
                    value_source => {
                        return Err(FuncBindingsError::UnexpectedValueSource(
                            value_source,
                            attribute_prototype_argument_id,
                        ))
                    }
                },
                None => {
                    return Err(FuncBindingsError::MissingValueSource(
                        attribute_prototype_argument_id,
                    ))
                }
            };

        let func_argument_id = AttributePrototypeArgument::func_argument_id_by_id(
            ctx,
            attribute_prototype_argument_id,
        )
        .await?;

        Ok(Self {
            func_argument_id,
            attribute_prototype_argument_id: Some(attribute_prototype_argument_id),
            attribute_func_input_location,
        })
    }
    pub fn assemble_attribute_input_location(
        prop_id: Option<si_events::PropId>,
        input_socket_id: Option<si_events::InputSocketId>,
    ) -> FuncBindingsResult<AttributeFuncArgumentSource> {
        let input_location = match (prop_id, input_socket_id) {
            (None, Some(input_socket_id)) => {
                AttributeFuncArgumentSource::InputSocket(input_socket_id.into())
            }

            (Some(prop_id), None) => AttributeFuncArgumentSource::Prop(prop_id.into()),
            _ => {
                return Err(FuncBindingsError::MalformedInput(
                    "cannot set more than one output location".to_owned(),
                ))
            }
        };
        Ok(input_location)
    }
}
