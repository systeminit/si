use std::collections::{
    HashMap,
    hash_map,
};

use itertools::Itertools;
use serde::{
    Deserialize,
    Serialize,
};

use super::{
    ComponentError,
    ComponentResult,
};
use crate::{
    AttributeValue,
    AttributeValueId,
    Component,
    ComponentId,
    DalContext,
    InputSocketId,
    OutputSocketId,
};

/// Represents a given [`Component`]'s [`crate::InputSocket`], identified by its
/// (non-unique) [`InputSocketId`] and unique [`AttributeValueId`]
#[derive(Eq, Hash, PartialEq, Clone, Debug, Copy, Serialize, Deserialize)]
pub struct ComponentInputSocket {
    pub component_id: ComponentId,
    pub input_socket_id: InputSocketId,
    pub attribute_value_id: AttributeValueId,
}

/// Represents a given [`Component`]'s [`crate::OutputSocket`], identified by its
/// (non-unique) [`OutputSocketId`] and unique [`AttributeValueId`]
#[derive(Eq, Hash, PartialEq, Clone, Debug, Copy, Serialize, Deserialize)]
pub struct ComponentOutputSocket {
    pub component_id: ComponentId,
    pub output_socket_id: OutputSocketId,
    pub attribute_value_id: AttributeValueId,
}

impl ComponentOutputSocket {
    /// Given a [`ComponentId`] and [`OutputSocketId`] find the [`ComponentOutputSocket`]
    pub async fn get_by_ids(
        ctx: &DalContext,
        component_id: ComponentId,
        output_socket_id: OutputSocketId,
    ) -> ComponentResult<Option<ComponentOutputSocket>> {
        let output_socket = Self::list_for_component_id(ctx, component_id)
            .await?
            .into_iter()
            .find(|socket| socket.output_socket_id == output_socket_id);

        Ok(output_socket)
    }

    /// Given a [`ComponentId`] and [`OutputSocketId`] find the [`ComponentOutputSocket`]
    /// returns an error if one is not found
    pub async fn get_by_ids_or_error(
        ctx: &DalContext,
        component_id: ComponentId,
        output_socket_id: OutputSocketId,
    ) -> ComponentResult<ComponentOutputSocket> {
        match Self::get_by_ids(ctx, component_id, output_socket_id).await? {
            Some(component_output_socket) => Ok(component_output_socket),
            None => Err(ComponentError::OutputSocketNotFoundForComponentId(
                output_socket_id,
                component_id,
            )),
        }
    }

    /// List all [`ComponentOutputSocket`]s for a given [`ComponentId`]
    pub async fn list_for_component_id(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<Vec<Self>> {
        let mut result = Vec::new();

        let socket_values = Component::attribute_values_for_all_sockets(ctx, component_id).await?;

        for attribute_value_id in socket_values {
            if let Some(output_socket_id) = AttributeValue::is_for(ctx, attribute_value_id)
                .await?
                .output_socket_id()
            {
                result.push(ComponentOutputSocket {
                    component_id,
                    output_socket_id,
                    attribute_value_id,
                });
            }
        }
        Ok(result)
    }

    /// List all [`AttributeValueId`]s for the given [`ComponentId`]s [`crate::OutputSocket`]s
    pub async fn attribute_values_for_component_id(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<Vec<AttributeValueId>> {
        let mut result = HashMap::new();

        let socket_values = Component::attribute_values_for_all_sockets(ctx, component_id).await?;

        for socket_value_id in socket_values {
            if let Some(output_socket_id) = AttributeValue::is_for(ctx, socket_value_id)
                .await?
                .output_socket_id()
            {
                match result.entry(output_socket_id) {
                    hash_map::Entry::Vacant(entry) => {
                        entry.insert(ComponentOutputSocket {
                            component_id,
                            attribute_value_id: socket_value_id,
                            output_socket_id,
                        });
                    }
                    hash_map::Entry::Occupied(_) => {
                        return Err(ComponentError::OutputSocketTooManyAttributeValues(
                            output_socket_id,
                        ));
                    }
                }
            }
        }
        Ok(result
            .into_values()
            .map(|component_output_socket| component_output_socket.attribute_value_id)
            .collect_vec())
    }

    pub async fn value_for_output_socket_id_for_component_id_opt(
        ctx: &DalContext,
        component_id: ComponentId,
        output_socket_id: OutputSocketId,
    ) -> ComponentResult<Option<serde_json::Value>> {
        let attribute_value_id = Self::get_by_ids_or_error(ctx, component_id, output_socket_id)
            .await?
            .attribute_value_id;

        let view = AttributeValue::view(ctx, attribute_value_id).await?;

        Ok(view)
    }
}

impl ComponentInputSocket {
    /// List all [`ComponentInputSocket`]s for a given [`ComponentId`]
    pub async fn list_for_component_id(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<Vec<ComponentInputSocket>> {
        let mut result = Vec::new();

        let socket_values = Component::attribute_values_for_all_sockets(ctx, component_id).await?;

        for attribute_value_id in socket_values {
            if let Some(input_socket_id) = AttributeValue::is_for(ctx, attribute_value_id)
                .await?
                .input_socket_id()
            {
                result.push(ComponentInputSocket {
                    component_id,
                    input_socket_id,
                    attribute_value_id,
                });
            }
        }
        Ok(result)
    }

    /// Given a [`ComponentId`] and [`InputSocketId`] find the [`ComponentInputSocket`]
    pub async fn get_by_ids(
        ctx: &DalContext,
        component_id: ComponentId,
        input_socket_id: InputSocketId,
    ) -> ComponentResult<Option<ComponentInputSocket>> {
        let input_socket = Self::list_for_component_id(ctx, component_id)
            .await?
            .into_iter()
            .find(|socket| socket.input_socket_id == input_socket_id);

        Ok(input_socket)
    }

    /// Given a [`ComponentId`] and [`InputSocketId`] find the [`ComponentInputSocket`]
    /// return an error if one is not found
    pub async fn get_by_ids_or_error(
        ctx: &DalContext,
        component_id: ComponentId,
        input_socket_id: InputSocketId,
    ) -> ComponentResult<ComponentInputSocket> {
        match Self::get_by_ids(ctx, component_id, input_socket_id).await? {
            Some(component_input_socket) => Ok(component_input_socket),
            None => Err(ComponentError::InputSocketNotFoundForComponentId(
                input_socket_id,
                component_id,
            )),
        }
    }

    /// List all [`AttributeValueId`]s for the given [`ComponentId`]s [`crate::InputSocket`]s
    pub async fn attribute_values_for_component_id(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<Vec<AttributeValueId>> {
        let mut result = HashMap::new();

        let socket_values = Component::attribute_values_for_all_sockets(ctx, component_id).await?;

        for socket_value_id in socket_values {
            if let Some(input_socket_id) = AttributeValue::is_for(ctx, socket_value_id)
                .await?
                .input_socket_id()
            {
                match result.entry(input_socket_id) {
                    hash_map::Entry::Vacant(entry) => {
                        entry.insert(ComponentInputSocket {
                            component_id,
                            attribute_value_id: socket_value_id,
                            input_socket_id,
                        });
                    }
                    hash_map::Entry::Occupied(_) => {
                        return Err(ComponentError::InputSocketTooManyAttributeValues(
                            input_socket_id,
                        ));
                    }
                }
            }
        }

        Ok(result
            .into_values()
            .map(|input_socket| input_socket.attribute_value_id)
            .collect_vec())
    }

    pub async fn value_for_input_socket_id_for_component_id_opt(
        ctx: &DalContext,
        component_id: ComponentId,
        input_socket_id: InputSocketId,
    ) -> ComponentResult<Option<serde_json::Value>> {
        let attribute_value_id = Self::get_by_ids_or_error(ctx, component_id, input_socket_id)
            .await?
            .attribute_value_id;

        let view = AttributeValue::view(ctx, attribute_value_id).await?;

        Ok(view)
    }
}
