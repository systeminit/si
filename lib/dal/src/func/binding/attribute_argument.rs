use serde::{Deserialize, Serialize};
use telemetry::prelude::*;

use crate::{
    attribute::prototype::argument::{
        value_source::ValueSource, AttributePrototypeArgument, AttributePrototypeArgumentId,
    },
    func::argument::FuncArgumentId,
    DalContext, InputSocketId, PropId,
};

use super::{FuncBindingsError, FuncBindingsResult};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AttributeArgumentBinding {
    pub func_argument_id: FuncArgumentId,
    pub attribute_prototype_argument_id: Option<AttributePrototypeArgumentId>,
    pub prop_id: Option<PropId>,
    pub input_socket_id: Option<InputSocketId>,
}
impl AttributeArgumentBinding {
    pub async fn assemble(
        ctx: &DalContext,
        attribute_prototype_argument_id: AttributePrototypeArgumentId,
    ) -> FuncBindingsResult<Self> {
        let attribute_prototype_argument =
            AttributePrototypeArgument::get_by_id(ctx, attribute_prototype_argument_id).await?;

        let (input_socket_id, prop_id) = match attribute_prototype_argument
            .value_source(ctx)
            .await?
        {
            Some(value_source) => match value_source {
                ValueSource::InputSocket(input_socket_id) => (Some(input_socket_id), None),
                ValueSource::Prop(prop_id) => (None, Some(prop_id)),
                ValueSource::StaticArgumentValue(_) => {
                    warn!("unimplemented: static argument values are not yet handled in func authoring");
                    (None, None)
                }
                value_source => {
                    return Err(FuncBindingsError::UnexpectedValueSource(
                        value_source,
                        attribute_prototype_argument_id,
                    ))
                }
            },
            None => (None, None),
        };

        let func_argument_id = AttributePrototypeArgument::func_argument_id_by_id(
            ctx,
            attribute_prototype_argument_id,
        )
        .await?;

        Ok(Self {
            func_argument_id,
            attribute_prototype_argument_id: Some(attribute_prototype_argument_id),
            prop_id,
            input_socket_id,
        })
    }
}
