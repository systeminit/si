use std::collections::{HashMap, HashSet, VecDeque};

use itertools::Itertools;
use telemetry::prelude::*;
use thiserror::Error;

use crate::attribute::value::AttributeValueError;
use crate::diagram::SummaryDiagramInferredEdge;
use crate::socket::input::InputSocketError;
use crate::workspace_snapshot::edge_weight::{EdgeWeightKind, EdgeWeightKindDiscriminants};
use crate::workspace_snapshot::WorkspaceSnapshotError;
use crate::{
    Component, ComponentError, ComponentId, ComponentType, DalContext, TransactionsError, WsEvent,
    WsEventError,
};

use super::{InputSocketMatch, OutputSocketMatch};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum FrameError {
    #[error("aggregation frames unsupported: {0}")]
    AggregateFramesUnsupported(ComponentId),
    #[error("attribute value error: {0}")]
    AttributeValueError(#[from] AttributeValueError),
    #[error("component error: {0}")]
    Component(#[from] ComponentError),
    #[error("input socket error: {0}")]
    InputSocketError(#[from] InputSocketError),
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
    input_socket_match: InputSocketMatch,
    output_socket_match: OutputSocketMatch,
}

pub type FrameResult<T> = Result<T, FrameError>;

/// A unit struct containing logic for working with frames.
pub struct Frame;

impl Frame {
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
    pub async fn upsert_parent(
        ctx: &DalContext,
        child_id: ComponentId,
        new_parent_id: ComponentId,
    ) -> FrameResult<()> {
        // let's see if we need to even do anything
        if let Some(current_parent_id) = Component::get_parent_by_id(ctx, child_id).await? {
            if current_parent_id == new_parent_id {
                return Ok(());
            }
        }

        match Component::get_type_by_id(ctx, new_parent_id).await? {
            ComponentType::ConfigurationFrameDown | ComponentType::ConfigurationFrameUp => {
                Self::attach_child_to_parent_inner(ctx, new_parent_id, child_id).await?;
            }
            ComponentType::Component => {
                return Err(FrameError::ParentIsNotAFrame(child_id, new_parent_id))
            }
            ComponentType::AggregationFrame => {
                return Err(FrameError::AggregateFramesUnsupported(new_parent_id))
            }
        }
        Ok(())
    }

    /// Removes the existing parent connection if it exists and adds the new one.
    /// Also, determines what needs to be rerun due to the change, based on which
    /// input sockets have new/removed/different output sockets driving them
    #[instrument(level = "info", skip(ctx))]
    async fn attach_child_to_parent_inner(
        ctx: &DalContext,
        parent_id: ComponentId,
        child_id: ComponentId,
    ) -> FrameResult<()> {
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
        Component::add_edge_to_frame(ctx, parent_id, child_id, EdgeWeightKind::FrameContains)
            .await?;
        drop(cycle_check_guard);

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

        // let the front end know if we've removed some inferred edges
        let mut inferred_edges: Vec<SummaryDiagramInferredEdge> = vec![];
        for pair in &values_to_run {
            inferred_edges.push(SummaryDiagramInferredEdge {
                to_socket_id: pair.input_socket_match.input_socket_id,
                to_component_id: pair.input_socket_match.component_id,
                from_socket_id: pair.output_socket_match.output_socket_id,
                from_component_id: pair.output_socket_match.component_id,
                to_delete: false, // irrelevant
            })
        }
        WsEvent::remove_inferred_edges(ctx, inferred_edges)
            .await?
            .publish_on_commit(ctx)
            .await?;

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
                .map(|values| values.input_socket_match.attribute_value_id)
                .collect_vec(),
        )
        .await?;

        Ok(())
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
    ) -> FrameResult<()> {
        // cache current state of the tree
        let before_change_impacted_input_sockets: HashSet<SocketAttributeValuePair> =
            Self::get_all_inferred_connections_for_component_tree(ctx, parent_id, child_id).await?;
        // remove the edge
        Component::remove_edge_from_frame(ctx, parent_id, child_id).await?;
        // get the new state of the tree (from the perspective of both components, now in disjoint trees because they were detached!)
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
        let mut inferred_edges: Vec<SummaryDiagramInferredEdge> = vec![];
        for pair in &diff {
            inferred_edges.push(SummaryDiagramInferredEdge {
                to_socket_id: pair.input_socket_match.input_socket_id,
                to_component_id: pair.input_socket_match.component_id,
                from_socket_id: pair.output_socket_match.output_socket_id,
                from_component_id: pair.output_socket_match.component_id,
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
            current_impacted_sockets
                .difference(&before_change_impacted_input_sockets)
                .cloned(),
        );
        // enqueue dvu for those values that no longer have an output socket driving them!
        ctx.add_dependent_values_and_enqueue(
            diff.into_iter()
                .map(|values| values.input_socket_match.attribute_value_id)
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
        let mut input_map: HashMap<InputSocketMatch, Vec<OutputSocketMatch>> = HashMap::new();
        let mut output_map: HashMap<OutputSocketMatch, Vec<InputSocketMatch>> = HashMap::new();
        let mut impacted_connections = HashSet::new();

        // find the top most parent of each tree (might be the same, but that's fine)
        let mut first_top_parent = child_id;
        while let Some(parent) = Component::get_parent_by_id(ctx, first_top_parent).await? {
            first_top_parent = parent;
        }
        let mut second_top_parent = parent_id;
        while let Some(parent) = Component::get_parent_by_id(ctx, second_top_parent).await? {
            second_top_parent = parent;
        }

        // Walk down the tree of descendants and accumulate connections for every input/output socket.
        let mut work_queue = VecDeque::new();
        work_queue.push_back(first_top_parent);

        // if we're dealing with two trees, get the children for the other one too
        if first_top_parent != second_top_parent {
            work_queue.push_back(second_top_parent);
        }

        while let Some(child) = work_queue.pop_front() {
            let input =
                Component::build_map_for_component_id_inferred_incoming_connections(ctx, child)
                    .await?;
            input_map.extend(input);
            let output =
                Component::build_map_for_component_id_inferred_outgoing_connections(ctx, child)
                    .await?;
            output_map.extend(output);

            let children = Component::get_children_for_id(ctx, child).await?;
            work_queue.extend(children);
        }

        // Process everything collecting in the input map and output map.
        for (input_socket, output_sockets) in input_map.into_iter() {
            for output_socket in output_sockets {
                impacted_connections.insert(SocketAttributeValuePair {
                    input_socket_match: input_socket,
                    output_socket_match: output_socket,
                });
            }
        }
        for (output_socket, input_sockets) in output_map.into_iter() {
            for input_socket in input_sockets {
                impacted_connections.insert(SocketAttributeValuePair {
                    input_socket_match: input_socket,
                    output_socket_match: output_socket,
                });
            }
        }
        debug!("imapcted connections: {:?}", impacted_connections);
        Ok(impacted_connections)
    }
}
