use std::collections::{
    HashMap,
    hash_map,
};

use itertools::Itertools;
use serde::{
    Deserialize,
    Serialize,
};
use telemetry::prelude::*;

use super::{
    ComponentError,
    ComponentResult,
};
use crate::{
    AttributePrototype,
    AttributeValue,
    AttributeValueId,
    Component,
    ComponentId,
    DalContext,
    InputSocketId,
    OutputSocket,
    OutputSocketId,
    attribute::{
        prototype::argument::{
            AttributePrototypeArgument,
            value_source::ValueSource,
        },
        value::ValueIsFor,
    },
    workspace_snapshot::node_weight::ArgumentTargets,
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
    /// Find all inferred [`ComponentInputSocket`]s that are pulling data from the provided
    /// [`AttributeValueId`] that represents an [`crate::OutputSocket`] for a specific [`Component`]
    ///
    /// Output sockets can drive Input Sockets through inference based on the following logic:
    ///
    /// Components, Down Frames, and Up Frames can drive Input Sockets of their parents if the parent is an
    /// Up Frame.
    ///
    /// Down Frames can drive Input Sockets of their children if the child is a Down Frame
    /// or a Component or an Up Frame.
    #[instrument(level = "debug", name="component.component_output_socket.find_inferred_connections" skip(ctx))]
    pub async fn find_inferred_connections(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> ComponentResult<Vec<ComponentInputSocket>> {
        // let's make sure this av is actually for an output socket
        let value_is_for = AttributeValue::is_for(ctx, attribute_value_id).await?;
        let output_socket_id = match value_is_for {
            ValueIsFor::Prop(_) | ValueIsFor::InputSocket(_) => {
                return Err(ComponentError::WrongAttributeValueType(
                    attribute_value_id,
                    value_is_for,
                ));
            }
            ValueIsFor::OutputSocket(sock) => sock,
        };
        let component_id = AttributeValue::component_id(ctx, attribute_value_id).await?;

        let workspace_snapshot = ctx.workspace_snapshot()?;
        let mut inferred_connection_graph =
            workspace_snapshot.inferred_connection_graph(ctx).await?;
        let mut connections = inferred_connection_graph
            .inferred_connections_for_component_stack(ctx, component_id)
            .await?;
        connections.retain(|inferred_connection| {
            inferred_connection.source_component_id == component_id
                && inferred_connection.output_socket_id == output_socket_id
        });
        let mut input_sockets = Vec::new();
        for connection in connections {
            if let Some(input_socket) = ComponentInputSocket::get_by_ids(
                ctx,
                connection.destination_component_id,
                connection.input_socket_id,
            )
            .await?
            {
                input_sockets.push(input_socket);
            }
        }

        // sort by component id for consistent ordering
        input_sockets.sort_by_key(|input| input.component_id);
        Ok(input_sockets)
    }

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
    /// Find all inferred [`ComponentOutputSocket`]s for the provided
    /// [`ComponentInputSocket`] is pulling data from.
    ///
    /// [`crate::InputSocket`]s pull data through inference based on the following logic:
    ///
    /// Components and Down Frames find the closest [`crate::OutputSocket`] in their
    /// ancestors they can connect to
    ///
    /// Depending on the [`crate::SocketArity`], Up Frames can connect to ancestors AND descendants.
    /// If there is ever ambiguity about which [`crate::InputSocket`] they should connect to, we default
    /// to none, forcing the user to explicity configure a connection by drawing an Edge
    #[instrument(level = "debug", name="component.component_output_socket.find_inferred_connections" skip(ctx))]
    pub async fn find_inferred_connections(
        &self,
        ctx: &DalContext,
    ) -> ComponentResult<Vec<ComponentOutputSocket>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let mut inferred_connection_graph =
            workspace_snapshot.inferred_connection_graph(ctx).await?;
        let mut connections = Vec::new();
        for inferred_connection in inferred_connection_graph
            .inferred_connections_for_input_socket(ctx, self.component_id, self.input_socket_id)
            .await?
        {
            let attribute_value_id = OutputSocket::component_attribute_value_id(
                ctx,
                inferred_connection.output_socket_id,
                inferred_connection.source_component_id,
            )
            .await?;
            connections.push(ComponentOutputSocket {
                component_id: inferred_connection.source_component_id,
                output_socket_id: inferred_connection.output_socket_id,
                attribute_value_id,
            });
        }

        connections.sort_by_key(|output| output.component_id);

        Ok(connections)
    }

    pub async fn connections(
        &self,
        ctx: &DalContext,
    ) -> ComponentResult<Vec<(ComponentId, OutputSocketId, AttributePrototypeArgument)>> {
        let mut result = vec![];

        let prototype_id = AttributeValue::prototype_id(ctx, self.attribute_value_id).await?;
        for apa_id in AttributePrototypeArgument::list_ids_for_prototype_and_destination(
            ctx,
            prototype_id,
            self.component_id,
        )
        .await?
        {
            let apa = AttributePrototypeArgument::get_by_id(ctx, apa_id).await?;

            if let Some(ArgumentTargets {
                source_component_id,
                ..
            }) = apa.targets()
            {
                if let Some(ValueSource::OutputSocket(from_output_socket_id)) =
                    AttributePrototypeArgument::value_source_opt(ctx, apa_id).await?
                {
                    result.push((source_component_id, from_output_socket_id, apa));
                }
            }
        }

        Ok(result)
    }

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

    /// Finds the source [`Component`] of any [`crate::ComponentType`] for a given [`ComponentInputSocket`] where the
    /// [`crate::InputSocket`] has [`crate::SocketArity::One`]
    #[instrument(
        name = "component.component_input_socket.find_connection_arity_one",
        level = "debug",
        skip_all
    )]
    pub async fn find_connection_arity_one(
        self,
        ctx: &DalContext,
    ) -> ComponentResult<Option<ComponentId>> {
        let maybe_explicit_connection_source = {
            let explicit_connections =
                Component::incoming_connections_for_id(ctx, self.component_id).await?;
            let filtered_explicit_connection_sources: Vec<ComponentId> = explicit_connections
                .iter()
                .filter(|c| c.to_input_socket_id == self.input_socket_id)
                .map(|c| c.from_component_id)
                .collect();
            if filtered_explicit_connection_sources.len() > 1 {
                return Err(ComponentError::TooManyExplicitConnectionSources(
                    filtered_explicit_connection_sources,
                    self.component_id,
                    self.input_socket_id,
                ));
            }
            filtered_explicit_connection_sources.first().copied()
        };
        let maybe_inferred_connection_source = {
            let inferred_connections = match self.find_inferred_connections(ctx).await {
                Ok(inferred_connections) => inferred_connections,
                Err(ComponentError::ComponentMissingTypeValueMaterializedView(_)) => {
                    debug!(
                        ?self,
                        "component type not yet set when finding available inferred connections to input socket"
                    );
                    Vec::new()
                }
                Err(other_err) => Err(other_err)?,
            };
            if inferred_connections.len() > 1 {
                return Err(ComponentError::TooManyInferredConnections(
                    inferred_connections,
                    self,
                ));
            }
            inferred_connections.first().map(|c| c.component_id)
        };

        match (
            maybe_explicit_connection_source,
            maybe_inferred_connection_source,
        ) {
            (Some(explicit_source), Some(inferred_source)) => {
                Err(ComponentError::UnexpectedExplicitAndInferredSources(
                    explicit_source,
                    inferred_source,
                    self,
                ))
            }
            (Some(explicit_source), None) => Ok(Some(explicit_source)),
            (None, Some(inferred_source)) => Ok(Some(inferred_source)),
            (None, None) => Ok(None),
        }
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

    /// Return true if the input socket already has an explicit connection (a user drew an edge)
    #[instrument(level = "debug", skip(ctx))]
    pub async fn is_manually_configured(&self, ctx: &DalContext) -> ComponentResult<bool> {
        // if the input socket has an explicit connection, then we will not gather any implicit
        // note we could do some weird logic here when it comes to sockets with arrity of many
        // but let's punt for now
        if let Some(maybe_attribute_prototype) =
            AttributePrototype::find_for_input_socket(ctx, self.input_socket_id).await?
        {
            // if this socket has an attribute prototype argument,
            //that means it has an explicit connection and we should not
            // look for implicits
            let maybe_apa = AttributePrototypeArgument::list_ids_for_prototype_and_destination(
                ctx,
                maybe_attribute_prototype,
                self.component_id,
            )
            .await?;
            if !maybe_apa.is_empty() {
                return Ok(true);
            }
        }
        Ok(false)
    }
}
