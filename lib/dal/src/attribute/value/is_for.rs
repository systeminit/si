use serde::{
    Deserialize,
    Serialize,
};
use si_events::ulid::Ulid;

use super::AttributeValueResult;
use crate::{
    DalContext,
    InputSocket,
    InputSocketId,
    OutputSocket,
    OutputSocketId,
    Prop,
    PropId,
    SchemaVariantId,
};

/// What "thing" on the schema variant, (either a prop, input socket, or output socket),
/// is a particular value the value of/for?
#[derive(Deserialize, Serialize, Debug, Copy, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase", tag = "kind", content = "id")]
pub enum ValueIsFor {
    Prop(PropId),
    InputSocket(InputSocketId),
    OutputSocket(OutputSocketId),
}

impl ValueIsFor {
    pub fn prop_id(&self) -> Option<PropId> {
        match self {
            ValueIsFor::Prop(prop_id) => Some(*prop_id),
            _ => None,
        }
    }

    pub fn output_socket_id(&self) -> Option<OutputSocketId> {
        match self {
            ValueIsFor::OutputSocket(id) => Some(*id),
            _ => None,
        }
    }

    pub fn input_socket_id(&self) -> Option<InputSocketId> {
        match self {
            ValueIsFor::InputSocket(id) => Some(*id),
            _ => None,
        }
    }

    pub async fn find_matching_in_other_variant(
        &self,
        ctx: &DalContext,
        other_variant_id: SchemaVariantId,
    ) -> AttributeValueResult<Option<ValueIsFor>> {
        Ok(match self {
            ValueIsFor::Prop(prop_id) => {
                let path = Prop::path_by_id(ctx, *prop_id).await?;
                Prop::find_prop_id_by_path_opt(ctx, other_variant_id, &path)
                    .await?
                    .map(ValueIsFor::Prop)
            }
            ValueIsFor::InputSocket(input_socket_id) => {
                let input_socket = InputSocket::get_by_id(ctx, *input_socket_id).await?;
                InputSocket::find_with_name(ctx, input_socket.name(), other_variant_id)
                    .await?
                    .map(|input_socket| ValueIsFor::InputSocket(input_socket.id()))
            }
            ValueIsFor::OutputSocket(output_socket_id) => {
                let output_socket = OutputSocket::get_by_id(ctx, *output_socket_id).await?;
                OutputSocket::find_with_name(ctx, output_socket.name(), other_variant_id)
                    .await?
                    .map(|output_socket| ValueIsFor::OutputSocket(output_socket.id()))
            }
        })
    }

    pub async fn debug_info(&self, ctx: &DalContext) -> AttributeValueResult<String> {
        Ok(match self {
            ValueIsFor::OutputSocket(output_socket_id) => {
                let socket = OutputSocket::get_by_id(ctx, *output_socket_id).await?;
                format!("Output Socket: {}", socket.name())
            }
            ValueIsFor::InputSocket(input_socket_id) => {
                let socket = InputSocket::get_by_id(ctx, *input_socket_id).await?;
                format!("Input Socket: {}", socket.name())
            }
            ValueIsFor::Prop(prop_id) => {
                let prop = Prop::get_by_id(ctx, *prop_id).await?;
                format!("Prop: {}", prop.path(ctx).await?.with_replaced_sep("."))
            }
        })
    }
}

impl From<ValueIsFor> for Ulid {
    fn from(value: ValueIsFor) -> Self {
        match value {
            ValueIsFor::OutputSocket(output_socket_id) => output_socket_id.into(),
            ValueIsFor::InputSocket(input_socket_id) => input_socket_id.into(),
            ValueIsFor::Prop(prop_id) => prop_id.into(),
        }
    }
}

impl From<PropId> for ValueIsFor {
    fn from(value: PropId) -> Self {
        Self::Prop(value)
    }
}

impl From<OutputSocketId> for ValueIsFor {
    fn from(value: OutputSocketId) -> Self {
        Self::OutputSocket(value)
    }
}

impl From<InputSocketId> for ValueIsFor {
    fn from(value: InputSocketId) -> Self {
        Self::InputSocket(value)
    }
}
