use std::collections::{hash_map, HashMap, HashSet};

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use si_id::{ulid::Ulid, ViewId};
use telemetry::prelude::*;

use crate::{
    attribute::{
        prototype::argument::{value_source::ValueSource, AttributePrototypeArgument},
        value::ValueIsFor,
    },
    diagram::view::View,
    workspace_snapshot::{
        edge_weight::EdgeWeightKindDiscriminants,
        node_weight::{category_node_weight::CategoryNodeKind, ArgumentTargets},
        InputSocketExt,
    },
    AttributePrototype, AttributeValue, AttributeValueId, Component, ComponentId, ComponentType,
    DalContext, EdgeWeight, EdgeWeightKind, InputSocket, InputSocketId, OutputSocket,
    OutputSocketId, SocketArity,
};

use super::{ComponentError, ComponentResult};

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
                ))
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
        ctx: &DalContext,
        component_input_socket: ComponentInputSocket,
    ) -> ComponentResult<Vec<ComponentOutputSocket>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let mut inferred_connection_graph =
            workspace_snapshot.inferred_connection_graph(ctx).await?;
        let mut connections = Vec::new();
        for inferred_connection in inferred_connection_graph
            .inferred_connections_for_input_socket(
                ctx,
                component_input_socket.component_id,
                component_input_socket.input_socket_id,
            )
            .await?
        {
            let attribute_value_id = OutputSocket::component_attribute_value_for_output_socket_id(
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
                    apa.value_source(ctx).await?
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
        ctx: &DalContext,
        component_input_socket: ComponentInputSocket,
    ) -> ComponentResult<Option<ComponentId>> {
        let maybe_explicit_connection_source = {
            let explicit_connections =
                Component::incoming_connections_for_id(ctx, component_input_socket.component_id)
                    .await?;
            let filtered_explicit_connection_sources: Vec<ComponentId> = explicit_connections
                .iter()
                .filter(|c| c.to_input_socket_id == component_input_socket.input_socket_id)
                .map(|c| c.from_component_id)
                .collect();
            if filtered_explicit_connection_sources.len() > 1 {
                return Err(ComponentError::TooManyExplicitConnectionSources(
                    filtered_explicit_connection_sources,
                    component_input_socket.component_id,
                    component_input_socket.input_socket_id,
                ));
            }
            filtered_explicit_connection_sources.first().copied()
        };
        let maybe_inferred_connection_source = {
            let inferred_connections = match Self::find_inferred_connections(
                ctx,
                component_input_socket,
            )
            .await
            {
                Ok(inferred_connections) => inferred_connections,
                Err(ComponentError::ComponentMissingTypeValueMaterializedView(_)) => {
                    debug!(?component_input_socket, "component type not yet set when finding available inferred connections to input socket");
                    Vec::new()
                }
                Err(other_err) => Err(other_err)?,
            };
            if inferred_connections.len() > 1 {
                return Err(ComponentError::TooManyInferredConnections(
                    inferred_connections,
                    component_input_socket,
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
                    component_input_socket,
                ))
            }
            (Some(explicit_source), None) => Ok(Some(explicit_source)),
            (None, Some(inferred_source)) => Ok(Some(inferred_source)),
            (None, None) => Ok(None),
        }
    }

    /// Return true if the input socket already has an explicit connection (a user drew an edge)
    #[instrument(level = "debug", skip(ctx))]
    pub async fn is_manually_configured(
        ctx: &DalContext,
        component_input_socket: ComponentInputSocket,
    ) -> ComponentResult<bool> {
        // if the input socket has an explicit connection, then we will not gather any implicit
        // note we could do some weird logic here when it comes to sockets with arrity of many
        // but let's punt for now
        if let Some(maybe_attribute_prototype) =
            AttributePrototype::find_for_input_socket(ctx, component_input_socket.input_socket_id)
                .await?
        {
            // if this socket has an attribute prototype argument,
            //that means it has an explicit connection and we should not
            // look for implicits
            let maybe_apa = AttributePrototypeArgument::list_ids_for_prototype_and_destination(
                ctx,
                maybe_attribute_prototype,
                component_input_socket.component_id,
            )
            .await?;
            if !maybe_apa.is_empty() {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

#[derive(Eq, Hash, PartialEq, Clone, Debug, Copy, Serialize, Deserialize)]
pub enum ComponentSocket {
    Input(ComponentInputSocket),
    Output(ComponentOutputSocket),
}

impl ComponentSocket {
    pub fn get_attribute_value(&self) -> AttributeValueId {
        match self {
            ComponentSocket::Input(component_input_socket) => {
                component_input_socket.attribute_value_id
            }
            ComponentSocket::Output(component_output_socket) => {
                component_output_socket.attribute_value_id
            }
        }
    }
    pub fn get_component_id(&self) -> ComponentId {
        match self {
            ComponentSocket::Input(component_input_socket) => component_input_socket.component_id,
            ComponentSocket::Output(component_output_socket) => {
                component_output_socket.component_id
            }
        }
    }
    pub fn get_socket_id(&self) -> Ulid {
        match self {
            ComponentSocket::Input(component_input_socket) => {
                component_input_socket.input_socket_id.into()
            }
            ComponentSocket::Output(component_output_socket) => {
                component_output_socket.output_socket_id.into()
            }
        }
    }
    pub async fn assemble(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> ComponentResult<Self> {
        let snap = ctx.workspace_snapshot()?;

        if let Some(input_socket_id) = snap
            .input_socket_id_find_for_attribute_value_id(attribute_value_id)
            .await?
        {
            let component_id = AttributeValue::component_id(ctx, attribute_value_id).await?;
            return Ok(ComponentSocket::Input(ComponentInputSocket {
                component_id,
                input_socket_id,
                attribute_value_id,
            }));
        } else if let Some(output_socket_id) =
            OutputSocket::find_for_attribute_value_id(ctx, attribute_value_id).await?
        {
            let component_id = AttributeValue::component_id(ctx, attribute_value_id).await?;
            return Ok(ComponentSocket::Output(ComponentOutputSocket {
                component_id,
                output_socket_id,
                attribute_value_id,
            }));
        }
        Err(ComponentError::CannotCloneFromDifferentVariants)
    }
}

#[derive(Eq, Hash, PartialEq, Clone, Debug, Copy, Serialize, Deserialize)]
pub enum DefaultConnection {
    Workspace(ComponentSocket),
    View(ComponentSocket),
    Frame(ComponentSocket),
}
#[derive(Eq, Hash, PartialEq, Clone, Debug, Copy, Serialize, Deserialize)]
struct DefaultConnectionWithPriority {
    connection: DefaultConnection,
    distance: u32,
}
impl DefaultConnection {
    pub async fn remove_for_attribute_value(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> ComponentResult<()> {
        let snap = ctx.workspace_snapshot()?;
        let nodes_to_disconnect_from = snap
            .outgoing_targets_for_edge_weight_kind(
                attribute_value_id,
                EdgeWeightKindDiscriminants::DefaultConnection,
            )
            .await?;
        for node_to_disconnect in nodes_to_disconnect_from {
            let node_id = snap.get_node_weight(node_to_disconnect).await?.id();
            snap.remove_edge_for_ulids(
                node_id,
                attribute_value_id,
                EdgeWeightKindDiscriminants::DefaultConnection,
            )
            .await?;
        }
        Ok(())
    }

    pub async fn remove_for_frame(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> ComponentResult<()> {
        let snap = ctx.workspace_snapshot()?;
        let component_id = AttributeValue::component_id(ctx, attribute_value_id).await?;

        snap.remove_edge_for_ulids(
            component_id,
            attribute_value_id,
            EdgeWeightKindDiscriminants::DefaultConnection,
        )
        .await?;
        Ok(())
    }
    pub async fn remove_for_workspace(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> ComponentResult<()> {
        let snap = ctx.workspace_snapshot()?;

        if let Some(category_node) = snap
            .get_category_node(None, CategoryNodeKind::Component)
            .await?
        {
            snap.remove_edge_for_ulids(
                category_node,
                attribute_value_id,
                EdgeWeightKindDiscriminants::DefaultConnection,
            )
            .await?;
        }
        Ok(())
    }
    pub async fn remove_for_view(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
        view_id: ViewId,
    ) -> ComponentResult<()> {
        let snap = ctx.workspace_snapshot()?;
        let view_node = snap.get_node_weight_by_id(view_id).await?;

        snap.remove_edge_for_ulids(
            view_node.id(),
            attribute_value_id,
            EdgeWeightKindDiscriminants::DefaultConnection,
        )
        .await?;

        Ok(())
    }

    pub async fn remove_default_connections_for_component(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<()> {
        let mut default_connection_to_check = Vec::new();
        // get workspace default things
        default_connection_to_check
            .extend(DefaultConnection::get_default_connections_for_workspace(ctx).await?);
        let views_for_component = View::list_for_component_id(ctx, component_id)
            .await
            .map_err(Box::new)?;
        for view_id in views_for_component {
            default_connection_to_check
                .extend(DefaultConnection::get_default_connections_for_view(ctx, view_id).await?);
        }

        for conn in default_connection_to_check {
            let component_socket = conn.get_default();
            if component_socket.get_component_id() == component_id {
                Self::remove_for_attribute_value(ctx, component_socket.get_attribute_value())
                    .await?;
            }
        }
        Ok(())
    }

    pub async fn get_configured_defaults_for_component(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<Vec<DefaultConnection>> {
        let snap = ctx.workspace_snapshot()?;
        let mut default_connections = Vec::new();
        for av_idx in snap
            .outgoing_targets_for_edge_weight_kind(
                component_id,
                EdgeWeightKind::DefaultConnection.into(),
            )
            .await?
        {
            let attr_value_weight = snap
                .get_node_weight(av_idx)
                .await?
                .get_attribute_value_node_weight()?;
            let attribute_value_id: AttributeValueId = attr_value_weight.id().into();
            if let Some(input_socket_id) = snap
                .input_socket_id_find_for_attribute_value_id(attribute_value_id)
                .await?
            {
                let component_id = AttributeValue::component_id(ctx, attribute_value_id).await?;
                let entry =
                    DefaultConnection::Frame(ComponentSocket::Input(ComponentInputSocket {
                        component_id,
                        input_socket_id,
                        attribute_value_id,
                    }));
                default_connections.push(entry);
            } else if let Some(output_socket_id) =
                OutputSocket::find_for_attribute_value_id(ctx, attribute_value_id).await?
            {
                let component_id = AttributeValue::component_id(ctx, attribute_value_id).await?;
                let entry =
                    DefaultConnection::Frame(ComponentSocket::Output(ComponentOutputSocket {
                        component_id,
                        output_socket_id,
                        attribute_value_id,
                    }));
                default_connections.push(entry);
            }
        }
        Ok(default_connections)
    }

    pub async fn get_default_connections_for_frame(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<Vec<DefaultConnection>> {
        // if component_id passed in is a frame, check direct children for default connections
        // if it's not, return none
        let snap = ctx.workspace_snapshot()?;
        let mut default_connections = Vec::new();
        // get direct descendants of the frame, and see if any of them have a policy set
        for av_idx in snap
            .outgoing_targets_for_edge_weight_kind(
                component_id,
                EdgeWeightKind::DefaultConnection.into(),
            )
            .await?
        {
            let attr_value_weight = snap
                .get_node_weight(av_idx)
                .await?
                .get_attribute_value_node_weight()?;
            let attribute_value_id: AttributeValueId = attr_value_weight.id().into();
            if let Some(input_socket_id) = snap
                .input_socket_id_find_for_attribute_value_id(attribute_value_id)
                .await?
            {
                let component_id = AttributeValue::component_id(ctx, attribute_value_id).await?;
                let entry =
                    DefaultConnection::Frame(ComponentSocket::Input(ComponentInputSocket {
                        component_id,
                        input_socket_id,
                        attribute_value_id,
                    }));
                default_connections.push(entry);
            } else if let Some(output_socket_id) =
                OutputSocket::find_for_attribute_value_id(ctx, attribute_value_id).await?
            {
                let component_id = AttributeValue::component_id(ctx, attribute_value_id).await?;
                let entry =
                    DefaultConnection::Frame(ComponentSocket::Output(ComponentOutputSocket {
                        component_id,
                        output_socket_id,
                        attribute_value_id,
                    }));
                default_connections.push(entry);
            }
        }
        // check children, but don't allow duplicates (i.e. if I am a frame, my defaultConnection takes precedence)
        let children = Component::get_children_for_id(ctx, component_id).await?;
        for child in children {
            // only check children that are NOT frames
            if let ComponentType::Component = Component::get_type_by_id(ctx, child).await? {
                for av_idx in snap
                    .outgoing_targets_for_edge_weight_kind(
                        child,
                        EdgeWeightKind::DefaultConnection.into(),
                    )
                    .await?
                {
                    let attr_value_weight = snap
                        .get_node_weight(av_idx)
                        .await?
                        .get_attribute_value_node_weight()?;
                    let attribute_value_id: AttributeValueId = attr_value_weight.id().into();
                    if let Some(input_socket_id) = snap
                        .input_socket_id_find_for_attribute_value_id(attribute_value_id)
                        .await?
                    {
                        let component_id =
                            AttributeValue::component_id(ctx, attribute_value_id).await?;
                        let entry = DefaultConnection::Frame(ComponentSocket::Input(
                            ComponentInputSocket {
                                component_id,
                                input_socket_id,
                                attribute_value_id,
                            },
                        ));
                        default_connections.push(entry);
                    } else if let Some(output_socket_id) =
                        OutputSocket::find_for_attribute_value_id(ctx, attribute_value_id).await?
                    {
                        let component_id =
                            AttributeValue::component_id(ctx, attribute_value_id).await?;
                        let entry = DefaultConnection::Frame(ComponentSocket::Output(
                            ComponentOutputSocket {
                                component_id,
                                output_socket_id,
                                attribute_value_id,
                            },
                        ));
                        default_connections.push(entry);
                    }
                }
            }
        }
        Ok(default_connections)
    }

    pub async fn get_default_connections_for_workspace(
        ctx: &DalContext,
    ) -> ComponentResult<Vec<DefaultConnection>> {
        let snap = ctx.workspace_snapshot()?;

        let mut default_connections = Vec::new();

        if let Some(category_node) = snap
            .get_category_node(None, CategoryNodeKind::Component)
            .await?
        {
            for av_idx in snap
                .outgoing_targets_for_edge_weight_kind(
                    category_node,
                    EdgeWeightKind::DefaultConnection.into(),
                )
                .await?
            {
                let attr_value_weight = snap
                    .get_node_weight(av_idx)
                    .await?
                    .get_attribute_value_node_weight()?;
                let attribute_value_id: AttributeValueId = attr_value_weight.id().into();
                if let Some(input_socket_id) = snap
                    .input_socket_id_find_for_attribute_value_id(attribute_value_id)
                    .await?
                {
                    let component_id =
                        AttributeValue::component_id(ctx, attribute_value_id).await?;
                    let entry = DefaultConnection::Workspace(ComponentSocket::Input(
                        ComponentInputSocket {
                            component_id,
                            input_socket_id,
                            attribute_value_id,
                        },
                    ));
                    default_connections.push(entry);
                } else if let Some(output_socket_id) =
                    OutputSocket::find_for_attribute_value_id(ctx, attribute_value_id).await?
                {
                    let component_id =
                        AttributeValue::component_id(ctx, attribute_value_id).await?;
                    let entry = DefaultConnection::Workspace(ComponentSocket::Output(
                        ComponentOutputSocket {
                            component_id,
                            output_socket_id,
                            attribute_value_id,
                        },
                    ));
                    default_connections.push(entry);
                }
            }
        }
        Ok(default_connections)
    }

    pub async fn set_default_connection_for_frame(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> ComponentResult<Option<DefaultConnection>> {
        // default connections for a frame have an edge from the attribute value to the component that contains that
        // attribute value
        let snap = ctx.workspace_snapshot()?;
        let component_id = AttributeValue::component_id(ctx, attribute_value_id).await?;
        snap.add_edge(
            component_id,
            EdgeWeight::new(EdgeWeightKind::DefaultConnection),
            attribute_value_id,
        )
        .await?;
        let component_socket = ComponentSocket::assemble(ctx, attribute_value_id).await?;
        Ok(Some(DefaultConnection::Frame(component_socket)))
    }

    pub async fn set_default_connection_for_workspace(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> ComponentResult<Option<DefaultConnection>> {
        // default connections for workspace have an edge directly from the Component category node to the
        let snap = ctx.workspace_snapshot()?;
        // starting with output sockets:
        // draw an edge from the component category node to the attribute value of the socket
        let category_node = snap
            .get_category_node(None, CategoryNodeKind::Component)
            .await?
            .ok_or(ComponentError::CannotCloneFromDifferentVariants)?;
        // for input sockets, when this exists, we need to components to look at possible output
        // sockets when they're created to look for matches

        // output sockets are similar to down frames, in that by setting this youare
        // triggering components to check their input sockets for matches when they're created
        // looking first at anyone in it's line of parentage, then anyone in the view, then anyone
        // in the workspace, depending on the level

        snap.add_edge(
            category_node,
            EdgeWeight::new(EdgeWeightKind::DefaultConnection),
            attribute_value_id,
        )
        .await?;
        let component_socket = ComponentSocket::assemble(ctx, attribute_value_id).await?;
        Ok(Some(DefaultConnection::Workspace(component_socket)))
    }

    pub async fn set_default_connection_for_view(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
        view_id: ViewId,
    ) -> ComponentResult<Option<DefaultConnection>> {
        // default connections for workspace have an edge directly from the Component category node to the
        let snap = ctx.workspace_snapshot()?;

        // draw an edge from the view node to the attribute value of the socket

        // for input sockets, when this exists, we need to components to look at possible output
        // sockets when they're created to look for matches

        // output sockets are similar to down frames, in that by setting this youare
        // triggering components to check their input sockets for matches when they're created
        // looking first at anyone in it's line of parentage, then anyone in the view, then anyone
        // in the workspace, depending on the level

        snap.add_edge(
            view_id,
            EdgeWeight::new(EdgeWeightKind::DefaultConnection),
            attribute_value_id,
        )
        .await?;
        let component_socket = ComponentSocket::assemble(ctx, attribute_value_id).await?;
        Ok(Some(DefaultConnection::View(component_socket)))
    }
    pub async fn get_default_connections_for_view(
        ctx: &DalContext,
        view_id: ViewId,
    ) -> ComponentResult<Vec<DefaultConnection>> {
        let snap = ctx.workspace_snapshot()?;

        let mut default_connections = Vec::new();

        for av_idx in snap
            .outgoing_targets_for_edge_weight_kind(
                view_id,
                EdgeWeightKind::DefaultConnection.into(),
            )
            .await?
        {
            let attr_value_weight = snap
                .get_node_weight(av_idx)
                .await?
                .get_attribute_value_node_weight()?;
            let attribute_value_id: AttributeValueId = attr_value_weight.id().into();
            if let Some(input_socket_id) = snap
                .input_socket_id_find_for_attribute_value_id(attribute_value_id)
                .await?
            {
                let component_id = AttributeValue::component_id(ctx, attribute_value_id).await?;
                let entry = DefaultConnection::View(ComponentSocket::Input(ComponentInputSocket {
                    component_id,
                    input_socket_id,
                    attribute_value_id,
                }));
                default_connections.push(entry);
            } else if let Some(output_socket_id) =
                OutputSocket::find_for_attribute_value_id(ctx, attribute_value_id).await?
            {
                let component_id = AttributeValue::component_id(ctx, attribute_value_id).await?;
                let entry =
                    DefaultConnection::View(ComponentSocket::Output(ComponentOutputSocket {
                        component_id,
                        output_socket_id,
                        attribute_value_id,
                    }));
                default_connections.push(entry);
            }
        }
        Ok(default_connections)
    }

    pub async fn get_default_connections_for_component(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<HashMap<ComponentSocket, Vec<DefaultConnection>>> {
        let mut default_connections_for_component = HashMap::new();
        let mut default_connection_to_check = Vec::new();
        // get workspace default things
        default_connection_to_check.extend(
            DefaultConnection::get_default_connections_for_workspace(ctx)
                .await?
                .iter()
                .map(|conn| DefaultConnectionWithPriority {
                    connection: *conn,
                    distance: 0, // 0 for workspace
                }),
        );
        // get view default things
        let views_for_component = View::list_for_component_id(ctx, component_id)
            .await
            .map_err(Box::new)?;
        for view_id in views_for_component {
            default_connection_to_check.extend(
                DefaultConnection::get_default_connections_for_view(ctx, view_id)
                    .await?
                    .iter()
                    .map(|conn| DefaultConnectionWithPriority {
                        connection: *conn,
                        distance: 1, // 1 for view
                    }),
            );
        }
        // get frame default things
        // for a component, check it's parent and lineage for default connections on any of them
        // track the distance, the bigger the number, the closer it is
        let base_frame_distance = 2;
        let max_frame_depth = 100; // reasonable limit for nesting
        let mut current_component = component_id;
        let mut depth = 0;

        while let Some(parent) = Component::get_parent_by_id(ctx, current_component).await? {
            let distance = max_frame_depth - depth + base_frame_distance;
            let parent_defaults =
                DefaultConnection::get_default_connections_for_frame(ctx, parent).await?;
            default_connection_to_check.extend(parent_defaults.into_iter().map(|conn| {
                DefaultConnectionWithPriority {
                    connection: conn,
                    distance,
                }
            }));
            current_component = parent;
            depth += 1;
        }

        // build up a map of potential matches.  We consider a match valid if the sockets can actually connect
        // and the value for the attribute value matches the output socket's value
        // OR for multi-arity sockets, we also check if the output socket matches a single entry in the array
        let mut potential_incoming_matches: HashMap<
            ComponentSocket,
            Vec<DefaultConnectionWithPriority>,
        > = HashMap::new();
        let mut potential_outgoing_matches: HashMap<
            ComponentSocket,
            Vec<DefaultConnectionWithPriority>,
        > = HashMap::new();
        let component = Component::get_by_id(ctx, component_id).await?;
        // get output sockets for component
        let component_output_sockets = component.output_socket_attribute_values(ctx).await?;
        let component_input_sockets = component.input_socket_attribute_values(ctx).await?;
        let output_sockets_to_check: Vec<DefaultConnectionWithPriority> =
            default_connection_to_check
                .iter()
                .filter_map(|default_conn| match default_conn.connection {
                    DefaultConnection::Workspace(component_socket) => match component_socket {
                        ComponentSocket::Input(_) => None,
                        ComponentSocket::Output(_) => Some(*default_conn),
                    },
                    DefaultConnection::View(component_socket) => match component_socket {
                        ComponentSocket::Input(_) => None,
                        ComponentSocket::Output(_) => Some(*default_conn),
                    },
                    DefaultConnection::Frame(component_socket) => match component_socket {
                        ComponentSocket::Input(_) => None,
                        ComponentSocket::Output(_) => Some(*default_conn),
                    },
                })
                .collect_vec();

        let input_sockets_to_check: Vec<DefaultConnectionWithPriority> =
            default_connection_to_check
                .iter()
                .filter_map(|default_conn| match default_conn.connection {
                    DefaultConnection::Workspace(component_socket) => match component_socket {
                        ComponentSocket::Input(_) => Some(*default_conn),
                        ComponentSocket::Output(_) => None,
                    },
                    DefaultConnection::View(component_socket) => match component_socket {
                        ComponentSocket::Input(_) => Some(*default_conn),
                        ComponentSocket::Output(_) => None,
                    },
                    DefaultConnection::Frame(component_socket) => match component_socket {
                        ComponentSocket::Input(_) => Some(*default_conn),
                        ComponentSocket::Output(_) => None,
                    },
                })
                .collect_vec();

        // loop over the default output sockets, collect what might match one of the component's
        // input sockets - don't worry we'll de-dupe next
        for component_output_socket in output_sockets_to_check {
            let socket = component_output_socket.connection.get_default();
            if let ComponentSocket::Output(component_socket) = socket {
                let output_socket =
                    OutputSocket::get_by_id(ctx, component_socket.output_socket_id).await?;
                for component_input in component_input_sockets.clone() {
                    if let Some(input_socket_id) =
                        InputSocket::find_for_attribute_value_id(ctx, component_input).await?
                    {
                        let input_socket = InputSocket::get_by_id(ctx, input_socket_id).await?;
                        if output_socket.fits_input(&input_socket) {
                            // record it as a potential match
                            potential_incoming_matches
                                .entry(ComponentSocket::assemble(ctx, component_input).await?)
                                .or_default()
                                .push(component_output_socket);
                        }
                    }
                }
            }
        }
        // loop over the default input sockets, collect what might match one of the component's
        // input sockets - don't worry we'll de-dupe next
        for component_input_socket in input_sockets_to_check {
            let socket = component_input_socket.connection.get_default();
            if let ComponentSocket::Input(component_socket) = socket {
                let input_socket =
                    InputSocket::get_by_id(ctx, component_socket.input_socket_id).await?;
                for component_output in component_output_sockets.clone() {
                    if let Some(output_socket_id) =
                        OutputSocket::find_for_attribute_value_id(ctx, component_output).await?
                    {
                        let output_socket = OutputSocket::get_by_id(ctx, output_socket_id).await?;
                        if output_socket.fits_input(&input_socket) {
                            // record it as a potential match
                            potential_outgoing_matches
                                .entry(ComponentSocket::assemble(ctx, component_output).await?)
                                .or_default()
                                .push(component_input_socket);
                        }
                    }
                }
            }
        }
        // now we need to dedupe and prioritize!
        // loop over potential output sockets, and prevent one output socket from connecting to multiple input sockets
        // for the same component and multiple output sockets connecting to the same input socket for the same component
        let outgoing_connections = Self::process_outgoing_connections(potential_outgoing_matches);

        // now input sockets are more tricky. Loop over potential incoming connections and...
        // for a given input socket, if there's a view-scoped default connection use it!
        // if not, check for workspace-scoped default connections next.
        // If the socket is single arity, we will only draw an edge if there's exactly one match
        // for a given scope
        // If it's a multi-arity socket, we will only draw an edge for multiple inputs at the same scope
        // We also need to make sure that for the whole component, we don't have 2 incoming connections from
        // the same component across multiple input sockets
        let incoming_connections =
            Self::process_incoming_connections(ctx, potential_incoming_matches).await?;
        default_connections_for_component.extend(incoming_connections);
        default_connections_for_component.extend(outgoing_connections);
        Ok(default_connections_for_component)
    }

    async fn process_incoming_connections(
        ctx: &DalContext,
        potential_matches: HashMap<ComponentSocket, Vec<DefaultConnectionWithPriority>>,
    ) -> ComponentResult<HashMap<ComponentSocket, Vec<DefaultConnection>>> {
        let mut deduped_connections: HashMap<ComponentSocket, Vec<DefaultConnectionWithPriority>> =
            HashMap::new();
        let mut filtered_connections: HashMap<ComponentSocket, Vec<DefaultConnectionWithPriority>> =
            HashMap::new();

        // Track input sockets that have multiple output connections
        let mut output_socket_usage = HashMap::new();
        // Track components that have multiple input connections from same output
        let mut component_input_usage = HashMap::new();
        dbg!(&potential_matches);
        let should_use_view = false;
        for (input_id, mut connections) in potential_matches {
            // Sort by priority (highest first)
            connections.sort_by(|a, b| b.distance.cmp(&a.distance));

            if connections.is_empty() {
                continue;
            }

            // Get highest priority connections
            let highest_priority = connections[0].distance;
            let highest_priority_connections: Vec<_> = connections
                .into_iter()
                .take_while(|conn| conn.distance == highest_priority)
                .collect();

            if !highest_priority_connections.is_empty() {
                for conn in highest_priority_connections.clone() {
                    let component_socket = conn.connection.get_default();
                    let component_id = component_socket.get_component_id();
                    let output_av = component_socket.get_attribute_value();

                    output_socket_usage
                        .entry(output_av)
                        .or_insert_with(HashSet::new)
                        .insert(input_id);

                    component_input_usage
                        .entry((component_id, input_id))
                        .or_insert_with(HashSet::new)
                        .insert(output_av);
                }
                filtered_connections.insert(input_id, highest_priority_connections);
            }
        }
        // first pass, identify conflicts + priorities
        // for (input_id, connections) in potential_matches.clone() {
        //     let only_view_connections = connections
        //         .iter()
        //         .filter(|conn| match conn.connection {
        //             DefaultConnection::Workspace(_) | DefaultConnection::Frame(_) => false,
        //             DefaultConnection::View(_) => true,
        //         })
        //         .collect_vec();
        //     let only_workspace_connections = connections
        //         .iter()
        //         .filter(|conn| match conn.connection {
        //             DefaultConnection::Workspace(_) => true,
        //             DefaultConnection::View(_) | DefaultConnection::Frame(_) => false,
        //         })
        //         .collect_vec();
        //     let only_frame_connections = connections
        //         .iter()
        //         .filter(|conn| match conn.connection {
        //             DefaultConnection::Workspace(_) | DefaultConnection::View(_) => false,
        //             DefaultConnection::Frame(_) => true,
        //         })
        //         .collect_vec();

        // if !only_frame_connections.is_empty() {
        //     // if there are frame connections, only use them
        // } else if !only_view_connections.is_empty() {
        //     // lets record these as maybes
        //     for conn in only_view_connections.clone() {
        //         let component_socket = conn.connection.get_default();
        //         let component_id = component_socket.get_component_id();
        //         let output_av = component_socket.get_attribute_value();

        //         output_socket_usage
        //             .entry(output_av)
        //             .or_insert_with(HashSet::new)
        //             .insert(input_id);

        //         component_input_usage
        //             .entry((component_id, input_id))
        //             .or_insert_with(HashSet::new)
        //             .insert(output_av);
        //     }
        //     filtered_connections.insert(
        //         input_id,
        //         only_view_connections.iter().map(|a| **a).collect_vec(),
        //     );
        // } else if !only_workspace_connections.is_empty() {
        //     // otherwise record these
        //     for conn in only_workspace_connections.clone() {
        //         let component_socket = conn.get_default();
        //         let component_id = component_socket.get_component_id();
        //         let output_av = component_socket.get_attribute_value();

        //         output_socket_usage
        //             .entry(output_av)
        //             .or_insert_with(HashSet::new)
        //             .insert(input_id);

        //         component_input_usage
        //             .entry((component_id, input_id))
        //             .or_insert_with(HashSet::new)
        //             .insert(output_av);
        //     }
        //     filtered_connections.insert(
        //         input_id,
        //         only_workspace_connections.iter().map(|a| **a).collect_vec(),
        //     );
        // }
        // }
        // Second pass - only keep non-conflicting connections
        for (input_id, connections) in filtered_connections {
            for conn in connections {
                let socket = conn.connection.get_default();
                let output_av = socket.get_attribute_value();
                let component_id = socket.get_component_id();

                if output_socket_usage
                    .get(&output_av)
                    .is_some_and(|s| s.len() == 1)
                    && component_input_usage
                        .get(&(component_id, input_id))
                        .is_some_and(|s| s.len() == 1)
                {
                    deduped_connections.entry(input_id).or_default().push(conn);
                }
            }
        }
        // final pass: if an input socket has arity one, remove it if it has multiple connections now
        // otherwise, keep all of them
        let mut final_connections: HashMap<ComponentSocket, Vec<DefaultConnection>> =
            HashMap::new();
        for (input_id, conn) in deduped_connections {
            let input_socket = InputSocket::get_by_id(ctx, input_id.get_socket_id().into()).await?;
            if input_socket.arity() == SocketArity::One && conn.len() == 1 {
                final_connections
                    .entry(input_id)
                    .or_default()
                    .push(conn[0].connection);
            } else if input_socket.arity() == SocketArity::Many {
                final_connections
                    .entry(input_id)
                    .or_default()
                    .extend(conn.iter().map(|c| c.connection).collect_vec());
            }
        }

        Ok(final_connections)
    }

    fn process_outgoing_connections(
        potential_matches: HashMap<ComponentSocket, Vec<DefaultConnectionWithPriority>>,
    ) -> HashMap<ComponentSocket, Vec<DefaultConnection>> {
        let mut final_connections = HashMap::new();

        // Track input sockets that have multiple output connections
        let mut input_socket_usage = HashMap::new();
        // Track components that have multiple input connections from same output
        let mut component_output_usage = HashMap::new();

        // First pass - identify conflicts
        for (output_id, connections) in &potential_matches {
            for conn in connections {
                let component_socket = conn.connection.get_default();
                let component_id = component_socket.get_component_id();
                let input_av = component_socket.get_attribute_value();

                input_socket_usage
                    .entry(input_av)
                    .or_insert_with(HashSet::new)
                    .insert(*output_id);

                component_output_usage
                    .entry((component_id, *output_id))
                    .or_insert_with(HashSet::new)
                    .insert(input_av);
            }
        }

        // Second pass - only keep non-conflicting connections
        for (output_id, connections) in potential_matches {
            for conn in connections {
                let socket = conn.connection.get_default();
                let input_av = socket.get_attribute_value();
                let component_id = socket.get_component_id();

                if input_socket_usage
                    .get(&input_av)
                    .is_some_and(|s| s.len() == 1)
                    && component_output_usage
                        .get(&(component_id, output_id))
                        .is_some_and(|s| s.len() == 1)
                {
                    final_connections
                        .entry(output_id)
                        .or_insert_with(Vec::new)
                        .push(conn.connection);
                }
            }
        }

        final_connections
    }

    fn get_default(&self) -> ComponentSocket {
        match self {
            DefaultConnection::Workspace(component_socket)
            | DefaultConnection::View(component_socket)
            | DefaultConnection::Frame(component_socket) => *component_socket,
        }
    }
}
