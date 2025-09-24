use std::collections::HashSet;

use itertools::Itertools;
use telemetry::prelude::*;
use thiserror::Error;

use super::{
    inferred_connection_graph::InferredConnectionGraphError,
    socket::{
        ComponentInputSocket,
        ComponentOutputSocket,
    },
};
use crate::{
    Component,
    ComponentError,
    ComponentId,
    ComponentType,
    DalContext,
    InputSocket,
    OutputSocket,
    TransactionsError,
    WsEvent,
    WsEventError,
    attribute::value::AttributeValueError,
    component::inferred_connection_graph::InferredConnection,
    diagram::SummaryDiagramInferredEdge,
    socket::{
        input::InputSocketError,
        output::OutputSocketError,
    },
    workspace_snapshot::{
        WorkspaceSnapshotError,
        dependent_value_root::DependentValueRootError,
    },
};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum FrameError {
    #[error("aggregation frames unsupported: {0}")]
    AggregateFramesUnsupported(ComponentId),
    #[error("attribute value error: {0}")]
    AttributeValueError(#[from] AttributeValueError),
    #[error("component error: {0}")]
    Component(#[from] ComponentError),
    #[error("dependent value root error: {0}")]
    DependentValueRoot(#[from] DependentValueRootError),
    #[error("InferredConnectionGraph error: {0}")]
    InferredConnectionGraph(#[from] InferredConnectionGraphError),
    #[error("input socket error: {0}")]
    InputSocketError(#[from] InputSocketError),
    #[error("OutputSocket error: {0}")]
    OutputSocket(#[from] OutputSocketError),
    #[error("parent is not a frame (child id: {0}) (parent id: {1})")]
    ParentIsNotAFrame(ComponentId, ComponentId),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
    #[error("WsEvent error: {0}")]
    WsEvent(#[from] WsEventError),
}

#[derive(Debug, Eq, Hash, PartialEq, Copy, Clone)]
struct SocketAttributeValuePair {
    component_input_socket: ComponentInputSocket,
    component_output_socket: ComponentOutputSocket,
}

pub type FrameResult<T> = Result<T, FrameError>;

/// A unit struct containing logic for working with frames.
pub struct Frame;

#[derive(Debug)]
pub struct InferredEdgeChanges {
    pub removed_edges: Vec<SummaryDiagramInferredEdge>,
    pub upserted_edges: Vec<SummaryDiagramInferredEdge>,
}

impl Frame {
    /// Given a [`ComponentId`] and either the parent or a child of it,
    /// calculate what needs to be updated given the change in [`ComponentType`]
    /// and enqueue those [`AttributeValue`]s to be updated
    pub async fn update_type_from_or_to_frame(
        ctx: &DalContext,
        component_id: ComponentId,
        reference_id: ComponentId,
        new_type: ComponentType,
    ) -> FrameResult<()> {
        let initial_impacted_values =
            Frame::get_all_inferred_connections_for_component_tree(ctx, reference_id, component_id)
                .await?;
        // do it
        Component::set_type_by_id(ctx, component_id, new_type).await?;
        let after_impacted_values =
            Frame::get_all_inferred_connections_for_component_tree(ctx, reference_id, component_id)
                .await?;
        let mut diff = HashSet::new();
        diff.extend(
            initial_impacted_values
                .difference(&after_impacted_values)
                .cloned(),
        );
        let mut inferred_edges: Vec<SummaryDiagramInferredEdge> = vec![];
        for pair in &diff {
            inferred_edges.push(SummaryDiagramInferredEdge {
                to_socket_id: pair.component_input_socket.input_socket_id,
                to_component_id: pair.component_input_socket.component_id,
                from_socket_id: pair.component_output_socket.output_socket_id,
                from_component_id: pair.component_output_socket.component_id,
                to_delete: false, // irrelevant
            })
        }
        // let the front end know what's been removed
        WsEvent::remove_inferred_edges(ctx, inferred_edges)
            .await?
            .publish_on_commit(ctx)
            .await?;
        // also get what's in current that's not in before (because these have also changed!)
        diff.extend(
            after_impacted_values
                .difference(&initial_impacted_values)
                .cloned(),
        );
        // enqueue dvu for those values that no longer have an output socket driving them!
        ctx.add_dependent_values_and_enqueue(
            diff.into_iter()
                .map(|values| values.component_input_socket.attribute_value_id)
                .collect_vec(),
        )
        .await?;
        Ok(())
    }

    /// For a pair of Components, find the top most parent of the tree (or each tree if they're not related to each other, for
    /// example, if they've been detached).
    /// Then, traverse the tree, collecting all inferred connections for all components
    /// We need the whole tree because nested components/frames might be indirectly affected by whatever the user is doing
    #[instrument(level = "info", skip(ctx), name = "frame.get_impacted_connections")]
    async fn get_all_inferred_connections_for_component_tree(
        ctx: &DalContext,
        parent_id: ComponentId,
        child_id: ComponentId,
    ) -> FrameResult<HashSet<SocketAttributeValuePair>> {
        let mut impacted_connections = HashSet::new();
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let mut inferred_connections = workspace_snapshot.inferred_connection_graph(ctx).await?;
        let mut stack_inferred_connections: HashSet<InferredConnection> = inferred_connections
            .inferred_connections_for_component_stack(parent_id)
            .await?
            .iter()
            .copied()
            .collect();
        stack_inferred_connections.extend(
            inferred_connections
                .inferred_connections_for_component_stack(child_id)
                .await?,
        );
        for incoming_connection in stack_inferred_connections {
            let component_input_socket = ComponentInputSocket {
                component_id: incoming_connection.destination_component_id,
                input_socket_id: incoming_connection.input_socket_id,
                attribute_value_id: InputSocket::component_attribute_value_id(
                    ctx,
                    incoming_connection.input_socket_id,
                    incoming_connection.destination_component_id,
                )
                .await?,
            };
            let component_output_socket = ComponentOutputSocket {
                component_id: incoming_connection.source_component_id,
                output_socket_id: incoming_connection.output_socket_id,
                attribute_value_id: OutputSocket::component_attribute_value_id(
                    ctx,
                    incoming_connection.output_socket_id,
                    incoming_connection.source_component_id,
                )
                .await?,
            };
            impacted_connections.insert(SocketAttributeValuePair {
                component_input_socket,
                component_output_socket,
            });
        }
        debug!("imapcted connections: {:?}", impacted_connections);
        Ok(impacted_connections)
    }
}
