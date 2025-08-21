use serde::{
    Deserialize,
    Serialize,
};
use serde_json::Value;
use strum::Display;

use super::{
    FuncBindingError,
    FuncBindingResult,
};
use crate::{
    DalContext,
    InputSocketId,
    OutputSocketId,
    PropId,
    SecretId,
    attribute::prototype::argument::{
        AttributePrototypeArgument,
        AttributePrototypeArgumentId,
        static_value::StaticArgumentValue,
        value_source::ValueSource,
    },
    func::argument::FuncArgumentId,
};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash, Display)]
pub enum AttributeFuncArgumentSource {
    Prop(PropId),
    InputSocket(InputSocketId),
    StaticArgument(Value),
    OutputSocket(OutputSocketId),
    Secret(SecretId),
}

impl From<AttributeFuncArgumentSource> for Option<si_events::PropId> {
    fn from(value: AttributeFuncArgumentSource) -> Self {
        match value {
            AttributeFuncArgumentSource::Prop(prop_id) => {
                Some(::si_events::PropId::from_raw_id(prop_id.into()))
            }
            AttributeFuncArgumentSource::InputSocket(_) => None,
            AttributeFuncArgumentSource::StaticArgument(_) => None,
            AttributeFuncArgumentSource::OutputSocket(_) => None,
            AttributeFuncArgumentSource::Secret(_) => None,
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
            AttributeFuncArgumentSource::OutputSocket(_) => None,
            AttributeFuncArgumentSource::Secret(_) => None,
        }
    }
}

impl From<AttributeFuncArgumentSource> for Option<Value> {
    fn from(value: AttributeFuncArgumentSource) -> Self {
        match value {
            AttributeFuncArgumentSource::Prop(_) => None,
            AttributeFuncArgumentSource::InputSocket(_) => None,
            AttributeFuncArgumentSource::StaticArgument(val) => Some(val),
            AttributeFuncArgumentSource::OutputSocket(_) => None,
            AttributeFuncArgumentSource::Secret(_) => None,
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
            func_argument_id: self.func_argument_id,
            attribute_prototype_argument_id: self.attribute_prototype_argument_id,
            prop_id: self.attribute_func_input_location.clone().into(),
            input_socket_id: self.attribute_func_input_location.clone().into(),
            static_value: self.attribute_func_input_location.clone().into(),
        }
    }

    pub async fn assemble(
        ctx: &DalContext,
        apa_id: AttributePrototypeArgumentId,
    ) -> FuncBindingResult<Self> {
        let attribute_func_input_location =
            match AttributePrototypeArgument::value_source(ctx, apa_id).await? {
                ValueSource::InputSocket(input_socket_id) => {
                    AttributeFuncArgumentSource::InputSocket(input_socket_id)
                }
                ValueSource::Prop(prop_id) => AttributeFuncArgumentSource::Prop(prop_id),
                ValueSource::StaticArgumentValue(static_argument_id) => {
                    let static_value =
                        StaticArgumentValue::get_by_id(ctx, static_argument_id).await?;
                    AttributeFuncArgumentSource::StaticArgument(static_value.value)
                }
                value_source => {
                    return Err(FuncBindingError::UnexpectedValueSource(
                        value_source,
                        apa_id,
                    ));
                }
            };

        let func_argument_id = AttributePrototypeArgument::func_argument_id(ctx, apa_id).await?;

        Ok(Self {
            func_argument_id,
            attribute_prototype_argument_id: Some(apa_id),
            attribute_func_input_location,
        })
    }
}
