use std::collections::HashSet;

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use telemetry::prelude::*;
use thiserror::Error;

use crate::attribute::value::AttributeValueError;
use crate::socket::input::InputSocketError;
use crate::workspace_snapshot::edge_weight::{EdgeWeightError, EdgeWeightKind};
use crate::workspace_snapshot::WorkspaceSnapshotError;
use crate::{
    AttributeValueId, Component, ComponentError, ComponentId, ComponentType, DalContext,
    TransactionsError,
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
}
#[derive(Eq, Hash, PartialEq, Copy, Clone)]
struct SocketAttributeValuePair {
    input_attribute_value_id: AttributeValueId,
    output_attribute_value_id: AttributeValueId,
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

    async fn attach_child_to_parent_inner(
        ctx: &DalContext,
        parent_id: ComponentId,
        child_id: ComponentId,
    ) -> FrameResult<()> {
        // is the current child already connected to a parent?
        let mut cached_impacted_values: HashSet<SocketAttributeValuePair> = HashSet::new();
        if let Some(current_parent_id) = Component::get_parent_by_id(ctx, child_id).await? {
            // cache input sockets impacted by the child component
            let before_remove_edge_input_sockets =
                Self::get_impacted_connections(ctx, child_id).await?;

            cached_impacted_values.extend(before_remove_edge_input_sockets);
            //remove the edge
            Component::remove_edge_from_frame(ctx, current_parent_id, child_id).await?;
        }

        let cycle_check_guard = ctx.workspace_snapshot()?.enable_cycle_check().await;
        Component::add_edge_to_frame(ctx, parent_id, child_id, EdgeWeightKind::FrameContains)
            .await?;
        drop(cycle_check_guard);
        let mut values_to_run: HashSet<SocketAttributeValuePair> = HashSet::new();
        let current_impacted_values = Self::get_impacted_connections(ctx, child_id).await?;
        // if an input socket + output socket is in both sets, we don't need to rerun it
        values_to_run.extend(
            cached_impacted_values
                .difference(&current_impacted_values)
                .copied(),
        );

        values_to_run.extend(
            current_impacted_values
                .difference(&cached_impacted_values)
                .copied(),
        );
        ctx.enqueue_dependent_values_update(
            values_to_run
                .into_iter()
                .map(|values| values.input_attribute_value_id)
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

        ctx.enqueue_dependent_values_update(
            before_change_impacted_input_sockets
                .into_iter()
                .map(|values| values.input_attribute_value_id)
                .collect_vec(),
        )
        .await?;
        Ok(())
    }

    /// For a given component, get all of the input sockets
    /// that have an inferred connection and for every output socket, get all
    /// downstream input sockets that have an inferred connection to it
    async fn get_impacted_connections(
        ctx: &DalContext,
        child_id: ComponentId,
    ) -> FrameResult<HashSet<SocketAttributeValuePair>> {
        let input_map =
            Component::build_map_for_component_id_inferred_incoming_connections(ctx, child_id)
                .await?;
        let output_map =
            Component::build_map_for_component_id_inferred_outgoing_connections(ctx, child_id)
                .await?;

        let mut impacted_connections = HashSet::new();
        for (input_socket, output_sockets) in input_map.into_iter() {
            for output_socket in output_sockets {
                impacted_connections.insert(SocketAttributeValuePair {
                    input_attribute_value_id: input_socket.attribute_value_id,
                    output_attribute_value_id: output_socket.attribute_value_id,
                });
            }
        }
        for (output_socket, input_sockets) in output_map.into_iter() {
            for input_socket in input_sockets {
                impacted_connections.insert(SocketAttributeValuePair {
                    input_attribute_value_id: input_socket.attribute_value_id,
                    output_attribute_value_id: output_socket.attribute_value_id,
                });
            }
        }
        Ok(impacted_connections)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InferredConnection {
    pub input_socket_match: InputSocketMatch,
    pub output_socket_match: OutputSocketMatch,
}
