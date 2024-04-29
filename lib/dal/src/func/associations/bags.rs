//! This module contains two-way "bag" structs used for the [`Func`](crate::Func) authoring
//! experience. The addition, mutation, deletion and overall existence of these bags dictates what
//! the frontend should display as well as what the user would like to change.
//!
//! These bags are carried by [`FuncAssociations`](crate::func::FuncAssociations).

use serde::{Deserialize, Serialize};
use telemetry::prelude::*;

use crate::attribute::prototype::argument::value_source::ValueSource;
use crate::attribute::prototype::argument::{
    AttributePrototypeArgument, AttributePrototypeArgumentId,
};
use crate::attribute::prototype::AttributePrototypeEventualParent;
use crate::func::argument::{FuncArgumentId, FuncArgumentKind};
use crate::func::associations::{FuncAssociationsError, FuncAssociationsResult};
use crate::{
    AttributePrototype, AttributePrototypeId, ComponentId, DalContext, InputSocketId,
    OutputSocketId, PropId, SchemaVariantId,
};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FuncArgumentBag {
    pub id: FuncArgumentId,
    pub name: String,
    pub kind: FuncArgumentKind,
    pub element_kind: Option<FuncArgumentKind>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct AttributePrototypeArgumentBag {
    pub func_argument_id: FuncArgumentId,
    pub id: AttributePrototypeArgumentId,
    pub prop_id: Option<PropId>,
    pub input_socket_id: Option<InputSocketId>,
}

impl AttributePrototypeArgumentBag {
    pub async fn assemble(
        ctx: &DalContext,
        id: AttributePrototypeArgumentId,
    ) -> FuncAssociationsResult<Self> {
        let attribute_prototype_argument = AttributePrototypeArgument::get_by_id(ctx, id).await?;

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
                    return Err(FuncAssociationsError::UnexpectedValueSource(
                        value_source,
                        id,
                    ))
                }
            },
            None => (None, None),
        };

        let func_argument_id = AttributePrototypeArgument::func_argument_id_by_id(ctx, id).await?;

        Ok(Self {
            func_argument_id,
            id,
            prop_id,
            input_socket_id,
        })
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct AttributePrototypeBag {
    pub id: AttributePrototypeId,
    pub component_id: Option<ComponentId>,
    pub schema_variant_id: Option<SchemaVariantId>,
    pub prop_id: Option<PropId>,
    pub output_socket_id: Option<OutputSocketId>,
    pub prototype_arguments: Vec<AttributePrototypeArgumentBag>,
}

impl AttributePrototypeBag {
    pub async fn assemble(
        ctx: &DalContext,
        id: AttributePrototypeId,
    ) -> FuncAssociationsResult<Self> {
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

        let attribute_prototype_argument_ids =
            AttributePrototypeArgument::list_ids_for_prototype(ctx, id).await?;

        let mut prototype_arguments = Vec::with_capacity(attribute_prototype_argument_ids.len());
        for attribute_prototype_argument_id in attribute_prototype_argument_ids {
            prototype_arguments.push(
                AttributePrototypeArgumentBag::assemble(ctx, attribute_prototype_argument_id)
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
