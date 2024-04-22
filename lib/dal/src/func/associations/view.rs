use serde::{Deserialize, Serialize};

use crate::attribute::prototype::argument::value_source::ValueSource;
use crate::attribute::prototype::argument::{
    AttributePrototypeArgument, AttributePrototypeArgumentId,
};
use crate::attribute::prototype::AttributePrototypeEventualParent;
use crate::func::argument::FuncArgumentId;
use crate::func::associations::FuncAssociationsResult;
use crate::{
    AttributePrototype, AttributePrototypeId, ComponentId, DalContext, InputSocketId,
    OutputSocketId, PropId, SchemaVariantId,
};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AttributePrototypeArgumentView {
    pub func_argument_id: FuncArgumentId,
    pub id: AttributePrototypeArgumentId,
    pub input_socket_id: Option<InputSocketId>,
}

impl AttributePrototypeArgumentView {
    pub async fn assemble(
        ctx: &DalContext,
        id: AttributePrototypeArgumentId,
    ) -> FuncAssociationsResult<Self> {
        let attribute_prototype_argument = AttributePrototypeArgument::get_by_id(ctx, id).await?;

        let input_socket_id =
            if let Some(value_source) = attribute_prototype_argument.value_source(ctx).await? {
                match value_source {
                    ValueSource::InputSocket(input_socket_id) => Some(input_socket_id),
                    ValueSource::OutputSocket(_)
                    | ValueSource::Prop(_)
                    | ValueSource::StaticArgumentValue(_) => None,
                }
            } else {
                None
            };

        let func_argument_id = AttributePrototypeArgument::func_argument_id_by_id(ctx, id).await?;

        Ok(Self {
            func_argument_id,
            id,
            input_socket_id,
        })
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AttributePrototypeView {
    pub id: AttributePrototypeId,
    pub component_id: Option<ComponentId>,
    pub schema_variant_id: Option<SchemaVariantId>,
    pub prop_id: Option<PropId>,
    pub output_socket_id: Option<OutputSocketId>,
    pub prototype_arguments: Vec<AttributePrototypeArgumentView>,
}

impl AttributePrototypeView {
    pub async fn assemble(
        ctx: &DalContext,
        id: AttributePrototypeId,
    ) -> FuncAssociationsResult<Self> {
        let attribute_prototype_argument_ids =
            AttributePrototypeArgument::list_ids_for_prototype(ctx, id).await?;

        let eventual_parent = AttributePrototype::eventual_parent(ctx, id).await?;
        let (component_id, schema_variant_id, prop_id, output_socket_id) = match eventual_parent {
            AttributePrototypeEventualParent::Component(component_id) => {
                (Some(component_id), None, None, None)
            }
            AttributePrototypeEventualParent::SchemaVariantFromInputSocket(
                schema_variant_id,
                _,
            ) => (None, Some(schema_variant_id), None, None),
            AttributePrototypeEventualParent::SchemaVariantFromOutputSocket(
                schema_variant_id,
                output_socket_id,
            ) => (None, Some(schema_variant_id), None, Some(output_socket_id)),
            AttributePrototypeEventualParent::SchemaVariantFromProp(schema_variant_id, prop_id) => {
                (None, Some(schema_variant_id), Some(prop_id), None)
            }
        };

        let mut prototype_arguments = Vec::new();
        for attribute_prototype_argument_id in attribute_prototype_argument_ids {
            prototype_arguments.push(
                AttributePrototypeArgumentView::assemble(ctx, attribute_prototype_argument_id)
                    .await?,
            );
        }

        Ok(Self {
            id,
            component_id,
            schema_variant_id,
            prop_id,
            output_socket_id,
            prototype_arguments,
        })
    }
}
