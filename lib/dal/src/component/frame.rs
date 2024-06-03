use std::collections::{HashMap, HashSet, VecDeque};

use itertools::Itertools;
use telemetry::prelude::*;
use thiserror::Error;

use crate::attribute::value::AttributeValueError;
use crate::diagram::SummaryDiagramInferredEdge;
use crate::socket::input::InputSocketError;
use crate::workspace_snapshot::edge_weight::{EdgeWeightError, EdgeWeightKind};
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
    #[error("edge weight error: {0}")]
    EdgeWeight(#[from] EdgeWeightError),
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
    #[instrument(level = "info", skip_all)]
    pub async fn orphan_child(ctx: &DalContext, child_id: ComponentId) -> FrameResult<()> {
        if let Some(parent_id) = Component::get_parent_by_id(ctx, child_id).await? {
            Self::detach_child_from_parent_inner(ctx, parent_id, child_id).await?;
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

    #[instrument(level = "info", skip(ctx))]
    async fn attach_child_to_parent_inner(
        ctx: &DalContext,
        parent_id: ComponentId,
        child_id: ComponentId,
    ) -> FrameResult<()> {
        let total_start = std::time::Instant::now();

        // is the current child already connected to a parent?
        let mut cached_impacted_values: HashSet<SocketAttributeValuePair> = HashSet::new();
        if let Some(current_parent_id) = Component::get_parent_by_id(ctx, child_id).await? {
            // cache input sockets impacted by the child component
            let before_remove_edge_input_sockets =
                Self::get_impacted_connections(ctx, child_id).await?;

            cached_impacted_values.extend(before_remove_edge_input_sockets);
            //remove the edge
            Component::remove_edge_from_frame(ctx, current_parent_id, child_id).await?;
            info!(
                "Remove existing edge from frame took: {:?}",
                total_start.elapsed()
            );
        }

        let cycle_check_guard = ctx.workspace_snapshot()?.enable_cycle_check().await;
        Component::add_edge_to_frame(ctx, parent_id, child_id, EdgeWeightKind::FrameContains)
            .await?;
        drop(cycle_check_guard);
        info!(
            "Cycle Check Guard dropped, add edge took {:?}",
            total_start.elapsed()
        );
        let mut values_to_run: HashSet<SocketAttributeValuePair> = HashSet::new();
        let current_impacted_values = Self::get_impacted_connections(ctx, child_id).await?;

        // if an input socket + output socket is in both sets, we don't need to rerun it
        values_to_run.extend(
            cached_impacted_values
                .difference(&current_impacted_values)
                .copied(),
        );
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

        values_to_run.extend(
            current_impacted_values
                .difference(&cached_impacted_values)
                .copied(),
        );
        ctx.add_dependent_values_and_enqueue(
            values_to_run
                .into_iter()
                .map(|values| values.input_socket_match.attribute_value_id)
                .collect_vec(),
        )
        .await?;

        Ok(())
    }

    async fn detach_child_from_parent_inner(
        ctx: &DalContext,
        parent_id: ComponentId,
        child_id: ComponentId,
    ) -> FrameResult<()> {
        let before_change_impacted_input_sockets: HashSet<SocketAttributeValuePair> =
            Self::get_impacted_connections(ctx, child_id).await?;
        //when detaching a child, need to re-run any attribute value functions for those impacted input sockets then queue up dvu!
        Component::remove_edge_from_frame(ctx, parent_id, child_id).await?;

        let mut inferred_edges: Vec<SummaryDiagramInferredEdge> = vec![];
        for pair in &before_change_impacted_input_sockets {
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

        ctx.add_dependent_values_and_enqueue(
            before_change_impacted_input_sockets
                .into_iter()
                .map(|values| values.input_socket_match.attribute_value_id)
                .collect_vec(),
        )
        .await?;
        Ok(())
    }

    /// For a given [`Component`], find all input sockets that have an inferred connection. For
    /// every output socket, get all downstream input sockets that have an inferred connection to
    /// the provided [`Component`].
    async fn get_impacted_connections(
        ctx: &DalContext,
        child_id: ComponentId,
    ) -> FrameResult<HashSet<SocketAttributeValuePair>> {
        let mut input_map: HashMap<InputSocketMatch, Vec<OutputSocketMatch>> = HashMap::new();
        let mut output_map: HashMap<OutputSocketMatch, Vec<InputSocketMatch>> = HashMap::new();
        let mut impacted_connections = HashSet::new();

        // Determine whether we should check descendants and/or ascendants based on the component
        // type.
        let (check_descendants, check_ascendants) =
            match Component::get_type_by_id(ctx, child_id).await? {
                ComponentType::AggregationFrame => (false, false),
                ComponentType::Component => (false, true),
                ComponentType::ConfigurationFrameDown => (true, true),
                ComponentType::ConfigurationFrameUp => (true, true),
            };

        // Grab all descendants and see who is impacted.
        if check_descendants {
            let mut work_queue = VecDeque::new();
            let children = Component::get_children_for_id(ctx, child_id).await?;
            work_queue.extend(children);

            while let Some(child) = work_queue.pop_front() {
                let input =
                    Component::build_map_for_component_id_inferred_incoming_connections(ctx, child)
                        .await?;
                input_map.extend(input);
                let output =
                    Component::build_map_for_component_id_inferred_outgoing_connections(ctx, child)
                        .await?;
                output_map.extend(output);

                let _ = Component::get_children_for_id(ctx, child)
                    .await?
                    .into_iter()
                    .map(|comp| work_queue.push_back(comp));
            }
        }

        // Grab all ascendants and see who is impacted.
        if check_ascendants {
            let mut work_queue = VecDeque::new();
            if let Some(parent) = Component::get_parent_by_id(ctx, child_id).await? {
                work_queue.push_front(parent);
            }

            while let Some(parent) = work_queue.pop_front() {
                let input = Component::build_map_for_component_id_inferred_incoming_connections(
                    ctx, parent,
                )
                .await?;
                input_map.extend(input);
                let output = Component::build_map_for_component_id_inferred_outgoing_connections(
                    ctx, parent,
                )
                .await?;
                output_map.extend(output);
                if let Some(grandparent) = Component::get_parent_by_id(ctx, parent).await? {
                    work_queue.push_back(grandparent);
                }
            }
        }

        // Check inferred outgoing and incoming connections.
        let input: HashMap<InputSocketMatch, Vec<OutputSocketMatch>> =
            Component::build_map_for_component_id_inferred_incoming_connections(ctx, child_id)
                .await?;
        input_map.extend(input);
        let output: HashMap<OutputSocketMatch, Vec<InputSocketMatch>> =
            Component::build_map_for_component_id_inferred_outgoing_connections(ctx, child_id)
                .await?;
        output_map.extend(output);

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

        Ok(impacted_connections)
    }
}
