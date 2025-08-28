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
    EdgeWeightKind,
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
        edge_weight::EdgeWeightKindDiscriminants,
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
        Component::set_type_by_id_unchecked(ctx, component_id, new_type).await?;
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

    /// Provides an ability to remove the existing ['Component']'s parent``
    #[instrument(level = "info", skip(ctx))]
    pub async fn orphan_child(ctx: &DalContext, child_id: ComponentId) -> FrameResult<()> {
        // Normally, we'd call `Component::get_parent_by_id` to get the parent's ID, but that
        // returns a hard error if there are multiple parents. Since we want to be able to use this
        // as an escape hatch for if we get into the situation of a Component having multiple
        // parents, we can't use that, and have to do the same thing it would have done to get
        // _all_ of the parents to truly orphan the child.
        let parent_idxs = ctx
            .workspace_snapshot()?
            .incoming_sources_for_edge_weight_kind(
                child_id,
                EdgeWeightKindDiscriminants::FrameContains,
            )
            .await?;
        if parent_idxs.len() == 1 {
            // We just determined that there is exactly one element in the vec, so if Vec::first()
            // returns anything other than `Some` we can't trust anything.
            let parent_idx = parent_idxs
                .first()
                .expect("Unable to get the first element of a Vec of len 1");
            let parent_id = ctx
                .workspace_snapshot()?
                .get_node_weight(*parent_idx)
                .await?
                .id()
                .into();

            Self::detach_child_from_parent_inner(ctx, parent_id, child_id).await?;
        } else {
            // When there are multiple parents, we're trying to recover from a broken state, and we
            // can't reliably detect everything necessary to do a DependentValuesUpdate, or most of
            // the other things we'd normally do. This means there won't be any WsEvents for
            // removal of the inferred edges.
            for parent_idx in parent_idxs {
                let parent_id = ctx
                    .workspace_snapshot()?
                    .get_node_weight(parent_idx)
                    .await?
                    .id()
                    .into();
                Component::remove_edge_from_frame(ctx, parent_id, child_id).await?;
            }
        }

        Ok(())
    }

    /// Provides the ability to attach or replace a child [`Component`]'s parent
    #[instrument(level = "info", skip(ctx))]
    pub async fn upsert_parent_for_tests(
        ctx: &DalContext,
        child_id: ComponentId,
        new_parent_id: ComponentId,
    ) -> FrameResult<Option<InferredEdgeChanges>> {
        Self::upsert_parent_inner_for_tests(ctx, child_id, new_parent_id, true).await
    }

    #[instrument(level = "info", skip(ctx))]
    async fn upsert_parent_inner_for_tests(
        ctx: &DalContext,
        child_id: ComponentId,
        new_parent_id: ComponentId,
        send_events: bool,
    ) -> FrameResult<Option<InferredEdgeChanges>> {
        // let's see if we need to even do anything
        if let Some(current_parent_id) = Component::get_parent_by_id(ctx, child_id).await? {
            if current_parent_id == new_parent_id {
                return Ok(None);
            }
        }

        match Component::get_type_by_id(ctx, new_parent_id).await? {
            ComponentType::ConfigurationFrameDown | ComponentType::ConfigurationFrameUp => {
                Ok(Some(
                    Self::attach_child_to_parent_inner_for_tests(
                        ctx,
                        new_parent_id,
                        child_id,
                        send_events,
                    )
                    .await?,
                ))
            }
            ComponentType::Component => Err(FrameError::ParentIsNotAFrame(child_id, new_parent_id)),
            ComponentType::AggregationFrame => {
                Err(FrameError::AggregateFramesUnsupported(new_parent_id))
            }
        }
    }

    /// Removes the existing parent connection if it exists and adds the new one.
    /// Also, determines what needs to be rerun due to the change, based on which
    /// input sockets have new/removed/different output sockets driving them
    #[instrument(level = "info", skip(ctx))]
    async fn attach_child_to_parent_inner_for_tests(
        ctx: &DalContext,
        parent_id: ComponentId,
        child_id: ComponentId,
        send_events: bool,
    ) -> FrameResult<InferredEdgeChanges> {
        // cache current map of input <-> output sockets based on what the parent knows about right now!!!!
        let initial_impacted_values: HashSet<SocketAttributeValuePair> =
            Self::get_all_inferred_connections_for_component_tree(ctx, parent_id, child_id).await?;
        // is the current child already connected to a parent?
        let mut post_edge_removal_impacted_values: HashSet<SocketAttributeValuePair> =
            HashSet::new();
        if let Some(current_parent_id) = Component::get_parent_by_id(ctx, child_id).await? {
            //remove the edge
            Component::remove_edge_from_frame(ctx, current_parent_id, child_id).await?;

            // get the map of input <-> output sockets after the edge was removed. so we can determine if more
            // updates need to be made due to the upsert
            // note we need to see what the current parent's tree looked like, as there could be nested impacts
            post_edge_removal_impacted_values.extend(
                Self::get_all_inferred_connections_for_component_tree(
                    ctx,
                    current_parent_id,
                    child_id,
                )
                .await?,
            );
        }

        let cycle_check_guard = ctx.workspace_snapshot()?.enable_cycle_check().await;
        // add the new edge
        Component::add_edge_to_frame_for_tests(
            ctx,
            parent_id,
            child_id,
            EdgeWeightKind::FrameContains,
        )
        .await?;
        drop(cycle_check_guard);
        ctx.workspace_snapshot()?
            .clear_inferred_connection_graph()
            .await;

        // now figure out what needs to rerun!
        let mut values_to_run: HashSet<SocketAttributeValuePair> = HashSet::new();

        // get the latest state of the component tree
        let current_impacted_values =
            Self::get_all_inferred_connections_for_component_tree(ctx, parent_id, child_id).await?;

        // an edge has been removed if it exists in the state after we've detached the component, and it's not in current state
        values_to_run.extend(
            post_edge_removal_impacted_values
                .difference(&current_impacted_values)
                .copied(),
        );
        // an edge has been removed if it exists before we added the new edge, and not in current
        values_to_run.extend(
            initial_impacted_values
                .difference(&current_impacted_values)
                .copied(),
        );

        // Let the frontend know what edges should be removed.
        let mut inferred_edges_to_remove: Vec<SummaryDiagramInferredEdge> =
            Vec::with_capacity(values_to_run.len());
        for pair in &values_to_run {
            inferred_edges_to_remove.push(SummaryDiagramInferredEdge {
                to_socket_id: pair.component_input_socket.input_socket_id,
                to_component_id: pair.component_input_socket.component_id,
                from_socket_id: pair.component_output_socket.output_socket_id,
                from_component_id: pair.component_output_socket.component_id,
                to_delete: false, // irrelevant
            })
        }

        if send_events {
            WsEvent::remove_inferred_edges(ctx, inferred_edges_to_remove.clone())
                .await?
                .publish_on_commit(ctx)
                .await?;
        }

        // After we let the frontend know what edges should be removed, now we should handle upsertion.
        let removed_edges_to_skip: HashSet<SummaryDiagramInferredEdge> =
            HashSet::from_iter(inferred_edges_to_remove.clone().into_iter());

        let mut inferred_edges_to_upsert = Vec::new();
        for pair in &current_impacted_values {
            let edge = SummaryDiagramInferredEdge {
                to_socket_id: pair.component_input_socket.input_socket_id,
                to_component_id: pair.component_input_socket.component_id,
                from_socket_id: pair.component_output_socket.output_socket_id,
                from_component_id: pair.component_output_socket.component_id,
                to_delete: false, // irrelevant
            };
            if !removed_edges_to_skip.contains(&edge) {
                inferred_edges_to_upsert.push(edge);
            }
        }

        if send_events {
            WsEvent::upsert_inferred_edges(ctx, inferred_edges_to_upsert.clone())
                .await?
                .publish_on_commit(ctx)
                .await?;
        }

        // an input socket needs to rerun if:
        // the input socket has a new/different output socket driving it
        values_to_run.extend(
            current_impacted_values
                .difference(&initial_impacted_values)
                .copied(),
        );

        // if we removed an edge, let's also see if there are input sockets that need to rerun
        if !post_edge_removal_impacted_values.is_empty() {
            values_to_run.extend(
                current_impacted_values
                    .difference(&post_edge_removal_impacted_values)
                    .copied(),
            );
        }

        // enqueue those values that we now know need to run
        ctx.add_dependent_values_and_enqueue(
            values_to_run
                .into_iter()
                .map(|values| values.component_input_socket.attribute_value_id)
                .collect_vec(),
        )
        .await?;

        Ok(InferredEdgeChanges {
            removed_edges: inferred_edges_to_remove,
            upserted_edges: inferred_edges_to_upsert,
        })
    }

    #[instrument(
        level = "info",
        skip(ctx),
        name = "frame.detach_child_from_parent_inner"
    )]
    async fn detach_child_from_parent_inner(
        ctx: &DalContext,
        parent_id: ComponentId,
        child_id: ComponentId,
        // get the new state of the tree (from the perspective of both components, now in disjoint trees because they were detached!)
    ) -> FrameResult<()> {
        // cache current state of the tree
        let before_change_impacted_input_sockets: HashSet<SocketAttributeValuePair> =
            Self::get_all_inferred_connections_for_component_tree(ctx, parent_id, child_id).await?;
        // remove the edge
        Component::remove_edge_from_frame(ctx, parent_id, child_id).await?;
        let current_impacted_sockets =
            Self::get_all_inferred_connections_for_component_tree(ctx, parent_id, child_id).await?;
        // find the edges that have been removed due to the detachment
        // note: there should not be any other changes as this is a pure detachment, not an upsert (where something has moved from
        // one frame to another)
        let mut diff: HashSet<SocketAttributeValuePair> = HashSet::new();
        diff.extend(
            before_change_impacted_input_sockets
                .difference(&current_impacted_sockets)
                .cloned(),
        );
        let mut inferred_edges_to_remove: Vec<SummaryDiagramInferredEdge> = vec![];
        for pair in &diff {
            inferred_edges_to_remove.push(SummaryDiagramInferredEdge {
                to_socket_id: pair.component_input_socket.input_socket_id,
                to_component_id: pair.component_input_socket.component_id,
                from_socket_id: pair.component_output_socket.output_socket_id,
                from_component_id: pair.component_output_socket.component_id,
                to_delete: false, // irrelevant
            })
        }
        // let the front end know what's been removed
        WsEvent::remove_inferred_edges(ctx, inferred_edges_to_remove.clone())
            .await?
            .publish_on_commit(ctx)
            .await?;

        // Inform the frontend of upsertions. This is a rare case but can happen under the
        // following scenario: there's a grandparent up-frame, a parent down-frame and a child
        // component or up-frame. If the parent frame has an input socket with an arity of one and
        // both the child and grandparent have an output socket with a matching annotation, then the
        // edge will be mutated. The parent will now have a different value at its input socket
        // because the source side of the edge has changed.
        {
            let removed_edges_to_skip: HashSet<SummaryDiagramInferredEdge> =
                HashSet::from_iter(inferred_edges_to_remove.into_iter());
            let mut inferred_edges_to_upsert = Vec::new();
            for pair in &current_impacted_sockets {
                // Only edges in "current" and not in "before" can be upserted.
                if !before_change_impacted_input_sockets.contains(pair) {
                    let edge = SummaryDiagramInferredEdge {
                        to_socket_id: pair.component_input_socket.input_socket_id,
                        to_component_id: pair.component_input_socket.component_id,
                        from_socket_id: pair.component_output_socket.output_socket_id,
                        from_component_id: pair.component_output_socket.component_id,
                        to_delete: false, // irrelevant
                    };
                    if !removed_edges_to_skip.contains(&edge) {
                        inferred_edges_to_upsert.push(edge);
                    }
                }
            }
            WsEvent::upsert_inferred_edges(ctx, inferred_edges_to_upsert)
                .await?
                .publish_on_commit(ctx)
                .await?;
        }

        // also get what's in current that's not in before (because these have also changed!)
        diff.extend(
            current_impacted_sockets
                .difference(&before_change_impacted_input_sockets)
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
            .inferred_connections_for_component_stack(ctx, parent_id)
            .await?
            .iter()
            .copied()
            .collect();
        stack_inferred_connections.extend(
            inferred_connections
                .inferred_connections_for_component_stack(ctx, child_id)
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
